use num_traits::cast::FromPrimitive;

use actix;
use bigdecimal::BigDecimal;
use clap::derive::Clap;
use diesel::{
    prelude::*,
    dsl,
};
#[macro_use]
extern crate diesel;
use tokio::sync::mpsc;
use tokio_diesel::*;
use tracing::info;
use tracing_subscriber::EnvFilter;

use near_indexer;

use crate::configs::{Opts, SubCommand};

mod configs;
mod models;
mod schema;

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::BlockResponse>) {
    let pool = models::establish_connection();

    while let Some(block) = stream.recv().await {
        // TODO: handle data as you need
        // Block
        info!(target: "indexer_for_explorer", "Block height {}", &block.block.header.height);
        match diesel::insert_into(schema::blocks::table)
            .values(models::Block::from_block_view(&block.block))
            .execute_async(&pool)
            .await
        {
            Ok(_) => {},
            Err(_) => continue,
        };

        // Chunks
        match diesel::insert_into(schema::chunks::table)
            .values(
                block
                    .chunks
                    .iter()
                    .map(|chunk| models::Chunk::from_chunk_view(block.block.header.height, chunk))
                    .collect::<Vec<models::Chunk>>(),
            )
            .execute_async(&pool)
            .await
        {
            Ok(_) => {},
            Err(_) => { eprintln!("Unable to save chunk, skipping"); },
        };

        // Outcomes from previous block
        for outcome in &block.outcomes {
            match outcome {
                near_indexer::Outcome::Transaction(execution_outcome) => {
                    diesel::update(schema::transactions::table.filter(
                        schema::transactions::hash.eq(execution_outcome.id.to_string())
                    ))
                    .set((
                        schema::transactions::dsl::receipt_id.eq(execution_outcome.outcome.receipt_ids.first().unwrap().to_string()),
                        schema::transactions::dsl::receipt_conversion_gas_burnt.eq(
                            Some(BigDecimal::from_u64(execution_outcome.outcome.gas_burnt).unwrap_or(0u64.into()))
                        ),
                        schema::transactions::dsl::receipt_conversion_tokens_burnt.eq(
                            Some(BigDecimal::from_u128(execution_outcome.outcome.tokens_burnt).unwrap_or(0u64.into()))
                        ),
                    ))
                    .execute_async(&pool)
                    .await
                    .unwrap();
                },
                near_indexer::Outcome::Receipt(execution_outcome) => {
                    diesel::update(schema::receipts::table.filter(
                        schema::receipts::receipt_id.eq(execution_outcome.id.to_string())
                    ))
                    .set((
                        schema::receipts::dsl::status.eq(
                            match execution_outcome.outcome.status {
                                near_indexer::near_primitives::views::ExecutionStatusView::Unknown => "unknown".to_string(),
                                near_indexer::near_primitives::views::ExecutionStatusView::Failure(_) => "failure".to_string(),
                                near_indexer::near_primitives::views::ExecutionStatusView::SuccessValue(_) => "success_value".to_string(),
                                near_indexer::near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => "success_receipt_id".to_string(),
                            }
                        ),
                    ))
                    .execute_async(&pool)
                    .await
                    .unwrap();
                }
            }
        }

        // Transactions
        diesel::insert_into(schema::transactions::table)
            .values(
                block
                    .chunks
                    .iter()
                    .map(|chunk| {
                        chunk
                            .transactions
                            .iter()
                            .map(|transaction| {
                                models::Transaction::from_transaction_view(
                                    block.block.header.height,
                                    block.block.header.timestamp,
                                    transaction,
                                )
                            })
                            .collect::<Vec<models::Transaction>>()
                    })
                    .flatten()
                    .collect::<Vec<models::Transaction>>(),
            )
            .execute_async(&pool)
            .await
            .unwrap();

        // TODO handle block.outcomes for transactions

        // Receipts
        for chunk in block.chunks {
            for receipt in chunk.receipts {
                // Save receipt
                // Check if Receipt with given `receipt_id` is already in DB
                // Receipt might be created with one of the previous receipts
                // based on `outcome.receipt_ids` (read more below)
                let receipt_id = receipt.receipt_id.clone().to_string();
                let receipt_exists = dsl::select(
                    dsl::exists(
                        schema::receipts::table.filter(
                            schema::receipts::receipt_id.eq(receipt_id.clone())
                        )
                    )
                )
                .get_result_async(&pool)
                .await
                .unwrap();

                if receipt_exists {
                    // Update previously created receipt with data
                    let receipt_changeset = models::Receipt::from_receipt(&receipt);
                    // .set(receipt_changeset)
                    diesel::update(
                        schema::receipts::table.filter(
                            schema::receipts::receipt_id.eq(receipt_id)
                        )
                    )
                        .set((
                            schema::receipts::dsl::predecessor_id.eq(receipt_changeset.predecessor_id),
                            schema::receipts::dsl::receiver_id.eq(receipt_changeset.receipt_id),
                            schema::receipts::dsl::status.eq(receipt_changeset.status),
                            schema::receipts::dsl::type_.eq(receipt_changeset.type_),
                        ))
                        .execute_async(&pool)
                        .await
                        .unwrap();
                } else {
                    // Create new receipt with fulfilled data
                    diesel::insert_into(schema::receipts::table)
                        .values(
                            models::Receipt::from_receipt(&receipt)
                        )
                        .execute_async(&pool)
                        .await
                        .unwrap();
                }

                // ReceiptData or ReceiptActions
                match &receipt.receipt {
                    ref _data @ near_indexer::near_primitives::views::ReceiptEnumView::Data { .. } => {
                        let receipt_data = models::ReceiptData::from_receipt(&receipt);
                        if let Ok(data) = receipt_data {
                            diesel::insert_into(schema::receipt_data::table)
                                .values(data)
                                .execute_async(&pool)
                                .await
                                .unwrap();
                        }
                    },
                    near_indexer::near_primitives::views::ReceiptEnumView::Action {
                            signer_id: _,
                            signer_public_key: _,
                            gas_price: _,
                            output_data_receivers,
                            input_data_ids,
                            actions
                        } => {
                            let receipt_action = models::ReceiptAction::from_receipt(&receipt);
                            if let Ok(receipt_action_) = receipt_action {
                                diesel::insert_into(schema::receipt_action::table)
                                    .values(receipt_action_)
                                    .execute_async(&pool)
                                    .await
                                    .unwrap();

                                // Input and output data
                                diesel::insert_into(schema::actions_output_data::table)
                                    .values(
                                        output_data_receivers
                                            .iter()
                                            .map(|data_receiver| models::ReceiptActionOutputData::from_data_receiver(
                                                    receipt.receipt_id.to_string().clone(),
                                                    data_receiver,
                                                ))
                                            .collect::<Vec<models::ReceiptActionOutputData>>()
                                    )
                                    .execute_async(&pool)
                                    .await
                                    .unwrap();

                                diesel::insert_into(schema::actions_input_data::table)
                                    .values(
                                        input_data_ids
                                            .iter()
                                            .map(|data_id| models::ReceiptActionInputData::from_data_id(
                                                receipt.receipt_id.to_string().clone(),
                                                data_id.to_string(),
                                            ))
                                            .collect::<Vec<models::ReceiptActionInputData>>()

                                    )
                                    .execute_async(&pool)
                                    .await
                                    .unwrap();
                            }


                            for (i, action) in actions.iter().enumerate() {
                                diesel::insert_into(schema::actions::table)
                                    .values(
                                        models::Action::from_action(
                                            receipt.receipt_id.to_string().clone(),
                                            i as i32,
                                            action,
                                        )
                                    )
                                    .execute_async(&pool)
                                    .await
                                    .unwrap();

                                // Accounts & AccessKeys
                                match action {
                                    near_indexer::near_primitives::views::ActionView::CreateAccount => {
                                        match diesel::insert_into(schema::accounts::table)
                                            .values(
                                                models::Account::new(
                                                    receipt.receiver_id.to_string().clone(),
                                                    i as i32,
                                                    receipt.receipt_id.to_string().clone(),
                                                    BigDecimal::from_u64(block.block.header.timestamp).unwrap_or(0.into()),
                                                )
                                            )
                                            .execute_async(&pool)
                                            .await
                                            {
                                                _ => {}
                                            };
                                    },
                                    near_indexer::near_primitives::views::ActionView::AddKey {
                                        public_key,
                                        access_key,
                                    } => {
                                        match diesel::insert_into(schema::access_keys::table)
                                            .values(
                                                models::AccessKey::new(
                                                    receipt.receiver_id.to_string().clone(),
                                                    public_key.to_string(),
                                                    access_key,
                                                )
                                            )
                                            .execute_async(&pool)
                                            .await
                                            {
                                                _ => {}
                                            };
                                    },
                                    _ => {},
                                };
                            }
                    }
                }

            }
        }

    }
}


fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();

    let env_filter = EnvFilter::new(
        "tokio_reactor=info,near=info,near=error,stats=info,telemetry=info,indexer_for_explorer=info",
    );
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    let opts: Opts = Opts::parse();

    let home_dir = opts
        .home_dir
        .unwrap_or_else(|| std::path::PathBuf::from(near_indexer::get_default_home()));

    match opts.subcmd {
        SubCommand::Run => {
            let indexer = near_indexer::Indexer::new(Some(&home_dir));
            let stream = indexer.streamer();
            actix::spawn(listen_blocks(stream));
            indexer.start();
        }
        SubCommand::Init(config) => near_indexer::init_configs(
            &home_dir,
            config.chain_id.as_ref().map(AsRef::as_ref),
            config.account_id.as_ref().map(AsRef::as_ref),
            config.test_seed.as_ref().map(AsRef::as_ref),
            config.num_shards,
            config.fast,
            config.genesis.as_ref().map(AsRef::as_ref),
            config.download,
            config.download_genesis_url.as_ref().map(AsRef::as_ref),
        )
    }
}
