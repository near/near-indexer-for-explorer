use std::convert::TryInto;

use clap::Clap;
#[macro_use]
extern crate diesel;
use diesel::PgConnection;
use futures::{join, StreamExt};
use tokio::sync::mpsc;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::configs::{Opts, SubCommand};

mod configs;
mod db_adapters;
mod models;
mod schema;

const INDEXER_FOR_EXPLORER: &str = "indexer_for_explorer";
const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

async fn handle_message(
    pool: actix_diesel::Database<PgConnection>,
    streamer_message: near_indexer::StreamerMessage,
    strict_mode: bool,
) {
    db_adapters::blocks::store_block(&pool, &streamer_message.block).await;

    // Chunks
    db_adapters::chunks::store_chunks(
        &pool,
        &streamer_message.chunks,
        &streamer_message.block.header.hash,
    )
    .await;

    // Transaction
    db_adapters::transactions::store_transactions(
        &pool,
        &streamer_message.chunks,
        &streamer_message.block.header.hash.to_string(),
        streamer_message.block.header.timestamp,
    )
    .await;

    // Receipts
    for chunk in &streamer_message.chunks {
        db_adapters::receipts::store_receipts(
            &pool,
            &chunk.receipts,
            &streamer_message.block.header.hash.to_string(),
            &chunk.header.chunk_hash,
            streamer_message.block.header.timestamp,
            strict_mode,
        )
        .await;
    }

    // ExecutionOutcomes
    let execution_outcomes_future = db_adapters::execution_outcomes::store_execution_outcomes(
        &pool,
        &streamer_message.chunks,
        streamer_message.block.header.timestamp,
    );

    // Accounts
    let accounts_future = async {
        for chunk in &streamer_message.chunks {
            db_adapters::accounts::handle_accounts(
                &pool,
                &chunk.receipt_execution_outcomes,
                streamer_message.block.header.height,
            )
            .await;
        }
    };

    // AccessKeys
    let access_keys_future = async {
        for chunk in &streamer_message.chunks {
            db_adapters::access_keys::handle_access_keys(
                &pool,
                &chunk.receipt_execution_outcomes,
                streamer_message.block.header.height,
            )
            .await;
        }
    };

    // StateChange related to Account
    let account_changes_future = db_adapters::account_changes::store_account_changes(
        &pool,
        &streamer_message.state_changes,
        &streamer_message.block.header.hash,
        streamer_message.block.header.timestamp,
    );

    join!(
        execution_outcomes_future,
        accounts_future,
        access_keys_future,
        account_changes_future,
    );
}

async fn listen_blocks(
    stream: mpsc::Receiver<near_indexer::StreamerMessage>,
    allow_missing_relation_in_start_blocks: Option<u32>,
) {
    let pool = models::establish_connection();
    let strict_mode = allow_missing_relation_in_start_blocks.unwrap_or(0);
    let mut handle_messages = tokio_stream::wrappers::ReceiverStream::new(stream)
        .enumerate()
        .map(|(index, streamer_message)| {
            info!(target: "indexer_for_explorer", "Block height {}", &streamer_message.block.header.height);
            handle_message(
                pool.clone(),
                streamer_message,
                index >= strict_mode as usize,
            )
        })
        .buffer_unordered(100);

    while let Some(_handled_message) = handle_messages.next().await {}
}

fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();

    let mut env_filter = EnvFilter::new(
        "tokio_reactor=info,near=info,near=error,stats=info,telemetry=info,indexer_for_explorer=info",
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
            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                sync_mode: args.clone().try_into().expect("Error in run arguments"),
                await_for_node_synced: if args.stream_while_syncing {
                    near_indexer::AwaitForNodeSyncedEnum::StreamWhileSyncing
                } else {
                    near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync
                },
            };
            actix::System::builder()
                .stop_on_panic(true)
                .run(move || {
                    let indexer = near_indexer::Indexer::new(indexer_config);
                    if args.store_genesis {
                        let near_config = indexer.near_config().clone();
                        actix::spawn(db_adapters::accounts::store_accounts_from_genesis(
                            near_config.clone(),
                        ));
                        actix::spawn(db_adapters::access_keys::store_access_keys_from_genesis(
                            near_config,
                        ))
                    }
                    let stream = indexer.streamer();
                    actix::spawn(listen_blocks(
                        stream,
                        args.allow_missing_relations_in_first_blocks,
                    ));
                })
                .unwrap();
        }
        SubCommand::Init(config) => near_indexer::init_configs(
            &home_dir,
            config.chain_id.as_ref().map(AsRef::as_ref),
            config.account_id.as_ref().map(AsRef::as_ref),
            config.test_seed.as_ref().map(AsRef::as_ref),
            config.num_shards,
            config.fast,
            config.genesis.as_ref().map(AsRef::as_ref),
            config.download,
            config.download_genesis_url.as_ref().map(AsRef::as_ref),
        ),
    }
}
