use bigdecimal::BigDecimal;

use crate::schema;
use schema::accounts;

#[derive(Insertable, Debug, Clone, QueryableByName)]
#[table_name = "accounts"]
pub struct Account {
    pub account_id: String,
    pub created_by_receipt_id: Option<String>,
    pub deleted_by_receipt_id: Option<String>,
    pub last_update_block_height: BigDecimal,
}

impl Account {
    pub fn new_from_receipt(
        account_id: &near_indexer::near_primitives::types::AccountId,
        created_by_receipt_id: &near_indexer::near_primitives::hash::CryptoHash,
        last_update_block_height: near_indexer::near_primitives::types::BlockHeight,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            created_by_receipt_id: Some(created_by_receipt_id.to_string()),
            deleted_by_receipt_id: None,
            last_update_block_height: last_update_block_height.into(),
        }
    }

    pub fn new_from_genesis(
        account_id: &near_indexer::near_primitives::types::AccountId,
        last_update_block_height: near_indexer::near_primitives::types::BlockHeight,
    ) -> Self {
        Self {
            account_id: account_id.to_string(),
            created_by_receipt_id: None,
            deleted_by_receipt_id: None,
            last_update_block_height: last_update_block_height.into(),
        }
    }
}
