use actix_diesel::dsl::AsyncRunQueryDsl;
use actix_diesel::{AsyncError, Database};
use bigdecimal::BigDecimal;
use diesel::PgConnection;

use crate::db_adapters::{assets, CHUNK_SIZE_FOR_BATCH_INSERT};
use crate::models;
use crate::schema;

use super::event_types;

pub(crate) async fn store_nft_events(
    pool: &Database<PgConnection>,
    shard: &near_indexer::IndexerShard,
    block_timestamp: u64,
    events_with_outcomes: &[(
        assets::event_types::Nep171Event,
        &near_indexer::IndexerExecutionOutcomeWithReceipt,
    )],
) -> anyhow::Result<()> {
    let nft_events = compose_nft_db_events(events_with_outcomes, block_timestamp, &shard.shard_id);

    for chunk in nft_events.chunks(CHUNK_SIZE_FOR_BATCH_INSERT) {
        let nft_events_chunk = chunk.to_owned();
        crate::await_retry_or_panic!(
            diesel::insert_into(schema::assets__non_fungible_token_events::table)
                .values(nft_events_chunk.clone())
                .execute_async(pool),
            10,
            "NonFungibleTokenEvent were adding to database".to_string(),
            &nft_events_chunk,
            &detect_nft_db_error
        );
    }

    Ok(())
}

async fn detect_nft_db_error(async_error: &AsyncError<diesel::result::Error>) -> bool {
    assets::events::detect_db_error(
        async_error,
        "assets__non_fungible_token_events_pkey",
        "assets__non_fungible_token_events_unique",
    )
    .await
}

fn compose_nft_db_events(
    events_with_outcomes: &[(
        assets::event_types::Nep171Event,
        &near_indexer::IndexerExecutionOutcomeWithReceipt,
    )],
    block_timestamp: u64,
    shard_id: &near_indexer::near_primitives::types::ShardId,
) -> Vec<models::assets::non_fungible_token_events::NonFungibleTokenEvent> {
    let mut nft_events = Vec::new();
    for (event, outcome) in events_with_outcomes {
        let contract_id = &outcome.receipt.receiver_id;
        match &event.event_kind {
            event_types::Nep171EventKind::NftMint(mint_events) => {
                for mint_event in mint_events {
                    let memo = mint_event.memo.clone().unwrap_or_else(|| "".to_string());
                    for token_id in &mint_event.token_ids {
                        nft_events.push(
                            models::assets::non_fungible_token_events::NonFungibleTokenEvent {
                                emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
                                emitted_at_block_timestamp: BigDecimal::from(block_timestamp),
                                emitted_in_shard_id: BigDecimal::from(*shard_id),
                                emitted_index_of_event_entry_in_shard: nft_events.len() as i32,
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
                    }
                }
            }
            event_types::Nep171EventKind::NftTransfer(transfer_events) => {
                for transfer_event in transfer_events {
                    let authorized_id = transfer_event
                        .authorized_id
                        .clone()
                        .unwrap_or_else(|| "".to_string());
                    let memo = transfer_event
                        .memo
                        .clone()
                        .unwrap_or_else(|| "".to_string());
                    for token_id in &transfer_event.token_ids {
                        nft_events.push(
                            models::assets::non_fungible_token_events::NonFungibleTokenEvent {
                                emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
                                emitted_at_block_timestamp: BigDecimal::from(block_timestamp),
                                emitted_in_shard_id: BigDecimal::from(*shard_id),
                                emitted_index_of_event_entry_in_shard: nft_events.len() as i32,
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
                    }
                }
            }
            event_types::Nep171EventKind::NftBurn(burn_events) => {
                for burn_event in burn_events {
                    let authorized_id = &burn_event
                        .authorized_id
                        .clone()
                        .unwrap_or_else(|| "".to_string());
                    let memo = burn_event.memo.clone().unwrap_or_else(|| "".to_string());
                    for token_id in &burn_event.token_ids {
                        nft_events.push(
                            models::assets::non_fungible_token_events::NonFungibleTokenEvent {
                                emitted_for_receipt_id: outcome.receipt.receipt_id.to_string(),
                                emitted_at_block_timestamp: BigDecimal::from(block_timestamp),
                                emitted_in_shard_id: BigDecimal::from(*shard_id),
                                emitted_index_of_event_entry_in_shard: nft_events.len() as i32,
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
                    }
                }
            }
        }
    }
    nft_events
}
