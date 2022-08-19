use std::collections::HashMap;

use actix_diesel::dsl::AsyncRunQueryDsl;

use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use futures::try_join;

use crate::models;
use crate::schema;

/// Saves new Accounts to database or deletes the ones should be deleted
pub(crate) async fn handle_accounts(
    pool: &actix_diesel::Database<PgConnection>,
    outcomes: &[near_lake_framework::near_indexer_primitives::IndexerExecutionOutcomeWithReceipt],
    block_height: near_lake_framework::near_indexer_primitives::types::BlockHeight,
) -> anyhow::Result<()> {
    if outcomes.is_empty() {
        return Ok(());
    }
    let successful_receipts = outcomes
        .iter()
        .filter(|outcome_with_receipt| {
            matches!(
                outcome_with_receipt.execution_outcome.outcome.status,
                near_lake_framework::near_indexer_primitives::views::ExecutionStatusView::SuccessValue(_)
                    | near_lake_framework::near_indexer_primitives::views::ExecutionStatusView::SuccessReceiptId(_)
            )
        })
        .map(|outcome_with_receipt| &outcome_with_receipt.receipt);

    let mut accounts = HashMap::<
        near_lake_framework::near_indexer_primitives::types::AccountId,
        models::accounts::Account,
    >::new();

    for receipt in successful_receipts {
        if let near_lake_framework::near_indexer_primitives::views::ReceiptEnumView::Action {
            actions,
            ..
        } = &receipt.receipt
        {
            for action in actions {
                match action {
                    near_lake_framework::near_indexer_primitives::views::ActionView::CreateAccount => {
                        accounts.insert(
                            receipt.receiver_id.clone(),
                            models::accounts::Account::new_from_receipt(
                                &receipt.receiver_id,
                                &receipt.receipt_id,
                                block_height,
                            ),
                        );
                    }
                    near_lake_framework::near_indexer_primitives::views::ActionView::Transfer { .. } => {
                        if receipt.receiver_id.len() == 64usize {
                            accounts.insert(
                                receipt.receiver_id.clone(),
                                models::accounts::Account::new_from_receipt(
                                    &receipt.receiver_id,
                                    &receipt.receipt_id,
                                    block_height,
                                ),
                            );
                        }
                    }
                    near_lake_framework::near_indexer_primitives::views::ActionView::DeleteAccount { .. } => {
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

    let (accounts_to_create_or_update, accounts_to_delete): (
        Vec<models::accounts::Account>,
        Vec<models::accounts::Account>,
    ) = accounts
        .values()
        .cloned()
        .partition(|model| model.created_by_receipt_id.is_some());

    let delete_accounts_future = async {
        for value in accounts_to_delete {
            let target = schema::accounts::table
                .filter(schema::accounts::dsl::account_id.eq(value.account_id.clone()))
                .filter(
                    schema::accounts::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                );

            crate::await_retry_or_panic!(
                diesel::update(target.clone())
                    .set((
                        schema::accounts::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::accounts::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(pool),
                10,
                "Accounts were deleted".to_string(),
                &value.account_id
            );
        }
        Ok(())
    };

    let create_or_update_accounts_future = async {
        crate::await_retry_or_panic!(
            diesel::insert_into(schema::accounts::table)
                .values(accounts_to_create_or_update.clone())
                .on_conflict_do_nothing()
                .execute_async(pool),
            10,
            "Accounts were created/updated".to_string(),
            &accounts_to_create_or_update
        );

        // [Implicit accounts](https://docs.near.org/docs/roles/integrator/implicit-accounts)
        // pretend to be created on each transfer to these accounts and cause some confusion
        // Resolving the issue https://github.com/near/near-indexer-for-explorer/issues/68 to avoid confusion
        // we block updating `created_by_receipt_id` for implicit accounts that were not deleted
        // (have `deleted_by_receipt_id` NOT NULL)
        // For this purpose we separate such accounts from others to handle them properly
        let (implicit_accounts_to_recreate, other_accounts_to_update): (
            Vec<models::accounts::Account>,
            Vec<models::accounts::Account>,
        ) = accounts_to_create_or_update.into_iter().partition(|model| {
            model.account_id.len() == 64 && model.deleted_by_receipt_id.is_none()
        });

        for value in implicit_accounts_to_recreate {
            let target = schema::accounts::table
                .filter(schema::accounts::dsl::account_id.eq(value.account_id.clone()))
                .filter(schema::accounts::dsl::deleted_by_receipt_id.is_not_null()) // this filter ensures we update only "deleted" accounts
                .filter(
                    schema::accounts::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                );

            crate::await_retry_or_panic!(
                diesel::update(target.clone())
                    .set((
                        schema::accounts::dsl::created_by_receipt_id
                            .eq(value.created_by_receipt_id.clone()),
                        schema::accounts::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::accounts::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(pool),
                10,
                "Implicit Account were updated".to_string(),
                &value.account_id
            );
        }

        for value in other_accounts_to_update {
            let target = schema::accounts::table
                .filter(schema::accounts::dsl::account_id.eq(value.account_id.clone()))
                .filter(
                    schema::accounts::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                );

            crate::await_retry_or_panic!(
                diesel::update(target.clone())
                    .set((
                        schema::accounts::dsl::created_by_receipt_id
                            .eq(value.created_by_receipt_id.clone()),
                        schema::accounts::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::accounts::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(pool),
                10,
                "Account was updated".to_string(),
                &value.account_id
            );
        }
        Ok(())
    };

    // Joining it unless we can't execute it in the correct order
    // see https://github.com/nearprotocol/nearcore/issues/3467
    try_join!(delete_accounts_future, create_or_update_accounts_future)?;
    Ok(())
}
