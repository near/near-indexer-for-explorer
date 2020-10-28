use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection};
use futures::{join, StreamExt};
use itertools::Itertools;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::{error, info};

use near_indexer::near_primitives;

use crate::models;
use crate::schema;
use diesel::pg::upsert::excluded;

/// Saves new Accounts to database or deletes the ones should be deleted
pub(crate) async fn handle_accounts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    outcomes: &near_indexer::ExecutionOutcomesWithReceipts,
) {
    if outcomes.is_empty() {
        return;
    }
    let successful_receipts_with_actions: Vec<(
        &near_primitives::views::ReceiptView,
        &Vec<near_primitives::views::ActionView>,
    )> = outcomes
        .values()
        .filter(|outcome_with_receipt| {
            match outcome_with_receipt.execution_outcome.outcome.status {
                near_primitives::views::ExecutionStatusView::SuccessValue(_)
                | near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => true,
                _ => false,
            }
        })
        .filter_map(|outcome_with_receipt| outcome_with_receipt.receipt.as_ref())
        .filter_map(|receipt| {
            if let near_primitives::views::ReceiptEnumView::Action { actions, .. } =
                &receipt.receipt
            {
                Some((receipt, actions))
            } else {
                None
            }
        })
        .collect();

    let store_accounts_future = store_accounts(&pool, &successful_receipts_with_actions);
    let remove_accounts_future = remove_accounts(&pool, &successful_receipts_with_actions);

    // Joining it unless we can't execute it in the correct order
    // see https://github.com/nearprotocol/nearcore/issues/3467
    join!(store_accounts_future, remove_accounts_future);
}

async fn store_accounts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    outcomes: &[(
        &near_primitives::views::ReceiptView,
        &Vec<near_primitives::views::ActionView>,
    )],
) {
    let accounts_to_create: Vec<models::accounts::Account> = outcomes
        .iter()
        .filter_map(|(receipt, actions)| {
            actions
                .iter()
                .filter_map(|action| match action {
                    near_primitives::views::ActionView::CreateAccount => {
                        Some(models::accounts::Account::new_from_receipt(
                            receipt.receiver_id.to_string(),
                            &receipt.receipt_id,
                        ))
                    }
                    near_primitives::views::ActionView::Transfer { .. } => {
                        if receipt.receiver_id.len() == 64usize {
                            Some(models::accounts::Account::new_from_receipt(
                                receipt.receiver_id.to_string(),
                                &receipt.receipt_id,
                            ))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .next()
        })
        .collect();

    loop {
        match diesel::insert_into(schema::accounts::table)
            .values(accounts_to_create.clone())
            .on_conflict(schema::accounts::dsl::account_id)
            .do_update()
            .set((
                schema::accounts::dsl::created_by_receipt_id
                    .eq(excluded(schema::accounts::dsl::created_by_receipt_id)),
                schema::accounts::dsl::deleted_by_receipt_id
                    .eq(excluded(schema::accounts::dsl::deleted_by_receipt_id)),
            ))
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while Accounts were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &accounts_to_create,
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }
}

async fn remove_accounts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    outcomes: &[(
        &near_primitives::views::ReceiptView,
        &Vec<near_primitives::views::ActionView>,
    )],
) {
    let accounts_to_delete: Vec<(String, String)> = outcomes
        .iter()
        .filter_map(|(receipt, actions)| actions
            .iter()
            .filter(|action| matches!(action, near_primitives::views::ActionView::DeleteAccount { .. }))
            .map(|_delete_account_action| (receipt.receiver_id.to_string(), receipt.receipt_id.to_string()) )
            .next()
        )
        .collect();

    for (account_id, deleted_by_receipt_id) in accounts_to_delete {
        loop {
            match diesel::update(schema::accounts::table)
                .filter(schema::accounts::dsl::account_id.eq(account_id.clone()))
                .set(schema::accounts::dsl::deleted_by_receipt_id.eq(deleted_by_receipt_id.clone()))
                .execute_async(&pool)
                .await
            {
                Ok(_) => break,
                Err(async_error) => {
                    error!(
                        target: crate::INDEXER_FOR_EXPLORER,
                        "Error occurred while Account were deleted from database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                        crate::INTERVAL.as_millis(),
                        async_error,
                        &account_id,
                    );
                    tokio::time::delay_for(crate::INTERVAL).await;
                }
            }
        }
    }
}

pub(crate) async fn store_accounts_from_genesis(near_config: near_indexer::NearConfig) {
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Adding/updating accounts from genesis..."
    );
    let pool = crate::models::establish_connection();

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
                            crate::INTERVAL.as_millis(),
                            async_error,
                        );
                        tokio::time::delay_for(crate::INTERVAL).await;
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
