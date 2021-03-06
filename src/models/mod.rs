use std::env;

use diesel::PgConnection;
use dotenv::dotenv;

pub use access_keys::AccessKey;
pub use account_changes::AccountChange;
pub use accounts::Account;
pub use blocks::Block;
pub use chunks::Chunk;
pub use execution_outcomes::{ExecutionOutcome, ExecutionOutcomeReceipt};
pub use receipts::{
    ActionReceipt, ActionReceiptAction, ActionReceiptInputData, ActionReceiptOutputData,
    DataReceipt, Receipt,
};
pub use transactions::{Transaction, TransactionAction};

pub(crate) use serializers::extract_action_type_and_value_from_action_view;

pub mod access_keys;
pub mod account_changes;
pub mod accounts;
pub mod blocks;
pub mod chunks;
pub mod enums;
pub mod execution_outcomes;
pub mod receipts;
mod serializers;
pub mod transactions;

pub(crate) fn establish_connection() -> actix_diesel::Database<PgConnection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| panic!("DATABASE_URL must be set in .env file"));
    actix_diesel::Database::builder().open(&database_url)
}
