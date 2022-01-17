use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::PgConnection;
use futures::future::try_join_all;

use crate::models;
use crate::schema;

pub(crate) async fn store_execution_outcomes(
    pool: &actix_diesel::Database<PgConnection>,
    shards: &[near_indexer::IndexerShard],
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let futures = shards.iter().map(|shard| {
        store_execution_outcomes_for_chunk(
            pool,
            &shard.receipt_execution_outcomes,
            shard.shard_id,
            block_timestamp,
        )
    });

    try_join_all(futures).await.map(|_| ())
}

/// Saves ExecutionOutcome to database and then saves ExecutionOutcomesReceipts
pub async fn store_execution_outcomes_for_chunk(
    pool: &actix_diesel::Database<PgConnection>,
    execution_outcomes: &[near_indexer::IndexerExecutionOutcomeWithReceipt],
    shard_id: near_indexer::near_primitives::types::ShardId,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let mut outcome_models: Vec<models::execution_outcomes::ExecutionOutcome> = vec![];
    let mut outcome_receipt_models: Vec<models::execution_outcomes::ExecutionOutcomeReceipt> =
        vec![];
    for (index_in_chunk, outcome) in execution_outcomes.iter().enumerate() {
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
