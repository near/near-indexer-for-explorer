use diesel::PgConnection;

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
pub mod aggregated;
pub mod assets;
pub mod blocks;
pub mod chunks;
pub mod enums;
pub mod execution_outcomes;
pub mod receipts;
mod serializers;
pub mod transactions;

pub fn establish_connection(database_url: &str) -> actix_diesel::Database<PgConnection> {
    actix_diesel::Database::builder()
        .pool_max_size(30)
        .open(database_url)
}
