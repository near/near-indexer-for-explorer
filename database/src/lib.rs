#[macro_use]
pub extern crate diesel;

pub use actix_diesel;

pub mod adapters;
pub mod models;
pub mod receipts_cache;

mod schema;
#[macro_use]
mod retryable;

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const EXPLORER_DATABASE: &str = "explorer_database";

const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);
