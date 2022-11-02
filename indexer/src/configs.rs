use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

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
    /// AWS Access Key with the rights to read from AWS S3
    #[clap(long, env)]
    pub lake_aws_access_key: String,
    /// AWS Secret Access Key with the rights to read from AWS S3
    #[clap(long, env)]
    pub lake_aws_secret_access_key: String,
    /// Enabled Indexer for Explorer debug level of logs
    #[clap(long)]
    pub debug: bool,
    /// Switches indexer to non-strict mode (skips Receipts without parent Transaction hash, stops storing AccountChanges and AccessKeys)
    #[clap(long)]
    pub non_strict_mode: bool,
    /// Sets the concurrency for indexing. Note: concurrency (set to 2+) may lead to warnings due to tight constraints between transactions and receipts (those will get resolved eventually, but unless it is the second pass of indexing, concurrency won't help at the moment).
    #[clap(long, default_value = "1")]
    pub concurrency: std::num::NonZeroU16,
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

    // Creates AWS Credentials for NEAR Lake
    fn lake_credentials(&self) -> aws_types::credentials::SharedCredentialsProvider {
        let provider = aws_types::Credentials::new(
            self.lake_aws_access_key.clone(),
            self.lake_aws_secret_access_key.clone(),
            None,
            None,
            "alertexer_lake",
        );
        aws_types::credentials::SharedCredentialsProvider::new(provider)
    }

    /// Creates AWS Shared Config for NEAR Lake
    pub fn lake_aws_sdk_config(&self) -> aws_types::sdk_config::SdkConfig {
        aws_types::sdk_config::SdkConfig::builder()
            .credentials_provider(self.lake_credentials())
            .region(aws_types::region::Region::new("eu-central-1"))
            .build()
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
        let s3_config = aws_sdk_s3::config::Builder::from(&self.lake_aws_sdk_config()).build();

        let config_builder = near_lake_framework::LakeConfigBuilder::default().s3_config(s3_config);

        match &self.chain_id {
            ChainId::Mainnet(_) => config_builder
                .mainnet()
                .start_block_height(get_start_block_height(self).await),
            ChainId::Testnet(_) => config_builder
                .testnet()
                .start_block_height(get_start_block_height(self).await),
        }
        .build()
        .expect("Failed to build LakeConfig")
    }
}

async fn get_start_block_height(opts: &Opts) -> u64 {
    match opts.start_options() {
        StartOptions::FromBlock { height } => *height,
        StartOptions::FromInterruption => {
            let pool = crate::models::establish_connection(&opts.database_url);
            let last_indexed_block: u64 = match crate::db_adapters::blocks::latest_block_height(
                &pool,
            )
            .await
            {
                Ok(last_indexed_block) => {
                    if let Some(last_indexed_block) = last_indexed_block {
                        last_indexed_block
                    } else {
                        final_block_height(opts).await
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        target: "alertexer",
                        "Failed to get last indexer block from Database. Failing to the latest one...\n{:#?}",
                        err
                    );
                    final_block_height(opts).await
                }
            };
            last_indexed_block
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

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

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
