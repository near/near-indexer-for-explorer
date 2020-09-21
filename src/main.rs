use actix;

use clap::derive::Clap;
#[macro_use]
extern crate diesel;
use tokio::sync::mpsc;
use tokio_diesel::*;
use tracing::info;
use tracing_subscriber::EnvFilter;

use near_indexer;

use crate::configs::{Opts, SubCommand};

mod configs;
mod models;
mod schema;

async fn listen_blocks(mut stream: mpsc::Receiver<near_indexer::StreamerMessage>) {
    let pool = models::establish_connection();

    while let Some(streamer_message) = stream.recv().await {
        // TODO: handle data as you need
        // Block
        info!(target: "indexer_for_explorer", "Block height {}", &streamer_message.block.header.height);
        match diesel::insert_into(schema::blocks::table)
            .values(models::Block::from_block_view(&streamer_message.block))
            .execute_async(&pool)
            .await
        {
            Ok(_) => {}
            Err(_) => continue,
        };

        // Chunks
        match diesel::insert_into(schema::chunks::table)
            .values(
                streamer_message
                    .chunks
                    .iter()
                    .map(|chunk| models::Chunk::from_chunk_view(streamer_message.block.header.height, chunk))
                    .collect::<Vec<models::Chunk>>(),
            )
            .execute_async(&pool)
            .await
        {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Unable to save chunk, skipping");
            }
        };
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
        SubCommand::Run => {
            let indexer_config = near_indexer::IndexerConfig {
                home_dir,
                sync_mode: near_indexer::SyncModeEnum::FromInterruption,
            };
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
