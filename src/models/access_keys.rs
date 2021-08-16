use bigdecimal::BigDecimal;

use crate::models::enums::AccessKeyPermission;
use crate::schema;
use schema::access_keys;

#[derive(Insertable, Clone, Debug)]
pub struct AccessKey {
    pub public_key: String,
    pub account_id: String,
    pub created_by_receipt_id: Option<String>,
    pub deleted_by_receipt_id: Option<String>,
    pub permission_kind: AccessKeyPermission,
    pub last_update_block_height: BigDecimal,
}

impl AccessKey {
    pub fn from_action_view(
        public_key: &near_crypto::PublicKey,
        account_id: &near_indexer::near_primitives::types::AccountId,
        access_key: &near_indexer::near_primitives::views::AccessKeyView,
        create_by_receipt_id: &near_indexer::near_primitives::hash::CryptoHash,
        last_update_block_height: near_indexer::near_primitives::types::BlockHeight,
    ) -> Self {
        Self {
            public_key: public_key.to_string(),
            account_id: account_id.to_string(),
            created_by_receipt_id: Some(create_by_receipt_id.to_string()),
            deleted_by_receipt_id: None,
            permission_kind: (&access_key.permission).into(),
            last_update_block_height: last_update_block_height.into(),
        }
    }

    pub fn from_genesis(
        public_key: &near_crypto::PublicKey,
        account_id: &near_indexer::near_primitives::types::AccountId,
        access_key: &near_indexer::near_primitives::account::AccessKey,
        last_update_block_height: near_indexer::near_primitives::types::BlockHeight,
    ) -> Self {
        Self {
            public_key: public_key.to_string(),
            account_id: account_id.to_string(),
            created_by_receipt_id: None,
            deleted_by_receipt_id: None,
            permission_kind: (&access_key.permission).into(),
            last_update_block_height: last_update_block_height.into(),
        }
    }
}
