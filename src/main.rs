use clap::Parser;
use std::convert::TryFrom;
#[macro_use]
extern crate diesel;

use actix_diesel::Database;
pub use cached::SizedCache;
use diesel::PgConnection;
use futures::future::try_join_all;
use futures::{try_join, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

use crate::configs::{Opts, SubCommand};

mod aggregated;
mod configs;
mod db_adapters;
mod models;
mod schema;
#[macro_use]
mod retriable;

// Categories for logging
const INDEXER_FOR_EXPLORER: &str = "indexer_for_explorer";
const AGGREGATED: &str = "aggregated";

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum ReceiptOrDataId {
    ReceiptId(near_indexer::near_primitives::hash::CryptoHash),
    DataId(near_indexer::near_primitives::hash::CryptoHash),
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
    streamer_message: near_indexer::StreamerMessage,
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
            &streamer_message.state_changes,
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

async fn listen_blocks(
    stream: mpsc::Receiver<near_indexer::StreamerMessage>,
    pool: Database<PgConnection>,
    concurrency: std::num::NonZeroU16,
    strict_mode: bool,
    stop_after_number_of_blocks: Option<std::num::NonZeroUsize>,
) {
    if let Some(stop_after_n_blocks) = stop_after_number_of_blocks {
        warn!(
            target: crate::INDEXER_FOR_EXPLORER,
            "Indexer will stop after indexing {} blocks", stop_after_n_blocks,
        );
    }
    if !strict_mode {
        warn!(
            target: crate::INDEXER_FOR_EXPLORER,
            "Indexer is starting in NON-STRICT mode",
        );
    }
    info!(target: crate::INDEXER_FOR_EXPLORER, "Stream has started");

    // We want to prevent unnecessary SELECT queries to the database to find
    // the Transaction hash for the Receipt.
    // Later we need to find the Receipt which is a parent to underlying Receipts.
    // Receipt ID will of the child will be stored as key and parent Transaction hash/Receipt ID
    // will be stored as a value
    let receipts_cache: ReceiptsCache =
        std::sync::Arc::new(Mutex::new(SizedCache::with_size(100_000)));

    let handle_messages =
        tokio_stream::wrappers::ReceiverStream::new(stream).map(|streamer_message| {
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
        });
    let mut handle_messages = if let Some(stop_after_n_blocks) = stop_after_number_of_blocks {
        handle_messages
            .take(stop_after_n_blocks.get())
            .boxed_local()
    } else {
        handle_messages.boxed_local()
    }
    .buffer_unordered(usize::from(concurrency.get()));

    while let Some(_handled_message) = handle_messages.next().await {}
    // Graceful shutdown
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Indexer will be shutdown gracefully in 7 seconds...",
    );
    drop(handle_messages);
    tokio::time::sleep(std::time::Duration::from_secs(7)).await;
}

/// Takes `home_dir` and `RunArgs` to build proper IndexerConfig and returns it
async fn construct_near_indexer_config(
    pool: &Database<PgConnection>,
    home_dir: std::path::PathBuf,
    args: configs::RunArgs,
) -> near_indexer::IndexerConfig {
    // Extract await mode to avoid duplication
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "construct_near_indexer_config"
    );
    let sync_mode: near_indexer::SyncModeEnum = match args.sync_mode {
        configs::SyncModeSubCommand::SyncFromInterruption(interruption_args)
            if interruption_args.delta == 1 =>
        {
            info!(target: crate::INDEXER_FOR_EXPLORER, "got from interruption");
            // If delta is 0 we just return IndexerConfig with sync_mode FromInterruption
            // without any changes
            near_indexer::SyncModeEnum::FromInterruption
        }
        configs::SyncModeSubCommand::SyncFromInterruption(interruption_args) => {
            info!(target: crate::INDEXER_FOR_EXPLORER, "got from interruption");
            info!(
                target: crate::INDEXER_FOR_EXPLORER,
                "delta is non zero, calculating..."
            );

            db_adapters::blocks::latest_block_height(pool)
                .await
                .ok()
                .map(|latest_block_height| {
                    if let Some(height) = latest_block_height {
                        near_indexer::SyncModeEnum::BlockHeight(
                            height.saturating_sub(interruption_args.delta),
                        )
                    } else {
                        near_indexer::SyncModeEnum::FromInterruption
                    }
                })
                .unwrap_or_else(|| near_indexer::SyncModeEnum::FromInterruption)
        }
        configs::SyncModeSubCommand::SyncFromBlock(block_args) => {
            near_indexer::SyncModeEnum::BlockHeight(block_args.height)
        }
        configs::SyncModeSubCommand::SyncFromLatest => near_indexer::SyncModeEnum::LatestSynced,
    };

    near_indexer::IndexerConfig {
        home_dir,
        sync_mode,
        await_for_node_synced: if args.stream_while_syncing {
            near_indexer::AwaitForNodeSyncedEnum::StreamWhileSyncing
        } else {
            near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync
        },
    }
}

fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();

    // We establish connection as early as possible as an additional sanity check.
    // Indexer should fail if .env file with credentials is missing/wrong
    let pool = models::establish_connection();

    let opts: Opts = Opts::parse();

    let mut env_filter = EnvFilter::new(
        "tokio_reactor=info,near=info,stats=info,telemetry=info,indexer=info,aggregated=info",
    );

    if opts.debug {
        env_filter = env_filter.add_directive(
            "indexer_for_explorer=debug"
                .parse()
                .expect("Failed to parse directive"),
        );
    } else {
        env_filter = env_filter.add_directive(
            "indexer_for_explorer=info"
                .parse()
                .expect("Failed to parse directive"),
        );
    };

    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        if !rust_log.is_empty() {
            for directive in rust_log.split(',').filter_map(|s| match s.parse() {
                Ok(directive) => Some(directive),
                Err(err) => {
                    eprintln!("Ignoring directive `{}`: {}", s, err);
                    None
                }
            }) {
                env_filter = env_filter.add_directive(directive);
            }
        }
    }

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    let home_dir = opts.home_dir.unwrap_or_else(near_indexer::get_default_home);

    match opts.subcmd {
        SubCommand::Run(args) => {
            tracing::info!(
                target: crate::INDEXER_FOR_EXPLORER,
                "NEAR Indexer for Explorer v{} starting...",
                env!("CARGO_PKG_VERSION")
            );

            let system = actix::System::new();
            system.block_on(async move {
                let indexer_config =
                    construct_near_indexer_config(&pool, home_dir, args.clone()).await;
                let indexer =
                    near_indexer::Indexer::new(indexer_config).expect("Failed to initiate Indexer");
                if args.store_genesis {
                    let near_config = indexer.near_config().clone();
                    db_adapters::genesis::store_genesis_records(pool.clone(), near_config.clone())
                        .await
                        .expect("Failed to store the records from the genesis");
                }

                // Regular indexer process starts here
                let stream = indexer.streamer();

                // Spawning the computation of aggregated data
                aggregated::spawn_aggregated_computations(pool.clone(), &indexer);

                listen_blocks(
                    stream,
                    pool,
                    args.concurrency,
                    !args.non_strict_mode,
                    args.stop_after_number_of_blocks,
                )
                .await;

                actix::System::current().stop();
            });
            system.run().unwrap();
        }
        SubCommand::Init(config) => near_indexer::init_configs(
            &home_dir,
            config.chain_id.as_ref().map(AsRef::as_ref),
            config.account_id.map(|account_id_string| {
                near_indexer::near_primitives::types::AccountId::try_from(account_id_string)
                    .expect("Received accound_id is not valid")
            }),
            config.test_seed.as_ref().map(AsRef::as_ref),
            config.num_shards,
            config.fast,
            config.genesis.as_ref().map(AsRef::as_ref),
            config.download_genesis,
            config.download_genesis_url.as_ref().map(AsRef::as_ref),
            config.download_config,
            config.download_config_url.as_ref().map(AsRef::as_ref),
            config.boot_nodes.as_ref().map(AsRef::as_ref),
            config.max_gas_burnt_view,
        )
        .expect("Failed to initiate config"),
    }
}
