use serde_json::{json, Value};

use near_indexer;

use near_indexer::near_primitives::views::ActionView;

use crate::schema;
use schema::actions;

#[derive(Insertable)]
pub struct Action {
    pub receipt_id: String,
    pub index: i32,
    pub type_: String,
    pub args: Option<serde_json::Value>,
}

impl Action {
    pub fn from_action(receipt_id: String, index: i32, action_view: &ActionView) -> Self {
        let (type_, args): (&str, Option<Value>) = match &action_view {
            ActionView::CreateAccount => ("create_account", None),
            ActionView::DeployContract { code } => {
                ("deploy_contract", Some(json!({ "code": code })))
            }
            ActionView::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => (
                "function_call",
                Some(json!({
                    "method_name": method_name,
                    "args": args,
                    "gas": gas,
                    "deposit": deposit.to_string(),
                })),
            ),
            ActionView::Transfer { deposit } => {
                ("transfer", Some(json!({ "deposit": deposit.to_string() })))
            }
            ActionView::Stake { stake, public_key } => (
                "stake",
                Some(json!({
                    "stake": stake.to_string(),
                    "public_key": public_key,
                })),
            ),
            ActionView::AddKey {
                public_key,
                access_key,
            } => (
                "add_key",
                Some(json!({
                    "public_key": public_key,
                    "access_key": access_key,
                })),
            ),
            ActionView::DeleteKey { public_key } => (
                "delete_key",
                Some(json!({
                    "public_key": public_key,
                })),
            ),
            ActionView::DeleteAccount { beneficiary_id } => (
                "delete_account",
                Some(json!({
                    "beneficiary_id": beneficiary_id,
                })),
            ),
        };
        Self {
            receipt_id,
            index,
            args,
            type_: type_.to_string(),
        }
    }
}
