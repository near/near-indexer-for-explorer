use std::convert::TryInto;

use clap::Clap;
#[macro_use]
extern crate diesel;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use futures::join;
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

async fn handle_message(
    pool: std::sync::Arc<Pool<ConnectionManager<PgConnection>>>,
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
    )
    .await;

    // Receipts
    let receipts: Vec<&near_indexer::near_primitives::views::ReceiptView> = streamer_message
        .chunks
        .iter()
        .flat_map(|chunk| &chunk.receipts)
        .chain(streamer_message.local_receipts.iter())
        .collect();

    db_adapters::receipts::store_receipts(
        &pool,
        receipts,
        &streamer_message.block.header.hash.to_string(),
        strict_mode,
    )
    .await;

    // ExecutionOutcomes
    let execution_outcomes_future = db_adapters::execution_outcomes::store_execution_outcomes(
        &pool,
        &streamer_message.receipt_execution_outcomes,
    );

    // Accounts
    let accounts_future =
        db_adapters::accounts::handle_accounts(&pool, &streamer_message.receipt_execution_outcomes);

    // AccessKeys
    let access_keys_future = db_adapters::access_keys::handle_access_keys(
        &pool,
        &streamer_message.receipt_execution_outcomes,
    );

    join!(
        execution_outcomes_future,
        accounts_future,
        access_keys_future
    );
}

async fn listen_blocks(
    mut stream: mpsc::Receiver<near_indexer::StreamerMessage>,
    allow_missing_relation_in_start_blocks: Option<u32>,
) {
    let pool = std::sync::Arc::new(models::establish_connection());
    let mut strict_mode = allow_missing_relation_in_start_blocks.unwrap_or_else(|| 0);
    while let Some(streamer_message) = stream.recv().await {
        // Block
        info!(target: "indexer_for_explorer", "Block height {}", &streamer_message.block.header.height);
        actix::spawn(handle_message(
            pool.clone(),
            streamer_message,
            strict_mode == 0,
        ));
        strict_mode = strict_mode.saturating_sub(1);
    }
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
            };
            let indexer = near_indexer::Indexer::new(indexer_config);
            if args.store_genesis {
                let near_config = indexer.near_config().clone();
                actix::spawn(db_adapters::accounts::store_accounts_from_genesis(
                    near_config,
                ));
            }
            let stream = indexer.streamer();
            actix::spawn(listen_blocks(
                stream,
                args.allow_missing_relations_in_first_blocks,
            ));
            indexer.start();
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
