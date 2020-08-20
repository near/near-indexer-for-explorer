use std::env;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;

pub mod blocks;
pub mod chunks;
pub mod transactions;
pub mod receipts;
pub mod actions;
pub mod accounts;
pub mod access_keys;

pub use blocks::Block;
pub use chunks::Chunk;
pub use transactions::Transaction;
pub use receipts::{
    Receipt, ReceiptData, ReceiptAction, ReceiptActionOutputData, ReceiptActionInputData
};
pub use actions::Action;
pub use accounts::Account;
pub use access_keys::AccessKey;


pub(crate) fn establish_connection() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| panic!("DATABASE_URL must be set in .env file"));
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    Pool::builder()
        .build(manager)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
