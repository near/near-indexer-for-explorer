use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::models::enums::ExecutionOutcomeStatus;

use crate::schema;
use schema::{execution_outcome_receipts, execution_outcomes};

#[derive(Insertable, Clone, Debug)]
pub struct ExecutionOutcome {
    pub receipt_id: String,
    pub executed_in_block_hash: String,
    pub executed_in_block_timestamp: BigDecimal,
    pub index_in_chunk: i32,
    pub gas_burnt: BigDecimal,
    pub tokens_burnt: BigDecimal,
    pub executor_account_id: String,
    pub status: ExecutionOutcomeStatus,
    pub shard_id: BigDecimal,
}

impl ExecutionOutcome {
    pub fn from_execution_outcome(
        execution_outcome: &near_indexer::near_primitives::views::ExecutionOutcomeWithIdView,
        index_in_chunk: i32,
        executed_in_block_timestamp: u64,
        shard_id: u64,
    ) -> Self {
        Self {
            executed_in_block_hash: execution_outcome.block_hash.to_string(),
            executed_in_block_timestamp: executed_in_block_timestamp.into(),
            index_in_chunk,
            receipt_id: execution_outcome.id.to_string(),
            gas_burnt: execution_outcome.outcome.gas_burnt.into(),
            tokens_burnt: BigDecimal::from_str(
                execution_outcome.outcome.tokens_burnt.to_string().as_str(),
            )
            .expect("`tokens_burnt` expected to be u128"),
            executor_account_id: execution_outcome.outcome.executor_id.to_string(),
            status: execution_outcome.outcome.status.clone().into(),
            shard_id: shard_id.into(),
        }
    }
}

#[derive(Insertable, Queryable, Clone, Debug)]
pub struct ExecutionOutcomeReceipt {
    pub executed_receipt_id: String,
    pub index_in_execution_outcome: i32,
    pub produced_receipt_id: String,
}
