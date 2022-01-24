use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::models::enums::{ActionKind, ExecutionOutcomeStatus};
use crate::schema;
use schema::{transaction_actions, transactions};

#[derive(Insertable, Clone, Debug)]
pub struct Transaction {
    pub transaction_hash: String,
    pub included_in_block_hash: String,
    pub included_in_chunk_hash: String,
    pub index_in_chunk: i32,
    pub block_timestamp: BigDecimal,
    pub signer_account_id: String,
    pub signer_public_key: String,
    pub nonce: BigDecimal,
    pub receiver_account_id: String,
    pub signature: String,
    pub status: ExecutionOutcomeStatus,
    pub converted_into_receipt_id: String,
    pub receipt_conversion_gas_burnt: BigDecimal,
    pub receipt_conversion_tokens_burnt: BigDecimal,
}

impl Transaction {
    pub fn from_indexer_transaction(
        tx: &near_indexer::IndexerTransactionWithOutcome,
        // hack for supporting duplicated transaction hashes
        transaction_hash: &str,
        converted_into_receipt_id: &str,
        block_hash: &near_indexer::near_primitives::hash::CryptoHash,
        chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
        block_timestamp: u64,
        index_in_chunk: i32,
    ) -> Self {
        Self {
            transaction_hash: transaction_hash.to_string(),
            included_in_block_hash: block_hash.to_string(),
            block_timestamp: block_timestamp.into(),
            index_in_chunk,
            nonce: tx.transaction.nonce.into(),
            signer_account_id: tx.transaction.signer_id.to_string(),
            signer_public_key: tx.transaction.public_key.to_string(),
            signature: tx.transaction.signature.to_string(),
            receiver_account_id: tx.transaction.receiver_id.to_string(),
            converted_into_receipt_id: converted_into_receipt_id.to_string(),
            included_in_chunk_hash: chunk_hash.to_string(),
            status: tx.outcome.execution_outcome.outcome.status.clone().into(),
            receipt_conversion_gas_burnt: tx.outcome.execution_outcome.outcome.gas_burnt.into(),
            receipt_conversion_tokens_burnt: BigDecimal::from_str(
                tx.outcome
                    .execution_outcome
                    .outcome
                    .tokens_burnt
                    .to_string()
                    .as_str(),
            )
            .expect("`token_burnt` must be u128"),
        }
    }
}

#[derive(Insertable, Clone, Debug)]
pub struct TransactionAction {
    pub transaction_hash: String,
    pub index_in_transaction: i32,
    pub action_kind: ActionKind,
    pub args: serde_json::Value,
}

impl TransactionAction {
    pub fn from_action_view(
        transaction_hash: String,
        index: i32,
        action_view: &near_indexer::near_primitives::views::ActionView,
    ) -> Self {
        let (action_kind, args) =
            crate::models::extract_action_type_and_value_from_action_view(action_view);
        Self {
            transaction_hash,
            index_in_transaction: index,
            args,
            action_kind,
        }
    }
}
