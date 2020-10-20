use serde::{Deserialize, Serialize};
use near_indexer::near_primitives::serialize::option_u128_dec_format;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct AccessKeyView {
    pub nonce: near_indexer::near_primitives::types::Nonce,
    pub permission: AccessKeyPermissionView,
}

impl From<&near_indexer::near_primitives::views::AccessKeyView> for AccessKeyView {
    fn from(access_key_view: &near_indexer::near_primitives::views::AccessKeyView) -> Self {
        Self {
            nonce: access_key_view.nonce,
            permission: access_key_view.permission.clone().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "permission_kind", content = "permission_details", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AccessKeyPermissionView {
    FunctionCall {
        #[serde(with = "option_u128_dec_format")]
        allowance: Option<near_indexer::near_primitives::types::Balance>,
        receiver_id: near_indexer::near_primitives::types::AccountId,
        method_names: Vec<String>,
    },
    FullAccess,
}

impl From<near_indexer::near_primitives::views::AccessKeyPermissionView> for AccessKeyPermissionView {
    fn from(permission: near_indexer::near_primitives::views::AccessKeyPermissionView) -> Self {
        match permission {
            near_indexer::near_primitives::views::AccessKeyPermissionView::FullAccess => Self::FullAccess,
            near_indexer::near_primitives::views::AccessKeyPermissionView::FunctionCall {
                allowance,
                receiver_id,
                method_names,
            } => Self::FunctionCall {
                allowance: allowance.clone(),
                receiver_id: receiver_id.clone(),
                method_names: method_names.clone(),
            }
        }
    }
}
