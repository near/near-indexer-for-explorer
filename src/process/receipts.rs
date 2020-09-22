use std::convert::TryFrom;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use futures::join;
use num_traits::cast::FromPrimitive;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use crate::models;
use crate::schema;

/// Saves receipts to database
pub(crate) async fn process_receipts(
    pool: &Pool<ConnectionManager<PgConnection>>,
    receipts: Vec<&near_indexer::near_primitives::views::ReceiptView>,
    block_height: u64,
) {
    let save_receipts_future = save_receipts(
        &pool,
        receipts
            .iter()
            .map(|receipt| models::Receipt::from_receipt_view(receipt, block_height))
            .collect::<Vec<models::Receipt>>(),
    );

    let process_receipt_actions_future = process_receipt_actions(&pool, &receipts);

    let process_receipt_data_future = process_receipt_data(&pool, &receipts);

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
        if matches!(receipt.receipt, near_indexer::near_primitives::views::ReceiptEnumView::Action { .. })
        {
            match models::ReceiptAction::try_from(*receipt) {
                Ok(model) => receipt_actions.push(model),
                Err(_) => continue,
            }

            if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
                actions,
                input_data_ids,
                output_data_receivers,
                ..
            } = &receipt.receipt
            {
                for (index, action) in actions.iter().enumerate() {
                    receipt_action_input_data.extend(input_data_ids.iter().map(|data_id| {
                        models::ReceiptActionInputData::from_data_id(
                            receipt.receipt_id.as_ref().to_vec(),
                            data_id.as_ref().to_vec(),
                        )
                    }));
                    receipt_action_output_data.extend(output_data_receivers.iter().map(
                        |receiver| {
                            models::ReceiptActionOutputData::from_data_receiver(
                                receipt.receipt_id.as_ref().to_vec(),
                                receiver,
                            )
                        },
                    ));
                    receipt_action_actions.push(models::ReceiptActionAction::from_action_view(
                        receipt.receipt_id.as_ref().to_vec(),
                        i32::from_usize(index).unwrap(),
                        action,
                    ));
                }
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
        .filter_map(|receipt| match models::ReceiptData::try_from(*receipt) {
            Ok(model) => Some(model),
            Err(_) => None,
        })
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
