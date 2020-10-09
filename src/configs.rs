use std::convert::TryFrom;
use std::str::FromStr;

use clap::Clap;

/// NEAR Indexer for Explorer
/// Watches for stream of blocks from the chain
#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Near Inc. <hello@nearprotocol.com>")]
pub(crate) struct Opts {
    /// Sets a custom config dir. Defaults to ~/.near/
    #[clap(short, long)]
    pub home_dir: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, Debug)]
pub(crate) enum SubCommand {
    /// Run NEAR Indexer Example. Start observe the network
    Run(RunArgs),
    /// Initialize necessary configs
    Init(InitConfigArgs),
}

#[derive(Clap, Debug)]
pub(crate) struct RunArgs {
    /// streamer SyncMode. Possible values
    #[clap(long, default_value = "from-interruption")]
    pub sync_mode: SyncMode,
    /// block height for block sync mode
    #[clap(long)]
    pub height: Option<u64>,
}

#[derive(Clap, Debug)]
pub(crate) enum SyncMode {
    /// continue from the block Indexer was interrupted
    FromInterruption,
    /// start from the newest block after node finishes syncing
    LastSynced,
    /// start from specified block height
    Block,
}

impl FromStr for SyncMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "from-interruption" => Ok(Self::FromInterruption),
            "last-synced" => Ok(Self::LastSynced),
            "block" => Ok(Self::Block),
            _ => Err("Not allowed value for sync-mode"),
        }
    }
}

impl TryFrom<RunArgs> for near_indexer::SyncModeEnum {
    type Error = &'static str;

    fn try_from(run_args: RunArgs) -> Result<Self, Self::Error> {
        match run_args.sync_mode {
            SyncMode::FromInterruption => Ok(Self::FromInterruption),
            SyncMode::LastSynced => Ok(Self::LatestSynced),
            SyncMode::Block => {
                Ok(Self::BlockHeight(run_args.height.expect(
                    "--height must be provided to use block sync-mode",
                )))
            }
        }
    }
}

#[derive(Clap, Debug)]
pub(crate) struct InitConfigArgs {
    /// chain/network id (localnet, testnet, devnet, betanet)
    #[clap(short, long)]
    pub chain_id: Option<String>,
    /// Account ID for the validator key
    #[clap(long)]
    pub account_id: Option<String>,
    /// Specify private key generated from seed (TESTING ONLY)
    #[clap(long)]
    pub test_seed: Option<String>,
    /// Number of shards to initialize the chain with
    #[clap(short, long, default_value = "1")]
    pub num_shards: u64,
    /// Makes block production fast (TESTING ONLY)
    #[clap(short, long)]
    pub fast: bool,
    /// Genesis file to use when initialize testnet (including downloading)
    #[clap(short, long)]
    pub genesis: Option<String>,
    #[clap(short, long)]
    /// Download the verified NEAR genesis file automatically.
    pub download: bool,
    /// Specify a custom download URL for the genesis-file.
    #[clap(long)]
    pub download_genesis_url: Option<String>,
}
