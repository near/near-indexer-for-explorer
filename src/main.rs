use clap::Clap;
#[macro_use]
extern crate diesel;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use tokio::sync::mpsc;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::configs::{Opts, SubCommand};
use std::convert::TryInto;

mod configs;
mod db_adapters;
mod models;
mod schema;

const INDEXER_FOR_EXPLORER: &str = "indexer_for_explorer";
const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

async fn handle_message(
    pool: std::sync::Arc<Pool<ConnectionManager<PgConnection>>>,
    streamer_message: near_indexer::StreamerMessage,
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
    )
    .await;

    // ExecutionOutcomes
    db_adapters::execution_outcomes::store_execution_outcomes(
        &pool,
        &streamer_message.receipt_execution_outcomes,
    )
    .await;

    // Accounts
    db_adapters::accounts::handle_accounts(&pool, &streamer_message.receipt_execution_outcomes)
        .await;
}

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::StreamerMessage>) {
    let pool = std::sync::Arc::new(models::establish_connection());

    while let Some(streamer_message) = stream.recv().await {
        // Block
        info!(target: "indexer_for_explorer", "Block height {}", &streamer_message.block.header.height);
        actix::spawn(handle_message(pool.clone(), streamer_message));
    }
}

fn main() {
    // We use it to automatically search the for root certificates to perform HTTPS calls
    // (sending telemetry and downloading genesis)
    openssl_probe::init_ssl_cert_env_vars();

    let env_filter = EnvFilter::new(
        "tokio_reactor=info,near=info,near=error,stats=info,telemetry=info,indexer_for_explorer=info",
    );
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
                sync_mode: args.try_into().expect("Error in run arguments"),
            };
            eprintln!("{:#?}", indexer_config);
            let indexer = near_indexer::Indexer::new(indexer_config);
            let stream = indexer.streamer();
            actix::spawn(listen_blocks(stream));
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
