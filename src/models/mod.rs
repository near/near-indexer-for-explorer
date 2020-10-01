use std::env;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use serde_json::json;

use near_indexer::near_primitives::views::ActionView;

use enums::ActionKind;
// pub use access_keys::AccessKey;
// pub use accounts::Account;
pub use blocks::Block;
pub use chunks::Chunk;
pub use execution_outcomes::{ExecutionOutcome, ExecutionOutcomeReceipt};
pub use receipts::{
    Receipt, ReceiptAction, ReceiptActionAction, ReceiptActionInputData, ReceiptActionOutputData,
    ReceiptData,
};
pub use transactions::{Transaction, TransactionAction};

pub mod enums;
// pub mod access_keys;
// pub mod accounts;
pub mod blocks;
pub mod chunks;
pub mod execution_outcomes;
pub mod receipts;
pub mod transactions;

pub(crate) fn establish_connection() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| panic!("DATABASE_URL must be set in .env file"));
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    Pool::builder()
        .build(manager)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub(crate) fn extract_action_type_and_value_from_action_view(
    action_view: &near_indexer::near_primitives::views::ActionView,
) -> (enums::ActionKind, serde_json::Value) {
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
                "access_key": access_key,
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
