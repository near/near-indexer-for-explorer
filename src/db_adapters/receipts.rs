use std::collections::HashMap;
use std::convert::TryFrom;

use diesel::pg::expression::array_comparison::any;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl};
use futures::join;
use num_traits::cast::FromPrimitive;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::{error, warn};

use crate::models;
use crate::schema;

/// Saves receipts to database
pub(crate) async fn store_receipts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: Vec<&near_indexer::near_primitives::views::ReceiptView>,
    block_hash: &str,
    strict_mode: bool,
) {
    let mut skipping_receipt_ids =
        std::collections::HashSet::<near_indexer::near_primitives::hash::CryptoHash>::new();

    let tx_hashes_for_receipts = find_tx_hashes_for_receipts(
        &pool,
        receipts.iter().map(|r| r.receipt_id.to_string()).collect(),
        strict_mode,
    )
    .await;

    let receipt_models: Vec<models::receipts::Receipt> = receipts
        .iter()
        .filter_map(|r| {
            if let Some(transaction_hash) =
                tx_hashes_for_receipts.get(r.receipt_id.to_string().as_str())
            {
                Some(models::Receipt::from_receipt_view(
                    r,
                    block_hash,
                    transaction_hash,
                ))
            } else {
                warn!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Skipping Receipt {} as we can't find parent Transaction for it.",
                    r.receipt_id.to_string()
                );
                skipping_receipt_ids.insert(r.receipt_id);
                None
            }
        })
        .collect();

    save_receipts(&pool, receipt_models).await;

    let (action_receipts, data_receipts): (Vec<&near_indexer::near_primitives::views::ReceiptView>, Vec<&near_indexer::near_primitives::views::ReceiptView>) = receipts
        .into_iter()
        .filter(|r| !skipping_receipt_ids.contains(&r.receipt_id))
        .partition(|receipt| matches!(receipt.receipt, near_indexer::near_primitives::views::ReceiptEnumView::Action { .. }));

    let process_receipt_actions_future = store_receipt_actions(&pool, action_receipts);

    let process_receipt_data_future = store_receipt_data(&pool, data_receipts);

    join!(process_receipt_actions_future, process_receipt_data_future);
}

/// Looks for already created parent transaction hash for given receipts
async fn find_tx_hashes_for_receipts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    mut receipt_ids: Vec<String>,
    strict_mode: bool,
) -> HashMap<String, String> {
    let mut tx_hashes_for_receipts: HashMap<String, String> = HashMap::new();

    let mut retries_left: u8 = 10; // retry at least times even in no-strict mode to avoid data loss
    loop {
        let tx_hashes_for_receipts_via_outcomes: Vec<(String, String)> = loop {
            match schema::execution_outcome_receipts::table
                .inner_join(
                    schema::receipts::table.on(
                        schema::execution_outcome_receipts::dsl::execution_outcome_receipt_id
                            .eq(schema::receipts::dsl::receipt_id),
                    ),
                )
                .filter(
                    schema::execution_outcome_receipts::dsl::receipt_id
                        .eq(any(receipt_ids.clone())),
                )
                .select((
                    schema::execution_outcome_receipts::dsl::receipt_id,
                    schema::receipts::dsl::transaction_hash,
                ))
                .load_async(&pool)
                .await
            {
                Ok(res) => {
                    break res;
                }
                Err(async_error) => {
                    error!(
                        target: crate::INDEXER_FOR_EXPLORER,
                        "Error occurred while fetching the parent receipt for Receipt. Retrying in {} milliseconds... \n {:#?}",
                        crate::INTERVAL.as_millis(),
                        async_error,
                    );
                    tokio::time::delay_for(crate::INTERVAL).await;
                }
            }
        };

        let found_hashes_len = tx_hashes_for_receipts_via_outcomes.len();
        tx_hashes_for_receipts.extend(tx_hashes_for_receipts_via_outcomes);

        if found_hashes_len == receipt_ids.len() {
            break;
        }

        receipt_ids.retain(|r| tx_hashes_for_receipts.contains_key(r.as_str()));

        let tx_hashes_for_receipt_via_transactions: Vec<(String, String)> = loop {
            match schema::transactions::table
                .filter(schema::transactions::dsl::receipt_id.eq(any(receipt_ids.clone())))
                .select((
                    schema::transactions::dsl::receipt_id,
                    schema::transactions::dsl::transaction_hash,
                ))
                .load_async(&pool)
                .await
            {
                Ok(res) => {
                    break res;
                }
                Err(async_error) => {
                    error!(
                        target: crate::INDEXER_FOR_EXPLORER,
                        "Error occurred while fetching the parent transaction for ExecutionOutcome. Retrying in {} milliseconds... \n {:#?}",
                        crate::INTERVAL.as_millis(),
                        async_error,
                    );
                    tokio::time::delay_for(crate::INTERVAL).await;
                }
            }
        };

        let found_hashes_len = tx_hashes_for_receipt_via_transactions.len();
        tx_hashes_for_receipts.extend(tx_hashes_for_receipt_via_transactions);

        if found_hashes_len == receipt_ids.len() {
            break;
        }

        receipt_ids.retain(|r| tx_hashes_for_receipts.contains_key(r.as_str()));

        if !strict_mode {
            if retries_left > 0 {
                retries_left -= 1;
                warn!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Going to retry to find parent transactions for receipts in {} milliseconds...",
                    crate::INTERVAL.as_millis()
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            } else {
                break;
            }
        } else {
            warn!(
                target: crate::INDEXER_FOR_EXPLORER,
                "Going to retry to find parent transactions for receipts in {} milliseconds...",
                crate::INTERVAL.as_millis()
            );
            tokio::time::delay_for(crate::INTERVAL).await;
            continue;
        }
    }

    tx_hashes_for_receipts
}

async fn save_receipts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: Vec<models::Receipt>,
) {
    loop {
        match diesel::insert_into(schema::receipts::table)
            .values(receipts.clone())
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while Receipt were adding to database. Retrying in {} milliseconds... \n {:#?} \n{:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    receipts,
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }
}

async fn store_receipt_actions(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: Vec<&near_indexer::near_primitives::views::ReceiptView>,
) {
    let receipt_actions: Vec<models::ReceiptAction> = receipts
        .iter()
        .filter_map(|receipt| models::ReceiptAction::try_from(*receipt).ok())
        .collect();

    let receipt_action_actions: Vec<models::ReceiptActionAction> = receipts
        .iter()
        .filter_map(|receipt| {
            if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                actions, ..
            } = &receipt.receipt
            {
                Some(actions.iter().enumerate().map(move |(index, action)| {
                    models::ReceiptActionAction::from_action_view(
                        receipt.receipt_id.to_string(),
                        i32::from_usize(index).expect("We expect usize to not overflow i32 here"),
                        action,
                    )
                }))
            } else {
                None
            }
        })
        .flatten()
        .collect();

    let receipt_action_input_data: Vec<models::ReceiptActionInputData> = receipts
        .iter()
        .filter_map(|receipt| {
            if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                input_data_ids,
                ..
            } = &receipt.receipt
            {
                Some(input_data_ids.iter().map(move |data_id| {
                    models::ReceiptActionInputData::from_data_id(
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

    let receipt_action_output_data: Vec<models::ReceiptActionOutputData> = receipts
        .iter()
        .filter_map(|receipt| {
            if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                output_data_receivers,
                ..
            } = &receipt.receipt
            {
                Some(output_data_receivers.iter().map(move |receiver| {
                    models::ReceiptActionOutputData::from_data_receiver(
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

    loop {
        match diesel::insert_into(schema::receipt_actions::table)
            .values(receipt_actions.clone())
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ReceiptActions were saving. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &receipt_actions,
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }

    loop {
        match diesel::insert_into(schema::receipt_action_actions::table)
            .values(receipt_action_actions.clone())
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ReceiptActionActions were saving. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &receipt_action_actions
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }

    loop {
        match diesel::insert_into(schema::receipt_action_output_data::table)
            .values(receipt_action_output_data.clone())
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ReceiptActionOutputData were saving. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &receipt_action_output_data
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }

    loop {
        match diesel::insert_into(schema::receipt_action_input_data::table)
            .values(receipt_action_input_data.clone())
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ReceiptActionInputData were saving. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &receipt_action_input_data
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }
}

async fn store_receipt_data(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: Vec<&near_indexer::near_primitives::views::ReceiptView>,
) {
    let receipt_data_models: Vec<models::ReceiptData> = receipts
        .iter()
        .filter_map(|receipt| models::ReceiptData::try_from(*receipt).ok())
        .collect();

    loop {
        match diesel::insert_into(schema::receipt_data::table)
            .values(receipt_data_models.clone())
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ReceiptData were saving. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &receipt_data_models
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }
}
