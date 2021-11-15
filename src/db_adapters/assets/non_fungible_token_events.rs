use actix_diesel::dsl::AsyncRunQueryDsl;
use actix_diesel::{AsyncError, Database};
use bigdecimal::BigDecimal;
use diesel::PgConnection;
use tracing::warn;

use super::nft_types;
use crate::models;
use crate::schema;

pub(crate) async fn store_nft(
    pool: &Database<PgConnection>,
    streamer_message: &near_indexer::StreamerMessage,
) -> anyhow::Result<()> {
    for shard in &streamer_message.shards {
        collect_and_store_nft_events(pool, shard, &streamer_message.block.header.timestamp).await?;
    }
    Ok(())
}

async fn collect_and_store_nft_events(
    pool: &Database<PgConnection>,
    shard: &near_indexer::IndexerShard,
    block_timestamp: &u64,
) -> anyhow::Result<()> {
    let mut index_in_shard: i32 = 0;
    for outcome in &shard.receipt_execution_outcomes {
        let nft_events = collect_nft_events(
            outcome,
            block_timestamp,
            &shard.shard_id,
            &mut index_in_shard,
        );

        crate::await_retry_or_panic!(
            diesel::insert_into(schema::assets__non_fungible_token_events::table)
                .values(nft_events.clone())
                .execute_async(pool),
            10,
            "NonFungibleTokenEvent were adding to database".to_string(),
            &nft_events,
            &is_error_handled
        );
    }
    Ok(())
}

async fn is_error_handled(async_error: &AsyncError<diesel::result::Error>) -> bool {
    if let actix_diesel::AsyncError::Execute(diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UniqueViolation,
        ref error_info,
    )) = *async_error
    {
        let duplicate_constraint = "assets__non_fungible_token_events_pkey";
        let broken_data_constraint = "assets__non_fungible_token_events_unique";
        let constraint_name = error_info.constraint_name().unwrap_or("");
        if constraint_name == duplicate_constraint {
            // Everything is fine, we have already written this to the DB
            return true;
        } else if constraint_name == broken_data_constraint {
            warn!(
                target: crate::INDEXER_FOR_EXPLORER,
                "NFT: data inconsistency is found"
            );
        }
    }
    false
}

fn collect_nft_events(
    outcome: &near_indexer::IndexerExecutionOutcomeWithReceipt,
    block_timestamp: &u64,
    shard_id: &near_indexer::near_primitives::types::ShardId,
    index_in_shard: &mut i32,
) -> Vec<models::assets::non_fungible_token_events::NonFungibleTokenEvent> {
    let prefix = "EVENT_JSON:";
    let event_logs: Vec<nft_types::Nep171Event> = outcome.execution_outcome.outcome.logs.iter().filter_map(|untrimmed_log| {
        // Now we have only nep171 events, we both parse the logs and handle nep171 here.
        // When other event types will be added, we need to rewrite the logic
        // so that we parse the logs only once for all,
        // and then handle them for each event type separately.
        let log = untrimmed_log.trim();
        if !log.starts_with(prefix) {
            return None;
        }

        let event: nft_types::NearEvent = match serde_json::from_str::<'_, nft_types::NearEvent>(
            log[prefix.len()..].trim(),
        ) {
            Ok(result) => result,
            Err(err) => {
                warn!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "NFT: provided event log does not correspond to any of formats defined in NEP. Will ignore this event. \n {:#?} \n{:#?}",
                    err,
                    untrimmed_log,
                );
                return None;
            }
        };

        let nft_types::NearEvent::Nep171(nep171_event) = event;
        Some(nep171_event)
    }).collect();

    let mut nft_events = Vec::new();
    let contract_id = &outcome.receipt.receiver_id;
    for log in event_logs {
        match log.event_kind {
            nft_types::Nep171EventKind::NftMint(mint_events) => {
                for mint_event in mint_events {
                    let memo = mint_event.memo.unwrap_or_else(|| "".to_string());
                    for token_id in mint_event.token_ids {
                        nft_events.push(
                            models::assets::non_fungible_token_events::NonFungibleTokenEvent {
                                emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
                                emitted_at_block_timestamp: BigDecimal::from(*block_timestamp),
                                emitted_in_shard_id: BigDecimal::from(*shard_id),
                                emitted_index_of_event_entry_in_shard: *index_in_shard,
                                emitted_by_contract_account_id: contract_id.to_string(),
                                token_id: token_id.escape_default().to_string(),
                                event_kind: models::enums::NftEventKind::Mint,
                                token_old_owner_account_id: "".to_string(),
                                token_new_owner_account_id: mint_event
                                    .owner_id
                                    .escape_default()
                                    .to_string(),
                                token_authorized_account_id: "".to_string(),
                                event_memo: memo.escape_default().to_string(),
                            },
                        );
                        *index_in_shard += 1;
                    }
                }
            }
            nft_types::Nep171EventKind::NftTransfer(transfer_events) => {
                for transfer_event in transfer_events {
                    let authorized_id = transfer_event
                        .authorized_id
                        .unwrap_or_else(|| "".to_string());
                    let memo = transfer_event.memo.unwrap_or_else(|| "".to_string());
                    for token_id in transfer_event.token_ids {
                        nft_events.push(
                            models::assets::non_fungible_token_events::NonFungibleTokenEvent {
                                emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
                                emitted_at_block_timestamp: BigDecimal::from(*block_timestamp),
                                emitted_in_shard_id: BigDecimal::from(*shard_id),
                                emitted_index_of_event_entry_in_shard: *index_in_shard,
                                emitted_by_contract_account_id: contract_id.to_string(),
                                token_id: token_id.escape_default().to_string(),
                                event_kind: models::enums::NftEventKind::Transfer,
                                token_old_owner_account_id: transfer_event
                                    .old_owner_id
                                    .escape_default()
                                    .to_string(),
                                token_new_owner_account_id: transfer_event
                                    .new_owner_id
                                    .escape_default()
                                    .to_string(),
                                token_authorized_account_id: authorized_id
                                    .escape_default()
                                    .to_string(),
                                event_memo: memo.escape_default().to_string(),
                            },
                        );
                        *index_in_shard += 1;
                    }
                }
            }
            nft_types::Nep171EventKind::NftBurn(burn_events) => {
                for burn_event in burn_events {
                    let authorized_id = &burn_event.authorized_id.unwrap_or_else(|| "".to_string());
                    let memo = burn_event.memo.unwrap_or_else(|| "".to_string());
                    for token_id in burn_event.token_ids {
                        nft_events.push(
                            models::assets::non_fungible_token_events::NonFungibleTokenEvent {
                                emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
                                emitted_at_block_timestamp: BigDecimal::from(*block_timestamp),
                                emitted_in_shard_id: BigDecimal::from(*shard_id),
                                emitted_index_of_event_entry_in_shard: *index_in_shard,
                                emitted_by_contract_account_id: contract_id.to_string(),
                                token_id: token_id.escape_default().to_string(),
                                event_kind: models::enums::NftEventKind::Burn,
                                token_old_owner_account_id: burn_event
                                    .owner_id
                                    .escape_default()
                                    .to_string(),
                                token_new_owner_account_id: "".to_string(),
                                token_authorized_account_id: authorized_id
                                    .escape_default()
                                    .to_string(),
                                event_memo: memo.escape_default().to_string(),
                            },
                        );
                        *index_in_shard += 1;
                    }
                }
            }
        }
    }
    nft_events
}
