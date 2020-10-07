use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection};
use futures::join;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

/// Saves new Accounts to database or deletes the ones should be deleted
pub(crate) async fn handle_accounts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    outcomes: &near_indexer::ExecutionOutcomesWithReceipts,
) {
    let successful_receipts_with_actions: Vec<(&near_primitives::views::ReceiptView, &Vec<near_primitives::views::ActionView>)> = outcomes
        .values()
        .filter(|outcome_with_receipt| {
            match outcome_with_receipt.execution_outcome.outcome.status {
                near_primitives::views::ExecutionStatusView::SuccessValue(_)
                | near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => true,
                _ => false,
            }
        })
        .filter_map(|outcome_with_receipt| outcome_with_receipt.receipt.as_ref())
        .filter_map(|receipt| if let near_primitives::views::ReceiptEnumView::Action { actions, .. } = &receipt.receipt {
            Some((receipt, actions))
        } else {
            None
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
    outcomes: &Vec<(&near_primitives::views::ReceiptView, &Vec<near_primitives::views::ActionView>)>,
) {
    let accounts_to_create: Vec<models::accounts::Account> = outcomes
        .iter()
        .filter_map(|(receipt, actions)| actions
            .iter()
            .filter(|action| matches!(action, near_primitives::views::ActionView::CreateAccount))
            .map(|_create_account_action| models::accounts::Account::new(
                receipt.receiver_id.to_string(),
                &receipt.receipt_id,
                )
            )
            .next()
        )
        .collect();

    loop {
        match diesel::insert_into(schema::accounts::table)
            .values(accounts_to_create.clone())
            .on_conflict_do_nothing()
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
    outcomes: &Vec<(&near_primitives::views::ReceiptView, &Vec<near_primitives::views::ActionView>)>,
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
