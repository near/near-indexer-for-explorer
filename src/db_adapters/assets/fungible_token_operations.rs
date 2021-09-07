use std::str::FromStr;

use actix_diesel::dsl::AsyncRunQueryDsl;
use anyhow::Context;
use bigdecimal::BigDecimal;
use diesel::PgConnection;
use tracing::error;

use near_indexer::near_primitives::hash::CryptoHash;
use near_indexer::near_primitives::views::{ActionView, ExecutionStatusView, ReceiptEnumView};

use crate::models;
use crate::schema;

pub(crate) async fn store_ft(
    pool: &actix_diesel::Database<PgConnection>,
    streamer_message: &near_indexer::StreamerMessage,
) {
    for shard in &streamer_message.shards {
        collect_and_store_ft_operations(
            &pool,
            &shard.receipt_execution_outcomes,
            &streamer_message.block.header.timestamp,
        )
        .await;
    }
}

async fn collect_and_store_ft_operations(
    pool: &actix_diesel::Database<PgConnection>,
    execution_outcomes: &[near_indexer::IndexerExecutionOutcomeWithReceipt],
    block_timestamp: &u64,
) {
    for outcome in execution_outcomes {
        // TODO it's dirty, rewrite this method
        let mut interval = crate::INTERVAL;
        let ft_operations: Vec<models::assets::fungible_token_operations::FungibleTokenOperation>;
        loop {
            ft_operations = match collect_ft_operations(outcome, block_timestamp) {
                Ok(value) => value,
                Err(error) => {
                    error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while collecting FungibleTokenOperation. Retrying in {} milliseconds... \n {:#?}",
                    interval.as_millis(),
                    error,
                );
                    tokio::time::sleep(interval).await;
                    if interval < crate::MAX_DELAY_TIME {
                        interval *= 2;
                    }
                    continue;
                }
            };
            break;
        }

        interval = crate::INTERVAL;
        loop {
            match diesel::insert_into(schema::assets__fungible_token_operations::table)
                .values(ft_operations.clone())
                .on_conflict_do_nothing()
                .execute_async(&pool)
                .await
            {
                Ok(_) => break,
                Err(async_error) => {
                    error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while FungibleTokenOperation were adding to database. Retrying in {} milliseconds... \n {:#?} \n{:#?}",
                    interval.as_millis(),
                    async_error,
                    &ft_operations,
                );
                    tokio::time::sleep(interval).await;
                    if interval < crate::MAX_DELAY_TIME {
                        interval *= 2;
                    }
                }
            }
        }
    }
}

fn collect_ft_operations(
    outcome: &near_indexer::IndexerExecutionOutcomeWithReceipt,
    block_timestamp: &u64,
) -> anyhow::Result<Vec<models::assets::fungible_token_operations::FungibleTokenOperation>> {
    match &outcome.receipt.receipt {
        ReceiptEnumView::Action { actions, .. } => actions
            .iter()
            .filter_map(|action| match action {
                ActionView::FunctionCall {
                    method_name, args, ..
                } if should_handle_function(method_name) => Some((method_name, args)),
                _ => None,
            })
            .map(|(method_name, args)| {
                let args_decoded = base64::decode(args).with_context(|| {
                    format!(
                        "Unable to decode function call arguments for receipt {}",
                        outcome.receipt.receipt_id
                    )
                })?;
                let args_json: serde_json::Value = serde_json::from_slice(&args_decoded)
                    .with_context(|| {
                        format!(
                            "Function call arguments for receipt {} is not a valid JSON",
                            outcome.receipt.receipt_id
                        )
                    })?;

                match method_name.as_str() {
                    "ft_transfer" => {
                        handle_ft_transfer(outcome, block_timestamp, method_name, &args_json)
                    }
                    "ft_resolve_transfer" => handle_ft_resolve_transfer(
                        outcome,
                        block_timestamp,
                        method_name,
                        &args_json,
                    ),
                    &_ => anyhow::bail!(
                        "Function {} is not supported, check receipt {}",
                        method_name,
                        outcome.receipt.receipt_id
                    ),
                }
            })
            .collect(),
        _ => Ok(Vec::new()),
    }
}

fn should_handle_function(method_name: &str) -> bool {
    let not_informative_functions = vec![
        "ft_on_transfer",
        "ft_resolve_protocol_call", // https://explorer.near.org/transactions/EeRXHaZxm2NLeowkajeC15qgJESVfWewZoxPFdpv1kV4
        "ft_transfer_call",
    ];
    method_name.starts_with("ft_") && !not_informative_functions.contains(&method_name)
}

fn handle_ft_transfer(
    outcome: &near_indexer::IndexerExecutionOutcomeWithReceipt,
    block_timestamp: &u64,
    method_name: &str,
    args: &serde_json::Value,
) -> anyhow::Result<models::assets::fungible_token_operations::FungibleTokenOperation> {
    let receipt_id = &outcome.receipt.receipt_id;
    let receiver_id = get_function_call_parameter(receipt_id, method_name, args, "receiver_id")?;

    let amount = match &outcome.execution_outcome.outcome.status {
        ExecutionStatusView::SuccessValue(_) => {
            get_function_call_parameter(receipt_id, method_name, args, "amount")?
        }
        _ => "0".to_string(),
    };

    Ok(
        models::assets::fungible_token_operations::FungibleTokenOperation {
            processed_in_block_timestamp: BigDecimal::from(*block_timestamp),
            processed_in_receipt_id: receipt_id.to_string(),
            ft_contract_account_id: outcome.receipt.receiver_id.to_string(),
            ft_sender_account_id: outcome.receipt.predecessor_id.to_string(),
            ft_receiver_account_id: receiver_id,
            called_method: method_name.to_string(),
            ft_amount: BigDecimal::from_str(&amount).context("`ft_amount` expected to be u128")?,
            args: args.clone(),
        },
    )
}

fn handle_ft_resolve_transfer(
    outcome: &near_indexer::IndexerExecutionOutcomeWithReceipt,
    block_timestamp: &u64,
    method_name: &str,
    args: &serde_json::Value,
) -> anyhow::Result<models::assets::fungible_token_operations::FungibleTokenOperation> {
    let receipt_id = &outcome.receipt.receipt_id;
    let sender_id = get_function_call_parameter(receipt_id, method_name, args, "sender_id")?;
    let receiver_id = get_function_call_parameter(receipt_id, method_name, args, "receiver_id")?;

    let amount: u128 = match &outcome.execution_outcome.outcome.status {
        ExecutionStatusView::SuccessValue(value) => {
            let value_decoded = base64::decode(value).with_context(|| {
                format!(
                    "Unable to decode execution outcome status for receipt {}",
                    receipt_id
                )
            })?;
            let value_stringify = std::str::from_utf8(&value_decoded).with_context(|| {
                format!("String representation of execution outcome status for receipt {} has non-UTF8 encoding", receipt_id)})?;
            let value_without_quotes = &value_stringify[1..value_stringify.len() - 1];
            value_without_quotes.parse::<u128>().with_context(|| {
                format!(
                    "Execution outcome status expected to be u128, receipt {}",
                    receipt_id
                )
            })?
        }
        _ => 0,
    };

    Ok(
        models::assets::fungible_token_operations::FungibleTokenOperation {
            processed_in_block_timestamp: BigDecimal::from(*block_timestamp),
            processed_in_receipt_id: receipt_id.to_string(),
            ft_contract_account_id: outcome.receipt.receiver_id.to_string(),
            ft_sender_account_id: sender_id,
            ft_receiver_account_id: receiver_id,
            called_method: method_name.to_string(),
            ft_amount: BigDecimal::from_str(&amount.to_string())
                .context("`ft_amount` expected to be u128")?,
            args: args.clone(),
        },
    )
}

fn get_function_call_parameter(
    receipt_id: &CryptoHash,
    method_name: &str,
    args: &serde_json::Value,
    field: &str,
) -> anyhow::Result<String> {
    Ok(args
        .get(field)
        .with_context(|| {
            format!(
                "Unable to get parameter `{}`, function {}, receipt {}, args {}",
                field, method_name, receipt_id, args
            )
        })?
        .as_str()
        .with_context(|| {
            format!(
                "Parameter `{}` has non-string type, function {}, receipt {}, args {}",
                field, method_name, receipt_id, args
            )
        })?
        .to_string())
}
