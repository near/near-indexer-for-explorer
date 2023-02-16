use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use explorer_database::{adapters, models};

use near_jsonrpc_client::{methods, JsonRpcClient};
use near_lake_framework::near_indexer_primitives::types::{BlockReference, Finality};

/// NEAR Indexer for Explorer Lake
/// Watches for stream of blocks from the chain
/// built on top of NEAR Lake Framework
#[derive(Parser, Debug)]
#[clap(
    version,
    author,
    about,
    disable_help_subcommand(true),
    propagate_version(true),
    next_line_help(true)
)]
pub(crate) struct Opts {
    /// Connection string to connect to the PostgreSQL Database to fetch AlertRules from
    #[clap(long, env)]
    pub database_url: String,
    /// Enabled Indexer for Explorer debug level of logs
    #[clap(long)]
    pub debug: bool,
    /// Switches indexer to non-strict mode (skips Receipts without parent Transaction hash, stops storing AccountChanges and AccessKeys)
    #[clap(long)]
    pub non_strict_mode: bool,
    /// Sets the concurrency for indexing. Note: concurrency (set to 2+) may lead to warnings due to tight constraints between transactions and receipts (those will get resolved eventually, but unless it is the second pass of indexing, concurrency won't help at the moment).
    #[clap(long, default_value = "1")]
    pub concurrency: std::num::NonZeroU16,
    /// Port to enable metrics/health service
    #[clap(long, short, env, default_value_t = 3030)]
    pub port: u16,
    /// Chain ID: testnet or mainnet
    #[clap(subcommand)]
    pub chain_id: ChainId,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ChainId {
    #[clap(subcommand)]
    Mainnet(StartOptions),
    #[clap(subcommand)]
    Testnet(StartOptions),
}

#[allow(clippy::enum_variant_names)]
#[derive(Subcommand, Debug, Clone)]
pub enum StartOptions {
    /// Start from specific block height
    FromBlock { height: u64 },
    /// Start from interruption (last_indexed_block value from Redis)
    FromInterruption,
    /// Start from the final block on the network (queries JSON RPC for finality: final)
    FromLatest,
}

impl Opts {
    /// Returns [StartOptions] for current [Opts]
    pub fn start_options(&self) -> &StartOptions {
        match &self.chain_id {
            ChainId::Mainnet(start_options) | ChainId::Testnet(start_options) => start_options,
        }
    }

    pub fn rpc_url(&self) -> &str {
        match self.chain_id {
            ChainId::Mainnet(_) => "https://rpc.mainnet.near.org",
            ChainId::Testnet(_) => "https://rpc.testnet.near.org",
        }
    }
}

impl Opts {
    pub async fn to_lake_config(&self) -> near_lake_framework::LakeConfig {
        let config_builder = near_lake_framework::LakeConfigBuilder::default();

        match &self.chain_id {
            ChainId::Mainnet(_) => config_builder.mainnet(),
            ChainId::Testnet(_) => config_builder.testnet(),
        }
        .start_block_height(get_start_block_height(self).await)
        .build()
        .expect("Failed to build LakeConfig")
    }
}

async fn get_start_block_height(opts: &Opts) -> u64 {
    match opts.start_options() {
        StartOptions::FromBlock { height } => *height,
        StartOptions::FromInterruption => {
            let pool = models::establish_connection(&opts.database_url);
            let last_indexed_block = adapters::blocks::latest_block_height(&pool)
                .await
                .expect("Failed to get last indexer block from Database");
            if let Some(last_indexed_block) = last_indexed_block {
                // -500 helps us to be sure we haven't missed anything
                last_indexed_block.saturating_sub(500)
            } else {
                tracing::warn!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "It seems the database is empty. Will start indexing from the beginning",
                );
                0 // S3 should return the first available block_height
            }
        }
        StartOptions::FromLatest => final_block_height(opts).await,
    }
}

pub(crate) fn init_tracing(debug: bool) -> anyhow::Result<()> {
    let mut env_filter =
        EnvFilter::new("near_lake_framework=info,indexer_for_explorer=info,stats=info");

    if debug {
        env_filter = env_filter
            .add_directive("indexer_for_explorer=debug".parse()?)
            .add_directive("near_lake_framework=debug".parse()?);
    }

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

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr);

    if std::env::var("ENABLE_JSON_LOGS").is_ok() {
        subscriber.json().init();
    } else {
        subscriber.compact().init();
    }

    Ok(())
}

async fn final_block_height(opts: &Opts) -> u64 {
    let client = JsonRpcClient::connect(opts.rpc_url());
    let request = methods::block::RpcBlockRequest {
        block_reference: BlockReference::Finality(Finality::Final),
    };

    let latest_block = client.call(request).await.unwrap();

    latest_block.header.height
}
