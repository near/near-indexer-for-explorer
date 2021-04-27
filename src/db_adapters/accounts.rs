use std::collections::HashMap;

use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use futures::{join, StreamExt};
use itertools::Itertools;
use tracing::{error, info};

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

/// Saves new Accounts to database or deletes the ones should be deleted
pub(crate) async fn handle_accounts(
    pool: &actix_diesel::Database<PgConnection>,
    outcomes: &[near_indexer::IndexerExecutionOutcomeWithReceipt],
    block_height: near_primitives::types::BlockHeight,
) {
    if outcomes.is_empty() {
        return;
    }
    let successful_receipts = outcomes
        .iter()
        .filter(|outcome_with_receipt| {
            match outcome_with_receipt.execution_outcome.outcome.status {
                near_primitives::views::ExecutionStatusView::SuccessValue(_)
                | near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => true,
                _ => false,
            }
        })
        .map(|outcome_with_receipt| &outcome_with_receipt.receipt);

    let mut accounts =
        HashMap::<near_primitives::types::AccountId, models::accounts::Account>::new();

    for receipt in successful_receipts {
        if let near_primitives::views::ReceiptEnumView::Action { actions, .. } = &receipt.receipt {
            for action in actions {
                match action {
                    near_primitives::views::ActionView::CreateAccount => {
                        accounts.insert(
                            receipt.receiver_id.clone(),
                            models::accounts::Account::new_from_receipt(
                                receipt.receiver_id.to_string(),
                                &receipt.receipt_id,
                                block_height,
                            ),
                        );
                    }
                    near_primitives::views::ActionView::Transfer { .. } => {
                        if receipt.receiver_id.len() == 64usize {
                            accounts.insert(
                                receipt.receiver_id.clone(),
                                models::accounts::Account::new_from_receipt(
                                    receipt.receiver_id.to_string(),
                                    &receipt.receipt_id,
                                    block_height,
                                ),
                            );
                        }
                    }
                    near_primitives::views::ActionView::DeleteAccount { .. } => {
                        accounts
                            .entry(receipt.receiver_id.clone())
                            .and_modify(|existing_account| {
                                existing_account.deleted_by_receipt_id =
                                    Some(receipt.receipt_id.to_string())
                            })
                            .or_insert_with(|| models::accounts::Account {
                                account_id: receipt.receiver_id.to_string(),
                                created_by_receipt_id: None,
                                deleted_by_receipt_id: Some(receipt.receipt_id.to_string()),
                                last_update_block_height: block_height.into(),
                            });
                    }
                    _ => {}
                }
            }
        }
    }

    let (accounts_to_insert, accounts_to_update): (
        Vec<models::accounts::Account>,
        Vec<models::accounts::Account>,
    ) = accounts
        .values()
        .cloned()
        .partition(|model| model.created_by_receipt_id.is_some());

    let update_accounts_future = async {
        for value in accounts_to_update {
            let target = schema::accounts::table
                .filter(schema::accounts::dsl::account_id.eq(value.account_id))
                .filter(
                    schema::accounts::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                );

            let mut interval = crate::INTERVAL;
            loop {
                match diesel::update(target.clone())
                    .set((
                        schema::accounts::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::accounts::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(&pool)
                    .await
                {
                    Ok(_) => break,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while updating Account. Retry in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::sleep(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        }
    };

    let insert_accounts_future = async {
        let mut interval = crate::INTERVAL;
        loop {
            match diesel::insert_into(schema::accounts::table)
                .values(accounts_to_insert.clone())
                .on_conflict_do_nothing()
                .execute_async(&pool)
                .await
            {
                Ok(_) => break,
                Err(async_error) => {
                    error!(
                        target: crate::INDEXER_FOR_EXPLORER,
                        "Error occurred while Accounts were adding to database. Retrying in {} milliseconds... \n {:#?}",
                        interval.as_millis(),
                        async_error,
                    );
                    tokio::time::sleep(interval).await;
                    if interval < crate::MAX_DELAY_TIME {
                        interval *= 2;
                    }
                }
            }
        }

        for value in accounts_to_insert {
            let target = schema::accounts::table
                .filter(schema::accounts::dsl::account_id.eq(value.account_id))
                .filter(
                    schema::accounts::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                );

            let mut interval = crate::INTERVAL;
            loop {
                match diesel::update(target.clone())
                    .set((
                        schema::accounts::dsl::created_by_receipt_id
                            .eq(value.created_by_receipt_id.clone()),
                        schema::accounts::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::accounts::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(&pool)
                    .await
                {
                    Ok(_) => break,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while updating Account. Retry in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::sleep(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        }
    };

    // Joining it unless we can't execute it in the correct order
    // see https://github.com/nearprotocol/nearcore/issues/3467
    join!(update_accounts_future, insert_accounts_future);
}

pub(crate) async fn store_accounts_from_genesis(near_config: near_indexer::NearConfig) {
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Adding/updating accounts from genesis..."
    );
    let pool = crate::models::establish_connection();
    let genesis_height = near_config.genesis.config.genesis_height;

    let accounts_models = near_config
        .genesis
        .records
        .as_ref()
        .iter()
        .filter_map(|record| {
            if let near_indexer::near_primitives::state_record::StateRecord::Account {
                account_id,
                ..
            } = record
            {
                Some(models::accounts::Account::new_from_genesis(
                    account_id.to_string(),
                    genesis_height,
                ))
            } else {
                None
            }
        });

    let portion_size = 5000;
    let total_accounts_chunks = accounts_models.clone().count() / portion_size + 1;
    let accounts_portion = accounts_models.chunks(portion_size);

    let insert_genesis_accounts: futures::stream::FuturesUnordered<_> = accounts_portion
        .into_iter()
        .map(|accounts| async {
            let collected_accounts = accounts.collect::<Vec<models::accounts::Account>>();
            let mut interval = crate::INTERVAL;
            loop {
                match diesel::insert_into(schema::accounts::table)
                    .values(collected_accounts.clone())
                    .on_conflict_do_nothing()
                    .execute_async(&pool)
                    .await
                {
                    Ok(result) => break result,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while Accounts from genesis were being added to database. Retrying in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::sleep(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        })
        .collect();

    let mut insert_genesis_accounts = insert_genesis_accounts.enumerate();

    while let Some((index, _result)) = insert_genesis_accounts.next().await {
        info!(
            target: crate::INDEXER_FOR_EXPLORER,
            "Accounts from genesis adding {}%",
            index * 100 / total_accounts_chunks
        );
    }

    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Accounts from genesis were added/updated successful."
    );
}
