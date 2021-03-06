use serde::{Deserialize, Serialize};
use serde_json::json;

use near_indexer::near_primitives::serialize::option_u128_dec_format;
use near_indexer::near_primitives::views::ActionView;

use crate::models::enums::ActionKind;

/// We want to store permission field more explicitly so we are making copy of nearcore struct
/// to change serde parameters of serialization.
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

/// This is a enum we want to store more explicitly, so we copy it from nearcore and provide
/// different serde representation settings
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(
    tag = "permission_kind",
    content = "permission_details",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub(crate) enum AccessKeyPermissionView {
    FunctionCall {
        #[serde(with = "option_u128_dec_format")]
        allowance: Option<near_indexer::near_primitives::types::Balance>,
        receiver_id: near_indexer::near_primitives::types::AccountId,
        method_names: Vec<String>,
    },
    FullAccess,
}

impl From<near_indexer::near_primitives::views::AccessKeyPermissionView>
    for AccessKeyPermissionView
{
    fn from(permission: near_indexer::near_primitives::views::AccessKeyPermissionView) -> Self {
        match permission {
            near_indexer::near_primitives::views::AccessKeyPermissionView::FullAccess => {
                Self::FullAccess
            }
            near_indexer::near_primitives::views::AccessKeyPermissionView::FunctionCall {
                allowance,
                receiver_id,
                method_names,
            } => Self::FunctionCall {
                allowance,
                receiver_id,
                method_names,
            },
        }
    }
}

pub(crate) fn extract_action_type_and_value_from_action_view(
    action_view: &near_indexer::near_primitives::views::ActionView,
) -> (crate::models::enums::ActionKind, serde_json::Value) {
    match action_view {
        ActionView::CreateAccount => (ActionKind::CreateAccount, json!({})),
        ActionView::DeployContract { code } => (
            ActionKind::DeployContract,
            json!({
                "code_sha256":  hex::encode(
                    base64::decode(code).expect("code expected to be encoded to base64")
                )
            }),
        ),
        ActionView::FunctionCall {
            method_name,
            args,
            gas,
            deposit,
        } => (
            ActionKind::FunctionCall,
            json!({
                "method_name": method_name.escape_default().to_string(),
                "args_base64": args,
                "gas": gas,
                "deposit": deposit.to_string(),
            }),
        ),
        ActionView::Transfer { deposit } => (
            ActionKind::Transfer,
            json!({ "deposit": deposit.to_string() }),
        ),
        ActionView::Stake { stake, public_key } => (
            ActionKind::Stake,
            json!({
                "stake": stake.to_string(),
                "public_key": public_key,
            }),
        ),
        ActionView::AddKey {
            public_key,
            access_key,
        } => (
            ActionKind::AddKey,
            json!({
                "public_key": public_key,
                "access_key": crate::models::serializers::AccessKeyView::from(access_key),
            }),
        ),
        ActionView::DeleteKey { public_key } => (
            ActionKind::DeleteKey,
            json!({
                "public_key": public_key,
            }),
        ),
        ActionView::DeleteAccount { beneficiary_id } => (
            ActionKind::DeleteAccount,
            json!({
                "beneficiary_id": beneficiary_id,
            }),
        ),
    }
}
