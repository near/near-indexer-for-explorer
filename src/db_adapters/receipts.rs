use std::collections::HashMap;
use std::convert::TryFrom;

use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::pg::expression::array_comparison::any;
use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl};
use futures::try_join;
use num_traits::cast::FromPrimitive;
use tracing::{error, warn};

use crate::models;
use crate::schema;

/// Saves receipts to database
pub(crate) async fn store_receipts(
    pool: &actix_diesel::Database<PgConnection>,
    receipts: &[near_indexer::near_primitives::views::ReceiptView],
    block_hash: &str,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
    strict_mode: bool,
) -> anyhow::Result<()> {
    if receipts.is_empty() {
        return Ok(());
    }
    let mut skipping_receipt_ids =
        std::collections::HashSet::<near_indexer::near_primitives::hash::CryptoHash>::new();

    let tx_hashes_for_receipts =
        find_tx_hashes_for_receipts(pool, receipts.to_vec(), strict_mode, block_hash, chunk_hash)
            .await?;
    let receipt_models: Vec<models::receipts::Receipt> = receipts
        .iter()
        .enumerate()
        .filter_map(|(index, r)| {
            if let Some(transaction_hash) =
                tx_hashes_for_receipts.get(r.receipt_id.to_string().as_str())
            {
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
    block_hash: &str,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
) -> anyhow::Result<HashMap<String, String>> {
    let mut tx_hashes_for_receipts: HashMap<String, String> = HashMap::new();

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
            let tx_hashes_for_data_id_via_data_output: Vec<(String, String)> = loop {
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
                        break res;
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
                HashMap::<String, String>::new();
            tx_hashes_for_data_id_via_data_output_hashmap
                .extend(tx_hashes_for_data_id_via_data_output);
            let tx_hashes_for_receipts_via_data_output: Vec<(String, String)> = receipts
                .iter()
                .filter_map(|r| match r.receipt {
                    near_indexer::near_primitives::views::ReceiptEnumView::Data {
                        data_id, ..
                    } => tx_hashes_for_data_id_via_data_output_hashmap
                        .get(data_id.to_string().as_str())
                        .map(|tx_hash| (r.receipt_id.to_string(), tx_hash.to_string())),
                    _ => None,
                })
                .collect();

            let found_hashes_len = tx_hashes_for_receipts_via_data_output.len();
            tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_data_output);

            if found_hashes_len == receipts.len() {
                break;
            }

            receipts.retain(|r| {
                !tx_hashes_for_receipts.contains_key(r.receipt_id.to_string().as_str())
            });
        }

        let tx_hashes_for_receipts_via_outcomes: Vec<(String, String)> =
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
                    .load_async::<(String, String)>(pool),
                10,
                "Parent Transaction for Receipts were fetched".to_string(),
                &receipts
            )
            .unwrap_or_default();

        let found_hashes_len = tx_hashes_for_receipts_via_outcomes.len();
        tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_outcomes);

        if found_hashes_len == receipts.len() {
            break;
        }

        receipts
            .retain(|r| !tx_hashes_for_receipts.contains_key(r.receipt_id.to_string().as_str()));

        let tx_hashes_for_receipt_via_transactions: Vec<(String, String)> =
            crate::await_retry_or_panic!(
                schema::transactions::table
                    .filter(
                        schema::transactions::dsl::converted_into_receipt_id.eq(any(
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
                        schema::transactions::dsl::converted_into_receipt_id,
                        schema::transactions::dsl::transaction_hash,
                    ))
                    .load_async::<(String, String)>(pool),
                10,
                "Parent Transaction for ExecutionOutcome were fetched".to_string(),
                &receipts
            )
            .unwrap_or_default();

        let found_hashes_len = tx_hashes_for_receipt_via_transactions.len();
        tx_hashes_for_receipts.extend(tx_hashes_for_receipt_via_transactions);

        if found_hashes_len == receipts.len() {
            break;
        }

        receipts
            .retain(|r| !tx_hashes_for_receipts.contains_key(r.receipt_id.to_string().as_str()));

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
