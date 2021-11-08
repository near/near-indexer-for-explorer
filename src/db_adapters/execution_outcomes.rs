use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::{ExpressionMethods, PgConnection, QueryDsl};

use crate::models;
use crate::schema;
use diesel::pg::expression::array_comparison::any;

pub(crate) async fn store_execution_outcomes(
    pool: &actix_diesel::Database<PgConnection>,
    shards: &[near_indexer::IndexerShard],
    block_timestamp: u64,
) -> anyhow::Result<()> {
    for shard in shards {
        store_execution_outcomes_for_chunk(
            pool,
            &shard.receipt_execution_outcomes,
            shard.shard_id,
            block_timestamp,
        )
        .await?;
    }
    Ok(())
}

/// Saves ExecutionOutcome to database and then saves ExecutionOutcomesReceipts
pub async fn store_execution_outcomes_for_chunk(
    pool: &actix_diesel::Database<PgConnection>,
    execution_outcomes: &[near_indexer::IndexerExecutionOutcomeWithReceipt],
    shard_id: near_indexer::near_primitives::types::ShardId,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let known_receipt_ids: std::collections::HashSet<String> = crate::await_retry_or_panic!(
        schema::receipts::table
            .filter(
                schema::receipts::dsl::receipt_id.eq(any(execution_outcomes
                    .iter()
                    .map(|outcome| outcome.execution_outcome.id.to_string())
                    .collect::<Vec<_>>())),
            )
            .select(schema::receipts::dsl::receipt_id)
            .load_async::<String>(pool),
        10,
        "Parent Receipt for ExecutionOutcome was fetched".to_string(),
        &execution_outcomes
    )
    .unwrap_or_default()
    .into_iter()
    .collect();

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
            shard_id,
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

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::execution_outcomes::table)
            .values(outcome_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "ExecutionOutcomes were stored in database".to_string(),
        &outcome_models
    );

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::execution_outcome_receipts::table)
            .values(outcome_receipt_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "ExecutionOutcomeReceipts were stored in database".to_string(),
        &outcome_receipt_models
    );

    Ok(())
}
