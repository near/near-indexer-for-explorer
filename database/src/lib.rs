#[macro_use]
pub extern crate diesel;

pub use actix_diesel;

pub mod db_adapters;
pub mod models;

mod schema;
#[macro_use]
mod retriable;

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const INDEXER_FOR_EXPLORER: &str = "indexer_for_explorer";

const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);
