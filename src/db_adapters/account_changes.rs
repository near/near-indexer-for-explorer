use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::PgConnection;
use futures::future::try_join_all;

use crate::schema;
use crate::{metrics, models};

pub(crate) async fn store_account_changes(
    pool: &actix_diesel::Database<PgConnection>,
    shards: &[near_indexer::IndexerShard],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let _timer = metrics::STORE_TIME
        .with_label_values(&["AccountChanges"])
        .start_timer();
    let futures = shards.iter().map(|shard| {
        store_account_changes_for_shard(pool, &shard.state_changes, block_hash, block_timestamp)
    });

    try_join_all(futures).await.map(|_| ())
}

/// Saves state change related to account to database
async fn store_account_changes_for_shard(
    pool: &actix_diesel::Database<PgConnection>,
    state_changes: &[near_indexer::near_primitives::views::StateChangeWithCauseView],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    if state_changes.is_empty() {
        return Ok(());
    }

    let account_changes_models: Vec<models::account_changes::AccountChange> = state_changes
        .iter()
        .enumerate()
        .filter_map(|(index_in_block, state_change)| {
            models::account_changes::AccountChange::from_state_change_with_cause(
                state_change,
                block_hash,
                block_timestamp,
                index_in_block as i32,
            )
        })
        .collect();

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::account_changes::table)
            .values(account_changes_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "AccountChanges were stored in database".to_string(),
        &account_changes_models
    );
    Ok(())
}
