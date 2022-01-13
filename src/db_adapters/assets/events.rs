use crate::db_adapters::assets;
use actix_diesel::dsl::AsyncRunQueryDsl;
use actix_diesel::{AsyncError, Database};
use bigdecimal::BigDecimal;
use diesel::PgConnection;
use tracing::warn;

use super::event_types;
use crate::models;
use crate::schema;

pub(crate) async fn store_events(
    pool: &Database<PgConnection>,
    streamer_message: &near_indexer::StreamerMessage,
) -> anyhow::Result<()> {
    for shard in &streamer_message.shards {
        collect_and_store_events(pool, shard, &streamer_message.block.header.timestamp).await?;
    }
    Ok(())
}

pub(crate) async fn is_error_handled(async_error: &AsyncError<diesel::result::Error>, duplicate_constraint_name: &str, broken_data_constraint_name: &str) -> bool {
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
    block_timestamp: &u64,
) -> anyhow::Result<()> {
    let mut index_in_shard: i32 = 0;
    for outcome in &shard.receipt_execution_outcomes {
        let events = extract_events(outcome);
        for event in events {
            match event {
                assets::event_types::NearEvent::Nep141(ft_event) => {
                    assets::fungible_token_events::handle_ft_events(pool, shard, outcome, block_timestamp, &ft_event, &mut index_in_shard).await?;
                }
                assets::event_types::NearEvent::Nep171(nft_event) => {
                    assets::non_fungible_token_events::handle_nft_events(pool, shard, outcome, block_timestamp, &nft_event, &mut index_in_shard).await?;
                }
            }
        }
    }
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
