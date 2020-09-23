use std::str::FromStr;

use bigdecimal::BigDecimal;
use serde_json::{json, Value};

use near_indexer::near_primitives::views::ActionView;

use crate::models::enums::{ActionType, ExecutionOutcomeStatus};
use crate::schema;
use schema::{transaction_actions, transactions};

#[derive(Insertable, Clone)]
pub struct Transaction {
    pub transaction_hash: Vec<u8>,
    pub block_height: BigDecimal,
    pub chunk_hash: Vec<u8>,
    pub signer_id: String,
    pub public_key: String,
    pub nonce: BigDecimal,
    pub receiver_id: String,
    pub signature: String,
    pub status: ExecutionOutcomeStatus,
    pub receipt_id: Vec<u8>,
    pub receipt_conversion_gas_burnt: BigDecimal,
    pub receipt_conversion_tokens_burnt: BigDecimal,
}

impl Transaction {
    pub fn from_indexer_transaction(
        tx: &near_indexer::IndexerTransactionWithOutcome,
        block_height: u64,
        chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    ) -> Self {
        Self {
            transaction_hash: tx.transaction.hash.as_ref().to_vec(),
            block_height: block_height.into(),
            nonce: tx.transaction.nonce.into(),
            signer_id: tx.transaction.signer_id.to_string(),
            public_key: tx.transaction.public_key.to_string(),
            signature: tx.transaction.signature.to_string(),
            receiver_id: tx.transaction.receiver_id.to_string(),
            receipt_id: tx
                .outcome
                .outcome
                .receipt_ids
                .first()
                .expect("`receipt_ids` must contain one Receipt Id")
                .as_ref()
                .to_vec(),
            chunk_hash: chunk_hash.as_ref().to_vec(),
            status: tx.outcome.outcome.status.clone().into(),
            receipt_conversion_gas_burnt: tx.outcome.outcome.gas_burnt.into(),
            receipt_conversion_tokens_burnt: BigDecimal::from_str(tx.outcome.outcome.tokens_burnt.to_string().as_str())
                .expect("`token_burnt` must be u128"),
        }
    }
}

#[derive(Insertable, Clone)]
pub struct TransactionAction {
    pub transaction_hash: Vec<u8>,
    pub index: i32,
    pub action_kind: ActionType,
    pub args: serde_json::Value,
}

impl TransactionAction {
    pub fn from_action_view(
        transaction_hash: Vec<u8>,
        index: i32,
        action_view: &near_indexer::near_primitives::views::ActionView,
    ) -> Self {
        let (action_kind, args): (ActionType, Value) = match &action_view {
            ActionView::CreateAccount => (ActionType::CreateAccount, json!({})),
            ActionView::DeployContract { code } => {
                (ActionType::DeployContract, json!({ "code": code }))
            }
            ActionView::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => (
                ActionType::FunctionCall,
                json!({
                    "method_name": method_name,
                    "args": args,
                    "gas": gas,
                    "deposit": deposit.to_string(),
                }),
            ),
            ActionView::Transfer { deposit } => (
                ActionType::Transfer,
                json!({ "deposit": deposit.to_string() }),
            ),
            ActionView::Stake { stake, public_key } => (
                ActionType::Stake,
                json!({
                    "stake": stake.to_string(),
                    "public_key": public_key,
                }),
            ),
            ActionView::AddKey {
                public_key,
                access_key,
            } => (
                ActionType::AddKey,
                json!({
                    "public_key": public_key,
                    "access_key": access_key,
                }),
            ),
            ActionView::DeleteKey { public_key } => (
                ActionType::DeleteKey,
                json!({
                    "public_key": public_key,
                }),
            ),
            ActionView::DeleteAccount { beneficiary_id } => (
                ActionType::DeleteAccount,
                json!({
                    "beneficiary_id": beneficiary_id,
                }),
            ),
        };
        Self {
            transaction_hash,
            index,
            args,
            action_kind,
        }
    }
}
