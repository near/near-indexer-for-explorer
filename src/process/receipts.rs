use std::collections::HashMap;
use std::convert::TryFrom;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{PgConnection, QueryDsl, ExpressionMethods};
use futures::join;
use num_traits::cast::FromPrimitive;
use tokio_diesel::{AsyncRunQueryDsl};
use tracing::{error, warn};

use crate::models;
use crate::schema;

/// Saves receipts to database
pub(crate) async fn process_receipts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: Vec<&near_indexer::near_primitives::views::ReceiptView>,
    block_height: u64,
) {
    let mut skipping_receipt_ids: Vec<near_indexer::near_primitives::hash::CryptoHash> = vec![];

    let tx_hashes_for_receipts_via_outcomes: Result<Vec<(String, String)>, tokio_diesel::AsyncError> = schema::execution_outcome_receipts::table
        .inner_join(schema::receipts::table)
        .filter(
            schema::execution_outcome_receipts::dsl::receipt_id.eq_any(receipts.iter().map(|r| r.receipt_id.to_string()).collect::<Vec<_>>())
        )
        .select((schema::execution_outcome_receipts::dsl::receipt_id, schema::receipts::dsl::transaction_hash))
        .load_async(&pool)
        .await;

    let tx_hashes_for_receipt_via_transactions: Result<Vec<(String, String)>, tokio_diesel::AsyncError> = schema::transactions::table
        .filter(
            schema::transactions::dsl::receipt_id.eq_any(receipts.iter().map(|r| r.receipt_id.to_string()).collect::<Vec<_>>())
        )
        .select((schema::transactions::dsl::receipt_id, schema::transactions::dsl::transaction_hash))
        .load_async(&pool)
        .await;

    let mut tx_hashes_for_receipts: HashMap<String, String> = HashMap::new();

    match tx_hashes_for_receipts_via_outcomes {
        Ok(result) => tx_hashes_for_receipts.extend(result),
        Err(_) => {}, // TODO: handle error
    };
    match tx_hashes_for_receipt_via_transactions {
        Ok(result) => tx_hashes_for_receipts.extend(result),
        Err(_) => {}, // TODO: handle error
    };

    let receipt_models: Vec<models::receipts::Receipt> = receipts
        .iter()
        .filter_map(|r| {
            if tx_hashes_for_receipts.contains_key(r.receipt_id.to_string().as_str()) {
                Some(models::Receipt::from_receipt_view(
                    r,
                    block_height,
                    tx_hashes_for_receipts
                        .get(r.receipt_id.to_string().as_str())
                        .expect("transaction hash expected as we have check for it earlier")
                        .clone()
                ))
            } else {
                warn!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Skipping Receipt {} as we can't find parent Transaction for it.",
                    r.receipt_id.to_string()
                );
                skipping_receipt_ids.push(r.receipt_id.clone());
                None
            }
        })
        .collect();

    // for receipt in &receipts {
    //     eprintln!("RECEIPT ID {}", receipt.receipt_id.to_string());
    //     let mut transaction_hash: Option<String> = None;
    //     let tx_lookup_through_outcomes: Result<Vec<String>, tokio_diesel::AsyncError> = schema::execution_outcome_receipts::table
    //         .inner_join(schema::receipts::table)
    //         .filter(
    //             schema::execution_outcome_receipts::dsl::receipt_id.eq(receipt.receipt_id.to_string())
    //         )
    //         .select(schema::receipts::dsl::transaction_hash)
    //         .load_async(&pool)
    //         .await;
    //
    //     match tx_lookup_through_outcomes {
    //         Ok(result) => {
    //             eprintln!("{:#?}", result);
    //             if !result.is_empty() {
    //                 transaction_hash = Some(result.get(0).unwrap().clone());
    //             }
    //         },
    //         Err(_) => {},
    //     }
    //
    //     if transaction_hash.is_none() {
    //         let tx_lookup_through_transactions: Result<Vec<String>, tokio_diesel::AsyncError> = schema::transactions::table
    //                 .filter(
    //                     schema::transactions::dsl::receipt_id.eq(receipt.receipt_id.to_string())
    //                 )
    //                 .select(schema::transactions::transaction_hash)
    //                 .load_async(&pool)
    //                 .await;
    //         match tx_lookup_through_transactions {
    //             Ok(result) => {
    //                 transaction_hash = Some(result.get(0).expect("At least one element is expected").clone());
    //             },
    //             Err(_) => {},
    //         }
    //     }
    //
    //     receipt_models.push(models::Receipt::from_receipt_view(receipt, block_height, transaction_hash.expect("`transaction_hash` expected here.")));
    // }
    let save_receipts_future = save_receipts(
        &pool,
        receipt_models
    );

    let receipts_to_process: Vec<&near_indexer::near_primitives::views::ReceiptView> = receipts
        .into_iter()
        .filter(|r| !skipping_receipt_ids.contains(&r.receipt_id))
        .collect();

    let process_receipt_actions_future = process_receipt_actions(&pool, &receipts_to_process);

    let process_receipt_data_future = process_receipt_data(&pool, &receipts_to_process);

    join!(
        save_receipts_future,
        process_receipt_actions_future,
        process_receipt_data_future
    );
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
                    "Error occurred while Receipt were adding to database. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }
}

async fn process_receipt_actions(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: &[&near_indexer::near_primitives::views::ReceiptView],
) {
    let mut receipt_actions: Vec<models::ReceiptAction> = vec![];
    let mut receipt_action_actions: Vec<models::ReceiptActionAction> = vec![];
    let mut receipt_action_input_data: Vec<models::ReceiptActionInputData> = vec![];
    let mut receipt_action_output_data: Vec<models::ReceiptActionOutputData> = vec![];

    for receipt in receipts {
        if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
            actions,
            input_data_ids,
            output_data_receivers,
            ..
        } = &receipt.receipt
        {
            match models::ReceiptAction::try_from(*receipt) {
                Ok(model) => receipt_actions.push(model),
                Err(_) => continue,
            }

            for (index, action) in actions.iter().enumerate() {
                receipt_action_input_data.extend(input_data_ids.iter().map(|data_id| {
                    models::ReceiptActionInputData::from_data_id(
                        receipt.receipt_id.to_string(),
                        data_id.to_string(),
                    )
                }));
                receipt_action_output_data.extend(output_data_receivers.iter().map(|receiver| {
                    models::ReceiptActionOutputData::from_data_receiver(
                        receipt.receipt_id.to_string(),
                        receiver,
                    )
                }));
                receipt_action_actions.push(models::ReceiptActionAction::from_action_view(
                    receipt.receipt_id.to_string(),
                    i32::from_usize(index).unwrap(),
                    action,
                ));
            }
        }
    }

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
                    "Error occurred while ReceiptActions were saving. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error
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
                    "Error occurred while ReceiptActionActions were saving. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error
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
                    "Error occurred while ReceiptActionOutputData were saving. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error
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
                    "Error occurred while ReceiptActionInputData were saving. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }
}

async fn process_receipt_data(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: &[&near_indexer::near_primitives::views::ReceiptView],
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
                    "Error occurred while ReceiptData were saving. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        };
    }
}
