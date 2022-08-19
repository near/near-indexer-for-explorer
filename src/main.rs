use clap::Parser;
#[macro_use]
extern crate diesel;

pub use cached::SizedCache;
use diesel::PgConnection;
use futures::future::try_join_all;
use futures::{try_join, StreamExt};
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::configs::Opts;

mod configs;
mod db_adapters;
mod models;
mod schema;
#[macro_use]
mod retriable;

// Categories for logging
const INDEXER_FOR_EXPLORER: &str = "indexer_for_explorer";
// const AGGREGATED: &str = "aggregated";

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum ReceiptOrDataId {
    ReceiptId(near_lake_framework::near_indexer_primitives::CryptoHash),
    DataId(near_lake_framework::near_indexer_primitives::CryptoHash),
}
// Creating type aliases to make HashMap types for cache more explicit
pub type ParentTransactionHashString = String;
// Introducing a simple cache for Receipts to find their parent Transactions without
// touching the database
// The key is ReceiptID
// The value is TransactionHash (the very parent of the Receipt)
pub type ReceiptsCache =
    std::sync::Arc<Mutex<SizedCache<ReceiptOrDataId, ParentTransactionHashString>>>;

async fn handle_message(
    pool: &actix_diesel::Database<PgConnection>,
    streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
    strict_mode: bool,
    receipts_cache: ReceiptsCache,
) -> anyhow::Result<()> {
    debug!(
        target: INDEXER_FOR_EXPLORER,
        "ReceiptsCache #{} \n {:#?}", streamer_message.block.header.height, &receipts_cache
    );
    db_adapters::blocks::store_block(pool, &streamer_message.block).await?;

    // Chunks
    db_adapters::chunks::store_chunks(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
    )
    .await?;

    // Transactions
    let transactions_future = db_adapters::transactions::store_transactions(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
        streamer_message.block.header.timestamp,
        streamer_message.block.header.height,
        std::sync::Arc::clone(&receipts_cache),
    );

    // Receipts
    let receipts_future = db_adapters::receipts::store_receipts(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
        streamer_message.block.header.timestamp,
        strict_mode,
        std::sync::Arc::clone(&receipts_cache),
    );

    // We can process transactions and receipts in parallel
    // because most of receipts depend on transactions from previous blocks,
    // so we can save up some time here.
    // In case of local receipts (they are stored in the same block with corresponding transaction),
    // we hope retry logic will cover it fine
    try_join!(transactions_future, receipts_future)?;

    // ExecutionOutcomes
    let execution_outcomes_future = db_adapters::execution_outcomes::store_execution_outcomes(
        pool,
        &streamer_message.shards,
        streamer_message.block.header.timestamp,
        std::sync::Arc::clone(&receipts_cache),
    );

    // Accounts
    let accounts_future = async {
        let futures = streamer_message.shards.iter().map(|shard| {
            db_adapters::accounts::handle_accounts(
                pool,
                &shard.receipt_execution_outcomes,
                streamer_message.block.header.height,
            )
        });

        try_join_all(futures).await.map(|_| ())
    };

    // Event-based entities (FT, NFT)
    let assets_events_future = db_adapters::assets::events::store_events(pool, &streamer_message);

    if strict_mode {
        // AccessKeys
        let access_keys_future = async {
            let futures = streamer_message.shards.iter().map(|shard| {
                db_adapters::access_keys::handle_access_keys(
                    pool,
                    &shard.receipt_execution_outcomes,
                    streamer_message.block.header.height,
                )
            });

            try_join_all(futures).await.map(|_| ())
        };

        // StateChange related to Account
        let account_changes_future = db_adapters::account_changes::store_account_changes(
            pool,
            &streamer_message.shards,
            &streamer_message.block.header.hash,
            streamer_message.block.header.timestamp,
        );

        try_join!(
            execution_outcomes_future,
            accounts_future,
            access_keys_future,
            assets_events_future,
            account_changes_future,
        )?;
    } else {
        try_join!(
            execution_outcomes_future,
            accounts_future,
            assets_events_future
        )?;
    }

    Ok(())
}

#[actix::main]
async fn main() -> anyhow::Result<()> {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();

    dotenv::dotenv().ok();

    let opts: Opts = Opts::parse();

    configs::init_tracing(opts.debug)?;

    // We establish connection as early as possible as an additional sanity check.
    // Indexer should fail if .env file with credentials is missing/wrong
    let pool = models::establish_connection(&opts.database_url);

    let strict_mode = !opts.non_strict_mode;

    // We want to prevent unnecessary SELECT queries to the database to find
    // the Transaction hash for the Receipt.
    // Later we need to find the Receipt which is a parent to underlying Receipts.
    // Receipt ID will of the child will be stored as key and parent Transaction hash/Receipt ID
    // will be stored as a value
    let receipts_cache: ReceiptsCache =
        std::sync::Arc::new(Mutex::new(SizedCache::with_size(100_000)));

    let config: near_lake_framework::LakeConfig = opts.to_lake_config().await;
    let (sender, stream) = near_lake_framework::streamer(config);

    tracing::info!(
        target: INDEXER_FOR_EXPLORER,
        "Starting Indexer for Explorer (lake)...",
    );
    let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
        .map(|streamer_message| {
            info!(
                target: crate::INDEXER_FOR_EXPLORER,
                "Block height {}", &streamer_message.block.header.height
            );
            handle_message(
                &pool,
                streamer_message,
                strict_mode,
                std::sync::Arc::clone(&receipts_cache),
            )
        })
        .buffer_unordered(1usize);

    while let Some(_handle_message) = handlers.next().await {}
    drop(handlers); // close the channel so the sender will stop

    // propagate errors from the sender
    match sender.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(anyhow::Error::from(e)), // JoinError
    }
}
