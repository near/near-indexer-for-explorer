use std::io::Write;

use clap::Parser;

pub use cached::SizedCache;
use futures::future::try_join_all;
use futures::{try_join, StreamExt};
use tokio::sync::Mutex;
use tracing::{debug, info};

use explorer_database::{adapters, models, receipts_cache};

use crate::configs::{Opts, StartOptions};

mod configs;
mod metrics;

// Categories for logging
const INDEXER_FOR_EXPLORER: &str = "indexer_for_explorer";

/// 100KB
const LOG_INTERVAL_BYTES: u64 = 100 * 1024;

async fn handle_message(
    pool: &explorer_database::actix_diesel::Database<explorer_database::diesel::PgConnection>,
    streamer_message: near_lake_framework::near_indexer_primitives::StreamerMessage,
    strict_mode: bool,
    receipts_cache_arc: receipts_cache::ReceiptsCacheArc,
) -> anyhow::Result<()> {
    metrics::BLOCK_COUNT.inc();
    metrics::LATEST_BLOCK_HEIGHT.set(streamer_message.block.header.height.try_into().unwrap());
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Handle Message"
    );

    debug!(
        target: INDEXER_FOR_EXPLORER,
        "ReceiptsCache #{} \n {:#?}", streamer_message.block.header.height, &receipts_cache_arc
    );
    adapters::blocks::store_block(pool, &streamer_message.block).await?;
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Stored block"
    );

    // Chunks
    adapters::chunks::store_chunks(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
    )
    .await?;
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Stored Chunks"
    );

    // Transactions
    let transactions_future = adapters::transactions::store_transactions(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
        streamer_message.block.header.timestamp,
        streamer_message.block.header.height,
        receipts_cache_arc.clone(),
    );
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Stored Transactions"
    );

    // Receipts
    let receipts_future = adapters::receipts::store_receipts(
        pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
        streamer_message.block.header.timestamp,
        strict_mode,
        receipts_cache_arc.clone(),
    );
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Stored Receipts"
    );
    // We can process transactions and receipts in parallel
    // because most of receipts depend on transactions from previous blocks,
    // so we can save up some time here.
    // In case of local receipts (they are stored in the same block with corresponding transaction),
    // we hope retry logic will cover it fine
    try_join!(transactions_future, receipts_future)?;
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Joined futures"
    );

    // ExecutionOutcomes
    let execution_outcomes_future = adapters::execution_outcomes::store_execution_outcomes(
        pool,
        &streamer_message.shards,
        streamer_message.block.header.timestamp,
        receipts_cache_arc.clone(),
    );
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Stored Outcomes"
    );

    // Accounts
    let accounts_future = async {
        let futures = streamer_message.shards.iter().map(|shard| {
            adapters::accounts::handle_accounts(
                pool,
                &shard.receipt_execution_outcomes,
                streamer_message.block.header.height,
            )
        });

        try_join_all(futures).await.map(|_| ())
    };
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Stored accounts"
    );

    // Event-based entities (FT, NFT)
    let assets_events_future = adapters::assets::events::store_events(pool, &streamer_message);
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Stored Events"
    );

    if strict_mode {
        // AccessKeys
        let access_keys_future = async {
            let futures = streamer_message.shards.iter().map(|shard| {
                adapters::access_keys::handle_access_keys(
                    pool,
                    &shard.state_changes,
                    streamer_message.block.header.height,
                )
            });

            try_join_all(futures).await.map(|_| ())
        };
        info!(
            target: crate::INDEXER_FOR_EXPLORER,
            "Stored Keys"
        );

        // StateChange related to Account
        #[cfg(feature = "account_changes")]
        let account_changes_future = adapters::account_changes::store_account_changes(
            pool,
            &streamer_message.shards,
            &streamer_message.block.header.hash,
            streamer_message.block.header.timestamp,
        );
        #[cfg(feature = "account_changes")]
        try_join!(
            execution_outcomes_future,
            accounts_future,
            access_keys_future,
            assets_events_future,
            account_changes_future
        )?;

        #[cfg(not(feature = "account_changes"))]
        try_join!(
            execution_outcomes_future,
            accounts_future,
            access_keys_future,
            assets_events_future,
        )?;
        info!(
            target: crate::INDEXER_FOR_EXPLORER,
            "Joined Changes"
        );
    } else {
        try_join!(
            execution_outcomes_future,
            accounts_future,
            assets_events_future
        )?;
        info!(
            target: crate::INDEXER_FOR_EXPLORER,
            "Joined All"
        );
    }

    Ok(())
}

async fn download_genesis_file(opts: &configs::Opts) -> anyhow::Result<std::path::PathBuf> {
    let res = reqwest::get(opts.genesis_file_url()).await?;

    let total_size = res.content_length().unwrap();

    let mut exe_path = std::env::current_exe()?;
    exe_path.pop();
    let genesis_path = exe_path.join("genesis.json");

    match std::fs::File::open(genesis_path.clone()) {
        Ok(_) => {
            tracing::info!(
                target: INDEXER_FOR_EXPLORER,
                "Using existing genesis file: {}",
                genesis_path.display()
            );
        }
        Err(_) => {
            let mut file = std::fs::File::create(genesis_path.clone())?;
            let mut downloaded: u64 = 0;
            let mut downloaded_since_last_log: u64 = 0;

            let mut stream = res.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                let chunk_len = chunk.len() as u64;

                downloaded = std::cmp::min(downloaded + chunk_len, total_size);
                downloaded_since_last_log += chunk_len;

                if downloaded_since_last_log >= LOG_INTERVAL_BYTES {
                    downloaded_since_last_log = 0;
                    tracing::info!(
                        target: INDEXER_FOR_EXPLORER,
                        "Downloading genesis.json: {}/{} ({}%)",
                        indicatif::HumanBytes(downloaded),
                        indicatif::HumanBytes(total_size),
                        downloaded * 100 / total_size
                    );
                }

                file.write_all(&chunk)?;
            }

            tracing::info!(
                target: INDEXER_FOR_EXPLORER,
                "Downloading genesis.json: {}/{} (100%)",
                indicatif::HumanBytes(total_size),
                indicatif::HumanBytes(total_size),
            );

            file.flush()?;
        }
    }

    Ok(genesis_path)
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
    let receipts_cache_arc: receipts_cache::ReceiptsCacheArc =
        std::sync::Arc::new(Mutex::new(SizedCache::with_size(100_000)));

    tracing::info!(
        target: INDEXER_FOR_EXPLORER,
        "Starting Indexer for Explorer (lake)...",
    );

    tokio::spawn(metrics::init_server(opts.port).expect("Failed to start metrics server"));

    match opts.start_options() {
        StartOptions::FromGenesis {
            from_interuption: _,
            genesis_file_path,
        } => match genesis_file_path {
            Some(genesis_file_path) => {
                use std::path::PathBuf;
                adapters::genesis::store_genesis_records(
                    pool.clone(),
                    PathBuf::from(genesis_file_path),
                )
                .await?;
            }
            None => {
                let genesis_file_path = download_genesis_file(&opts).await?;
                adapters::genesis::store_genesis_records(pool.clone(), genesis_file_path).await?;
            }
        },
        StartOptions::FromLatest => {}
        StartOptions::FromBlock { height: _ } => {}
        StartOptions::FromInterruption => {}
    }
    // TODO(mhala) does not work after adding from-interruption flag
    // if opts.start_options() == &StartOptions::FromGenesis {
    //     let genesis_file_path = download_genesis_file(&opts).await?;
    //     adapters::genesis::store_genesis_records(pool.clone(), genesis_file_path).await?;
    // }

    let config: near_lake_framework::LakeConfig = opts.to_lake_config().await;
    let (sender, stream) = near_lake_framework::streamer(config);

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
                receipts_cache_arc.clone(),
            )
        })
        .buffer_unordered(usize::from(opts.concurrency.get()));

    while let Some(handle_message) = handlers.next().await {
        if let Err(e) = handle_message {
            tracing::error!(
                target: crate::INDEXER_FOR_EXPLORER,
                "Encountered error while indexing: {}",
                e
            );
            if strict_mode {
                anyhow::bail!(e)
            }
        }
    }

    drop(handlers); // close the channel so the sender will stop
    match sender.await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(anyhow::Error::from(e)),
    }
}
