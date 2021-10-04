use clap::Clap;
use std::convert::TryFrom;
#[macro_use]
extern crate diesel;

use actix_diesel::Database;
use diesel::PgConnection;
use futures::{try_join, StreamExt};
use tokio::sync::mpsc;
use tracing::{info, warn};
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

async fn handle_message(
    pool: &actix_diesel::Database<PgConnection>,
    streamer_message: near_indexer::StreamerMessage,
    strict_mode: bool,
) -> anyhow::Result<()> {
    db_adapters::blocks::store_block(&pool, &streamer_message.block).await?;

    // Chunks
    db_adapters::chunks::store_chunks(
        &pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash,
    )
    .await?;

    // Transaction
    db_adapters::transactions::store_transactions(
        &pool,
        &streamer_message.shards,
        &streamer_message.block.header.hash.to_string(),
        streamer_message.block.header.timestamp,
    )
    .await;

    // Receipts
    for shard in &streamer_message.shards {
        if let Some(chunk) = &shard.chunk {
            db_adapters::receipts::store_receipts(
                &pool,
                &chunk.receipts,
                &streamer_message.block.header.hash.to_string(),
                &chunk.header.chunk_hash,
                streamer_message.block.header.timestamp,
                strict_mode,
            )
            .await?;
        }
    }

    // ExecutionOutcomes
    let execution_outcomes_future = db_adapters::execution_outcomes::store_execution_outcomes(
        &pool,
        &streamer_message.shards,
        streamer_message.block.header.timestamp,
    );

    // Accounts
    let accounts_future = async {
        for shard in &streamer_message.shards {
            db_adapters::accounts::handle_accounts(
                &pool,
                &shard.receipt_execution_outcomes,
                streamer_message.block.header.height,
            )
            .await?;
        }
        Ok(())
    };

    // Assets (NFT)
    let nft_events_future =
        db_adapters::assets::non_fungible_token_events::store_nft(&pool, &streamer_message);

    if strict_mode {
        // AccessKeys
        let access_keys_future = async {
            for shard in &streamer_message.shards {
                db_adapters::access_keys::handle_access_keys(
                    &pool,
                    &shard.receipt_execution_outcomes,
                    streamer_message.block.header.height,
                )
                .await?;
            }
            Ok(())
        };

        // StateChange related to Account
        let account_changes_future = db_adapters::account_changes::store_account_changes(
            &pool,
            &streamer_message.state_changes,
            &streamer_message.block.header.hash,
            streamer_message.block.header.timestamp,
        );

        try_join!(
            execution_outcomes_future,
            accounts_future,
            access_keys_future,
            nft_events_future,
            account_changes_future,
        )?;
    } else {
        try_join!(
            execution_outcomes_future,
            accounts_future,
            nft_events_future
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
    let handle_messages =
        tokio_stream::wrappers::ReceiverStream::new(stream).map(|streamer_message| {
            info!(
                target: crate::INDEXER_FOR_EXPLORER,
                "Block height {}", &streamer_message.block.header.height
            );
            handle_message(&pool, streamer_message, strict_mode)
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

            db_adapters::blocks::latest_block_height(&pool)
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

    let mut env_filter = EnvFilter::new(
        "tokio_reactor=info,near=info,near=error,stats=info,telemetry=info,indexer=info,indexer_for_explorer=info,aggregated=info",
    );

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

    let opts: Opts = Opts::parse();

    let home_dir = opts
        .home_dir
        .unwrap_or_else(|| std::path::PathBuf::from(near_indexer::get_default_home()));

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
                let indexer = near_indexer::Indexer::new(indexer_config);
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
        ),
    }
}
