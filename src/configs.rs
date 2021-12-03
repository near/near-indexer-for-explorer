use std::convert::TryFrom;

use clap::Parser;

/// NEAR Indexer for Explorer
/// Watches for stream of blocks from the chain
#[derive(Parser, Debug)]
#[clap(
    version,
    author,
    about,
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::PropagateVersion),
    setting(clap::AppSettings::NextLineHelp)
)]
pub(crate) struct Opts {
    /// Sets a custom config dir. Defaults to ~/.near/
    #[clap(short, long)]
    pub home_dir: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum SubCommand {
    /// Run NEAR Indexer Example. Start observe the network
    Run(RunArgs),
    /// Initialize necessary configs
    Init(InitConfigArgs),
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct RunArgs {
    /// Store initial data from genesis like Accounts, AccessKeys
    #[clap(long)]
    pub store_genesis: bool,
    /// Force streaming while node is syncing
    #[clap(long)]
    pub stream_while_syncing: bool,
    /// Switches indexer to non-strict mode (skips Receipts without parent Transaction hash, stops storing AccountChanges and AccessKeys)
    #[clap(long)]
    pub non_strict_mode: bool,
    /// Stops indexer completely after indexing the provided number of blocks
    #[clap(long, short)]
    pub stop_after_number_of_blocks: Option<std::num::NonZeroUsize>,
    /// Sets the concurrency for indexing. Note: concurrency (set to 2+) may lead to warnings due to tight constraints between transactions and receipts (those will get resolved eventually, but unless it is the second pass of indexing, concurrency won't help at the moment).
    #[clap(long, default_value = "1")]
    pub concurrency: std::num::NonZeroU16,
    #[clap(subcommand)]
    pub sync_mode: SyncModeSubCommand,
}

#[allow(clippy::enum_variant_names)] // we want commands to be more explicit
#[derive(Parser, Debug, Clone)]
pub(crate) enum SyncModeSubCommand {
    /// continue from the block Indexer was interrupted
    SyncFromInterruption(InterruptionArgs),
    /// start from the newest block after node finishes syncing
    SyncFromLatest,
    /// start from specified block height
    SyncFromBlock(BlockArgs),
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct InterruptionArgs {
    /// start indexing this number of blocks earlier than the actual interruption happened
    #[clap(long, default_value = "0")]
    pub delta: u64,
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct BlockArgs {
    /// block height for block sync mode
    #[clap(long)]
    pub height: u64,
}

impl TryFrom<SyncModeSubCommand> for near_indexer::SyncModeEnum {
    type Error = &'static str;

    fn try_from(sync_mode: SyncModeSubCommand) -> Result<Self, Self::Error> {
        match sync_mode {
            SyncModeSubCommand::SyncFromInterruption(_) => Err("Unable to convert SyncFromInterruption variant because it has additional parameter which is not acceptable by near_indexer::SyncModeEnum::SyncFromInterruption"),
            SyncModeSubCommand::SyncFromLatest => Ok(Self::LatestSynced),
            SyncModeSubCommand::SyncFromBlock(args) => Ok(Self::BlockHeight(args.height)),
        }
    }
}

#[derive(Parser, Debug)]
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
    /// Download the verified NEAR config file automatically.
    #[clap(long)]
    pub download_config: bool,
    #[clap(long)]
    pub download_config_url: Option<String>,
    /// Download the verified NEAR genesis file automatically.
    #[clap(long)]
    pub download_genesis: bool,
    /// Specify a custom download URL for the genesis-file.
    #[clap(long)]
    pub download_genesis_url: Option<String>,
    /// Customize max_gas_burnt_view runtime limit.  If not specified, value
    /// from genesis configuration will be taken.
    #[clap(long)]
    pub max_gas_burnt_view: Option<u64>,
    /// Initialize boots nodes in <node_key>@<ip_addr> format seperated by commas
    /// to bootstrap the network and store them in config.json
    #[clap(long)]
    pub boot_nodes: Option<String>,
}
