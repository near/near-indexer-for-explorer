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
}

impl AccessKey {
    pub fn from_action_view(
        public_key: &near_crypto::PublicKey,
        account_id: &str,
        access_key: &near_indexer::near_primitives::views::AccessKeyView,
        create_by_receipt_id: &near_indexer::near_primitives::hash::CryptoHash,
    ) -> Self {
        Self {
            public_key: public_key.to_string(),
            account_id: account_id.to_string(),
            created_by_receipt_id: Some(create_by_receipt_id.to_string()),
            deleted_by_receipt_id: None,
            permission_kind: (&access_key.permission).into(),
        }
    }
}
