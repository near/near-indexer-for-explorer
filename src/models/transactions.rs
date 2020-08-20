use num_traits::cast::FromPrimitive;

use bigdecimal::BigDecimal;

use near_indexer;

use crate::schema;
use schema::transactions;

#[derive(Insertable)]
pub struct Transaction {
    pub hash: String,
    pub block_id: BigDecimal,
    pub block_timestamp: BigDecimal,
    pub nonce: BigDecimal,
    pub signer_id: String,
    pub signer_public_key: String,
    pub signature: String,
    pub receiver_id: String,
    pub receipt_conversion_gas_burnt: Option<BigDecimal>,
    pub receipt_conversion_tokens_burnt: Option<BigDecimal>,
    pub receipt_id: Option<String>,
}

impl Transaction {
    pub fn from_transaction_view(
        block_height: u64,
        block_timestamp: u64,
        transaction_view: &near_indexer::near_primitives::views::SignedTransactionView,
    ) -> Self {
        Self {
            hash: transaction_view.hash.to_string(),
            block_id: BigDecimal::from_u64(block_height).unwrap_or(0.into()),
            block_timestamp: BigDecimal::from_u64(block_timestamp).unwrap_or(0.into()),
            nonce: BigDecimal::from_u64(transaction_view.nonce).unwrap_or(0.into()),
            signer_id: transaction_view.signer_id.to_string(),
            signer_public_key: transaction_view.public_key.to_string(),
            signature: transaction_view.signature.to_string(),
            receiver_id: transaction_view.receiver_id.to_string(),
            receipt_conversion_gas_burnt: None,
            receipt_conversion_tokens_burnt: None,
            receipt_id: None,
        }
    }
}
