use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use crate::models;
use crate::schema;
use diesel::pg::expression::array_comparison::any;

pub(crate) async fn store_execution_outcomes(
    pool: &Pool<ConnectionManager<PgConnection>>,
    chunks: &[near_indexer::IndexerChunkView],
    block_timestamp: u64,
) {
    for chunk in chunks {
        store_execution_outcomes_for_chunk(
            &pool,
            &chunk.receipt_execution_outcomes,
            &chunk.header.chunk_hash,
            block_timestamp,
        )
        .await;
    }
}

/// Saves ExecutionOutcome to database and then saves ExecutionOutcomesReceipts
pub async fn store_execution_outcomes_for_chunk(
    pool: &Pool<ConnectionManager<PgConnection>>,
    execution_outcomes: &[near_indexer::IndexerExecutionOutcomeWithReceipt],
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
) {
    let mut interval = crate::INTERVAL;
    let known_receipt_ids: std::collections::HashSet<String> = loop {
        match schema::receipts::table
            .filter(
                schema::receipts::dsl::receipt_id.eq(any(execution_outcomes
                    .iter()
                    .map(|outcome| outcome.execution_outcome.id.to_string())
                    .collect::<Vec<_>>())),
            )
            .select(schema::receipts::dsl::receipt_id)
            .load_async(&pool)
            .await
        {
            Ok(res) => {
                break res.into_iter().collect();
            }
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while fetching the parent receipt for ExecutionOutcome. Retrying in {} milliseconds... \n {:#?}",
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

    let mut outcome_models: Vec<models::execution_outcomes::ExecutionOutcome> = vec![];
    let mut outcome_receipt_models: Vec<models::execution_outcomes::ExecutionOutcomeReceipt> =
        vec![];
    for (index_in_chunk, outcome) in execution_outcomes
        .iter()
        .filter(|outcome| known_receipt_ids.contains(&(outcome.execution_outcome.id).to_string()))
        .enumerate()
    {
        let model = models::execution_outcomes::ExecutionOutcome::from_execution_outcome(
            &outcome.execution_outcome,
            index_in_chunk as i32,
            block_timestamp,
            &chunk_hash.to_string(),
        );
        outcome_models.push(model);

        outcome_receipt_models.extend(
            outcome
                .execution_outcome
                .outcome
                .receipt_ids
                .iter()
                .enumerate()
                .map(
                    |(index, receipt_id)| models::execution_outcomes::ExecutionOutcomeReceipt {
                        executed_receipt_id: outcome.execution_outcome.id.to_string(),
                        index_in_execution_outcome: index as i32,
                        produced_receipt_id: receipt_id.to_string(),
                    },
                ),
        );
    }

    let mut interval = crate::INTERVAL;
    loop {
        match diesel::insert_into(schema::execution_outcomes::table)
            .values(outcome_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ExecutionOutcome were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    interval.as_millis(),
                    async_error,
                    &outcome_models,
                );
                tokio::time::sleep(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }

    let mut interval = crate::INTERVAL;
    loop {
        match diesel::insert_into(schema::execution_outcome_receipts::table)
            .values(outcome_receipt_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ExecutionOutcomeReceipt were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    interval.as_millis(),
                    async_error,
                    &outcome_receipt_models
                );
                tokio::time::sleep(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}
