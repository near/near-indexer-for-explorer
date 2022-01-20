use actix_diesel::{AsyncError, Database};
use diesel::PgConnection;
use tracing::warn;

use crate::db_adapters::assets;

use super::event_types;

pub(crate) async fn store_events(
    pool: &Database<PgConnection>,
    streamer_message: &near_indexer::StreamerMessage,
) -> anyhow::Result<()> {
    let futures = streamer_message.shards.iter().map(|shard| {
        collect_and_store_events(pool, shard, streamer_message.block.header.timestamp)
    });

    futures::future::try_join_all(futures).await.map(|_| ())
}

pub(crate) async fn detect_db_error(
    async_error: &AsyncError<diesel::result::Error>,
    duplicate_constraint_name: &str,
    broken_data_constraint_name: &str,
) -> bool {
    if let actix_diesel::AsyncError::Execute(diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UniqueViolation,
        ref error_info,
    )) = *async_error
    {
        let constraint_name = error_info.constraint_name().unwrap_or("");
        if constraint_name == duplicate_constraint_name {
            // Everything is fine, we have already written this to the DB
            return true;
        } else if constraint_name == broken_data_constraint_name {
            warn!(
                target: crate::INDEXER_FOR_EXPLORER,
                "assets::events: data inconsistency is found"
            );
        }
    }
    false
}

async fn collect_and_store_events(
    pool: &Database<PgConnection>,
    shard: &near_indexer::IndexerShard,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let mut ft_events_with_outcomes = Vec::new();
    let mut nft_events_with_outcomes = Vec::new();

    for outcome in &shard.receipt_execution_outcomes {
        let events = extract_events(outcome);
        for event in events {
            match event {
                assets::event_types::NearEvent::Nep141(ft_event) => {
                    ft_events_with_outcomes.push((ft_event, outcome));
                }
                assets::event_types::NearEvent::Nep171(nft_event) => {
                    nft_events_with_outcomes.push((nft_event, outcome));
                }
            }
        }
    }

    let ft_future = assets::fungible_token_events::store_ft_events(
        pool,
        shard,
        block_timestamp,
        &ft_events_with_outcomes,
    );
    let nft_future = assets::non_fungible_token_events::store_nft_events(
        pool,
        shard,
        block_timestamp,
        &nft_events_with_outcomes,
    );
    futures::try_join!(ft_future, nft_future)?;
    Ok(())
}

fn extract_events(
    outcome: &near_indexer::IndexerExecutionOutcomeWithReceipt,
) -> Vec<event_types::NearEvent> {
    let prefix = "EVENT_JSON:";
    outcome.execution_outcome.outcome.logs.iter().filter_map(|untrimmed_log| {
        let log = untrimmed_log.trim();
        if !log.starts_with(prefix) {
            return None;
        }

        match serde_json::from_str::<'_, event_types::NearEvent>(
            log[prefix.len()..].trim(),
        ) {
            Ok(result) => Some(result),
            Err(err) => {
                warn!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Provided event log does not correspond to any of formats defined in NEP. Will ignore this event. \n {:#?} \n{:#?}",
                    err,
                    untrimmed_log,
                );
                None
            }
        }
    }).collect()
}
