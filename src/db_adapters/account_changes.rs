use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::PgConnection;
use tracing::error;

use crate::models;
use crate::schema;

/// Saves state change related to account to database
pub(crate) async fn store_account_changes(
    pool: &actix_diesel::Database<PgConnection>,
    state_changes: &[near_indexer::near_primitives::views::StateChangeWithCauseView],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
) {
    if state_changes.is_empty() {
        return;
    }

    let account_changes_models: Vec<models::account_changes::AccountChange> = state_changes
        .iter()
        .filter_map(|state_change| {
            models::account_changes::AccountChange::from_state_change_with_cause(
                state_change,
                &block_hash,
                block_timestamp,
            )
        })
        .collect();

    let mut interval = crate::INTERVAL;
    loop {
        match diesel::insert_into(schema::account_changes::table)
            .values(account_changes_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while AccountChanges were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    interval.as_millis(),
                    async_error,
                    &account_changes_models
                );
                tokio::time::sleep(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}
