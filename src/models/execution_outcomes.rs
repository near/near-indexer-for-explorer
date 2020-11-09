// use num_traits::cast::FromPrimitive;
use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::models::enums::ExecutionOutcomeStatus;

use crate::schema;
use schema::{execution_outcome_receipts, execution_outcomes};

#[derive(Insertable, Clone, Debug)]
pub struct ExecutionOutcome {
    pub receipt_id: String,
    pub block_hash: String,
    pub gas_burnt: BigDecimal,
    pub tokens_burnt: BigDecimal,
    pub executor_account_id: String,
    pub status: ExecutionOutcomeStatus,
}

impl From<&near_indexer::near_primitives::views::ExecutionOutcomeWithIdView> for ExecutionOutcome {
    fn from(
        execution_outcome: &near_indexer::near_primitives::views::ExecutionOutcomeWithIdView,
    ) -> Self {
        Self {
            block_hash: execution_outcome.block_hash.to_string(),
            receipt_id: execution_outcome.id.to_string(),
            gas_burnt: execution_outcome.outcome.gas_burnt.into(),
            tokens_burnt: BigDecimal::from_str(
                execution_outcome.outcome.tokens_burnt.to_string().as_str(),
            )
            .expect("`tokens_burnt` expected to be u128"),
            executor_account_id: execution_outcome.outcome.executor_id.to_string(),
            status: execution_outcome.outcome.status.clone().into(),
        }
    }
}

#[derive(Insertable, Queryable, Clone, Debug)]
pub struct ExecutionOutcomeReceipt {
    pub execution_outcome_receipt_id: String,
    pub index: i32,
    pub receipt_id: String,
}
