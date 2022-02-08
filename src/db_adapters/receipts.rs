use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use near_indexer::near_primitives;

use actix_diesel::dsl::AsyncRunQueryDsl;
use cached::Cached;
use diesel::pg::expression::array_comparison::any;
use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl};
use futures::future::try_join_all;
use futures::try_join;
use num_traits::cast::FromPrimitive;
use tracing::{error, warn};

use crate::models;
use crate::schema;

/// Saves receipts to database
pub(crate) async fn store_receipts(
    pool: &actix_diesel::Database<PgConnection>,
    shards: &[near_indexer::IndexerShard],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
    strict_mode: bool,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .filter(|chunk| !chunk.receipts.is_empty())
        .map(|chunk| {
            store_chunk_receipts(
                pool,
                &chunk.receipts,
                block_hash,
                &chunk.header.chunk_hash,
                block_timestamp,
                strict_mode,
                receipts_cache.clone(),
            )
        });

    try_join_all(futures).await.map(|_| ())
}

async fn store_chunk_receipts(
    pool: &actix_diesel::Database<PgConnection>,
    receipts: &[near_indexer::near_primitives::views::ReceiptView],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
    strict_mode: bool,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let mut skipping_receipt_ids =
        std::collections::HashSet::<near_indexer::near_primitives::hash::CryptoHash>::new();

    let tx_hashes_for_receipts = find_tx_hashes_for_receipts(
        pool,
        receipts.to_vec(),
        strict_mode,
        block_hash,
        chunk_hash,
        std::sync::Arc::clone(&receipts_cache),
    )
    .await?;

    let receipt_models: Vec<models::receipts::Receipt> = receipts
        .iter()
        .enumerate()
        .filter_map(|(index, r)| {
            // We need to search for parent transaction hash in cache differently
            // depending on the Receipt kind
            // In case of Action Receipt we are looking for ReceiptId
            // In case of Data Receipt we are looking for DataId
            let receipt_or_data_id = match r.receipt {
                near_primitives::views::ReceiptEnumView::Action { .. } => {
                    crate::ReceiptOrDataId::ReceiptId(r.receipt_id)
                }
                near_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                    crate::ReceiptOrDataId::DataId(data_id)
                }
            };
            if let Some(transaction_hash) = tx_hashes_for_receipts.get(&receipt_or_data_id) {
                Some(models::Receipt::from_receipt_view(
                        r,
                        block_hash,
                        transaction_hash,
                        chunk_hash,
                        index as i32,
                        block_timestamp,
                    ))
            } else {
                warn!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Skipping Receipt {} as we can't find parent Transaction for it. Happen in block hash {}, chunk hash {}",
                    r.receipt_id.to_string(),
                    block_hash,
                    chunk_hash,
                );
                skipping_receipt_ids.insert(r.receipt_id);
                None
            }
        })
        .collect();

    // At the moment we can observe output data in the Receipt it's impossible to know
    // the Receipt Id of that Data Receipt. That's why we insert the pair DataId<>ParentTransactionHash
    // to ReceiptsCache
    let mut receipts_cache_lock = receipts_cache.lock().await;
    for receipt in receipts {
        if let near_primitives::views::ReceiptEnumView::Action {
            output_data_receivers,
            ..
        } = &receipt.receipt
        {
            if !output_data_receivers.is_empty() {
                if let Some(transaction_hash) = tx_hashes_for_receipts
                    .get(&crate::ReceiptOrDataId::ReceiptId(receipt.receipt_id))
                {
                    for data_receiver in output_data_receivers {
                        receipts_cache_lock.cache_set(
                            crate::ReceiptOrDataId::DataId(data_receiver.data_id),
                            transaction_hash.clone(),
                        );
                    }
                }
            }
        }
    }
    // releasing the lock
    drop(receipts_cache_lock);

    save_receipts(pool, receipt_models).await?;

    let (action_receipts, data_receipts): (
        Vec<&near_indexer::near_primitives::views::ReceiptView>,
        Vec<&near_indexer::near_primitives::views::ReceiptView>,
    ) = receipts
        .iter()
        .filter(|r| !skipping_receipt_ids.contains(&r.receipt_id))
        .partition(|receipt| {
            matches!(
                receipt.receipt,
                near_indexer::near_primitives::views::ReceiptEnumView::Action { .. }
            )
        });

    let process_receipt_actions_future =
        store_receipt_actions(pool, action_receipts, block_timestamp);

    let process_receipt_data_future = store_receipt_data(pool, data_receipts);

    try_join!(process_receipt_actions_future, process_receipt_data_future)?;
    Ok(())
}

/// Looks for already created parent transaction hash for given receipts
async fn find_tx_hashes_for_receipts(
    pool: &actix_diesel::Database<PgConnection>,
    mut receipts: Vec<near_indexer::near_primitives::views::ReceiptView>,
    strict_mode: bool,
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<HashMap<crate::ReceiptOrDataId, crate::ParentTransactionHashString>> {
    let mut tx_hashes_for_receipts: HashMap<
        crate::ReceiptOrDataId,
        crate::ParentTransactionHashString,
    > = HashMap::new();

    let mut receipts_cache_lock = receipts_cache.lock().await;
    // add receipt-transaction pairs from the cache to the response
    tx_hashes_for_receipts.extend(receipts.iter().filter_map(|receipt| {
        match receipt.receipt {
            near_primitives::views::ReceiptEnumView::Action { .. } => receipts_cache_lock
                .cache_get(&crate::ReceiptOrDataId::ReceiptId(receipt.receipt_id))
                .map(|parent_transaction_hash| {
                    (
                        crate::ReceiptOrDataId::ReceiptId(receipt.receipt_id),
                        parent_transaction_hash.clone(),
                    )
                }),
            near_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                // Pair DataId:ParentTransactionHash won't be used after this moment
                // We want to clean it up to prevent our cache from growing
                receipts_cache_lock
                    .cache_remove(&crate::ReceiptOrDataId::DataId(data_id))
                    .map(|parent_transaction_hash| {
                        (
                            crate::ReceiptOrDataId::DataId(data_id),
                            parent_transaction_hash,
                        )
                    })
            }
        }
    }));
    // releasing the lock
    drop(receipts_cache_lock);

    // discard the Receipts already in cache from the attempts to search
    receipts.retain(|r| match r.receipt {
        near_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
            !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::DataId(data_id))
        }
        near_primitives::views::ReceiptEnumView::Action { .. } => {
            !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
        }
    });
    if receipts.is_empty() {
        return Ok(tx_hashes_for_receipts);
    }

    warn!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Looking for parent transaction hash in database for {} receipts {:#?}",
        &receipts.len(),
        &receipts,
    );

    let mut retries_left: u8 = 4; // retry at least times even in no-strict mode to avoid data loss
    let mut find_tx_retry_interval = crate::INTERVAL;
    loop {
        let data_ids: Vec<String> = receipts
            .iter()
            .filter_map(|r| match r.receipt {
                near_indexer::near_primitives::views::ReceiptEnumView::Data { data_id, .. } => {
                    Some(data_id.to_string())
                }
                _ => None,
            })
            .collect();
        if !data_ids.is_empty() {
            let mut interval = crate::INTERVAL;
            let tx_hashes_for_data_id_via_data_output: Vec<(
                crate::ReceiptOrDataId,
                crate::ParentTransactionHashString,
            )> = loop {
                match schema::action_receipt_output_data::table
                    .inner_join(
                        schema::receipts::table.on(
                            schema::action_receipt_output_data::dsl::output_from_receipt_id
                                .eq(schema::receipts::dsl::receipt_id),
                        ),
                    )
                    .filter(
                        schema::action_receipt_output_data::dsl::output_data_id
                            .eq(any(data_ids.clone())),
                    )
                    .select((
                        schema::action_receipt_output_data::dsl::output_data_id,
                        schema::receipts::dsl::originated_from_transaction_hash,
                    ))
                    .load_async(pool)
                    .await
                {
                    Ok(res) => {
                        break res
                            .into_iter()
                            .map(
                                |(receipt_id_string, transaction_hash_string): (String, String)| {
                                    (
                                        crate::ReceiptOrDataId::ReceiptId(
                                            near_primitives::hash::CryptoHash::from_str(
                                                &receipt_id_string,
                                            )
                                            .expect("Failed to convert String to CryptoHash"),
                                        ),
                                        transaction_hash_string,
                                    )
                                },
                            )
                            .collect();
                    }
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while fetching the parent receipt for Receipt. Retrying in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::sleep(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            };

            let mut tx_hashes_for_data_id_via_data_output_hashmap =
                HashMap::<crate::ReceiptOrDataId, crate::ParentTransactionHashString>::new();
            tx_hashes_for_data_id_via_data_output_hashmap
                .extend(tx_hashes_for_data_id_via_data_output);
            let tx_hashes_for_receipts_via_data_output: Vec<(
                crate::ReceiptOrDataId,
                crate::ParentTransactionHashString,
            )> = receipts
                .iter()
                .filter_map(|r| match r.receipt {
                    near_indexer::near_primitives::views::ReceiptEnumView::Data {
                        data_id, ..
                    } => tx_hashes_for_data_id_via_data_output_hashmap
                        .get(&crate::ReceiptOrDataId::DataId(data_id))
                        .map(|tx_hash| {
                            (
                                crate::ReceiptOrDataId::ReceiptId(r.receipt_id),
                                tx_hash.to_string(),
                            )
                        }),
                    _ => None,
                })
                .collect();

            let found_hashes_len = tx_hashes_for_receipts_via_data_output.len();
            tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_data_output);

            if found_hashes_len == receipts.len() {
                break;
            }

            receipts.retain(|r| {
                !tx_hashes_for_receipts
                    .contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
            });
        }

        let tx_hashes_for_receipts_via_outcomes: Vec<(String, crate::ParentTransactionHashString)> =
            crate::await_retry_or_panic!(
                schema::execution_outcome_receipts::table
                    .inner_join(
                        schema::receipts::table
                            .on(schema::execution_outcome_receipts::dsl::executed_receipt_id
                                .eq(schema::receipts::dsl::receipt_id)),
                    )
                    .filter(
                        schema::execution_outcome_receipts::dsl::produced_receipt_id.eq(any(
                            receipts
                                .clone()
                                .iter()
                                .filter(|r| {
                                    matches!(
                                r.receipt,
                                near_indexer::near_primitives::views::ReceiptEnumView::Action { .. }
                            )
                                })
                                .map(|r| r.receipt_id.to_string())
                                .collect::<Vec<String>>()
                        )),
                    )
                    .select((
                        schema::execution_outcome_receipts::dsl::produced_receipt_id,
                        schema::receipts::dsl::originated_from_transaction_hash,
                    ))
                    .load_async::<(String, crate::ParentTransactionHashString)>(pool),
                10,
                "Parent Transaction for Receipts were fetched".to_string(),
                &receipts
            )
            .unwrap_or_default();

        let found_hashes_len = tx_hashes_for_receipts_via_outcomes.len();
        tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_outcomes.into_iter().map(
            |(receipt_id_string, transaction_hash_string)| {
                (
                    crate::ReceiptOrDataId::ReceiptId(
                        near_primitives::hash::CryptoHash::from_str(&receipt_id_string)
                            .expect("Failed to convert String to CryptoHash"),
                    ),
                    transaction_hash_string,
                )
            },
        ));

        if found_hashes_len == receipts.len() {
            break;
        }

        receipts.retain(|r| {
            !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
        });

        let tx_hashes_for_receipt_via_transactions: Vec<(
            String,
            crate::ParentTransactionHashString,
        )> = crate::await_retry_or_panic!(
            schema::transactions::table
                .filter(
                    schema::transactions::dsl::converted_into_receipt_id.eq(any(receipts
                        .clone()
                        .iter()
                        .filter(|r| {
                            matches!(
                                r.receipt,
                                near_indexer::near_primitives::views::ReceiptEnumView::Action { .. }
                            )
                        })
                        .map(|r| r.receipt_id.to_string())
                        .collect::<Vec<String>>())),
                )
                .select((
                    schema::transactions::dsl::converted_into_receipt_id,
                    schema::transactions::dsl::transaction_hash,
                ))
                .load_async::<(String, crate::ParentTransactionHashString)>(pool),
            10,
            "Parent Transaction for ExecutionOutcome were fetched".to_string(),
            &receipts
        )
        .unwrap_or_default();

        let found_hashes_len = tx_hashes_for_receipt_via_transactions.len();
        tx_hashes_for_receipts.extend(tx_hashes_for_receipt_via_transactions.into_iter().map(
            |(receipt_id_string, transaction_hash_string)| {
                (
                    crate::ReceiptOrDataId::ReceiptId(
                        near_primitives::hash::CryptoHash::from_str(&receipt_id_string)
                            .expect("Failed to convert String to CryptoHash"),
                    ),
                    transaction_hash_string,
                )
            },
        ));

        if found_hashes_len == receipts.len() {
            break;
        }

        receipts.retain(|r| {
            !tx_hashes_for_receipts.contains_key(&crate::ReceiptOrDataId::ReceiptId(r.receipt_id))
        });

        if !strict_mode {
            if retries_left > 0 {
                retries_left -= 1;
            } else {
                break;
            }
        }
        warn!(
            target: crate::INDEXER_FOR_EXPLORER,
            "Going to retry to find parent transactions for receipts in {} milliseconds... \n {:#?}\n block hash {} \nchunk hash {}",
            find_tx_retry_interval.as_millis(),
            &receipts,
            block_hash,
            chunk_hash
        );
        tokio::time::sleep(find_tx_retry_interval).await;
        if find_tx_retry_interval < crate::MAX_DELAY_TIME {
            find_tx_retry_interval *= 2;
        }
    }

    Ok(tx_hashes_for_receipts)
}

async fn save_receipts(
    pool: &actix_diesel::Database<PgConnection>,
    receipts: Vec<models::Receipt>,
) -> anyhow::Result<()> {
    crate::await_retry_or_panic!(
        diesel::insert_into(schema::receipts::table)
            .values(receipts.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "Receipts were stored in database".to_string(),
        &receipts
    );
    Ok(())
}

async fn store_receipt_actions(
    pool: &actix_diesel::Database<PgConnection>,
    receipts: Vec<&near_indexer::near_primitives::views::ReceiptView>,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let receipt_actions: Vec<models::ActionReceipt> = receipts
        .iter()
        .filter_map(|receipt| models::ActionReceipt::try_from(*receipt).ok())
        .collect();

    let receipt_action_actions: Vec<models::ActionReceiptAction> = receipts
        .iter()
        .filter_map(|receipt| {
            if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                actions, ..
            } = &receipt.receipt
            {
                Some(actions.iter().enumerate().map(move |(index, action)| {
                    models::ActionReceiptAction::from_action_view(
                        receipt.receipt_id.to_string(),
                        i32::from_usize(index).expect("We expect usize to not overflow i32 here"),
                        action,
                        receipt.predecessor_id.to_string(),
                        receipt.receiver_id.to_string(),
                        block_timestamp,
                    )
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let receipt_action_input_data: Vec<models::ActionReceiptInputData> = receipts
        .iter()
        .filter_map(|receipt| {
            if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                input_data_ids,
                ..
            } = &receipt.receipt
            {
                Some(input_data_ids.iter().map(move |data_id| {
                    models::ActionReceiptInputData::from_data_id(
                        receipt.receipt_id.to_string(),
                        data_id.to_string(),
                    )
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let receipt_action_output_data: Vec<models::ActionReceiptOutputData> = receipts
        .iter()
        .filter_map(|receipt| {
            if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                output_data_receivers,
                ..
            } = &receipt.receipt
            {
                Some(output_data_receivers.iter().map(move |receiver| {
                    models::ActionReceiptOutputData::from_data_receiver(
                        receipt.receipt_id.to_string(),
                        receiver,
                    )
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::action_receipts::table)
            .values(receipt_actions.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "ReceiptActions were stored in database".to_string(),
        &receipt_actions
    );

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::action_receipt_actions::table)
            .values(receipt_action_actions.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "ReceiptActionActions were stored in database".to_string(),
        &receipt_action_actions
    );

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::action_receipt_output_data::table)
            .values(receipt_action_output_data.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "ReceiptActionOutputData were stored in database".to_string(),
        &receipt_action_output_data
    );

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::action_receipt_input_data::table)
            .values(receipt_action_input_data.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "ReceiptActionInputData were stored in database".to_string(),
        &receipt_action_input_data
    );

    Ok(())
}

async fn store_receipt_data(
    pool: &actix_diesel::Database<PgConnection>,
    receipts: Vec<&near_indexer::near_primitives::views::ReceiptView>,
) -> anyhow::Result<()> {
    let receipt_data_models: Vec<models::DataReceipt> = receipts
        .iter()
        .filter_map(|receipt| models::DataReceipt::try_from(*receipt).ok())
        .collect();

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::data_receipts::table)
            .values(receipt_data_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "ReceiptData were stored in database".to_string(),
        &receipt_data_models
    );

    Ok(())
}
