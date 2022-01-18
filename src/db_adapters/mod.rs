pub(crate) mod access_keys;
pub(crate) mod account_changes;
pub(crate) mod accounts;
pub(crate) mod aggregated;
pub(crate) mod assets;
pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
pub(crate) mod genesis;
pub(crate) mod receipts;
pub(crate) mod transactions;

const CHUNK_SIZE_FOR_BATCH_INSERT: usize = 500;
