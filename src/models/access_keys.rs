
use near_indexer::near_primitives::views::{AccessKeyView, AccessKeyPermissionView};

use crate::schema;
use schema::access_keys;

#[derive(Insertable)]
pub struct AccessKey {
    pub account_id: String,
    pub public_key: String,
    pub access_key_type: String,
}

impl AccessKey {
    pub fn new(account_id: String, public_key: String, access_key_type: &AccessKeyView) -> Self {
        Self {
            account_id,
            public_key: public_key,
            access_key_type: match access_key_type.permission {
                ref _data @ AccessKeyPermissionView::FunctionCall { .. } => "function_call".to_string(),
                _ => "full_access".to_string(),
            },
        }
    }
}
