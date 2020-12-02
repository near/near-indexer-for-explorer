use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use crate::models;
use crate::schema;

/// Saves state change related to account to database
pub(crate) async fn store_state_changes(
    pool: &Pool<ConnectionManager<PgConnection>>,
    state_changes: &[near_indexer::near_primitives::views::StateChangeWithCauseView],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
) {
    if state_changes.is_empty() {
        return;
    }

    let state_changes_models: Vec<models::state_changes::StateChange> = state_changes
        .iter()
        .filter_map(|state_change| {
            models::state_changes::StateChange::from_state_change_with_cause(
                state_change,
                &block_hash,
                block_timestamp,
            )
        })
        .collect();

    let mut interval = crate::INTERVAL;
    loop {
        match diesel::insert_into(schema::state_changes::table)
            .values(state_changes_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while StateChanges were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    interval.as_millis(),
                    async_error,
                    &state_changes_models
                );
                tokio::time::delay_for(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}
