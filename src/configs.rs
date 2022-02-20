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
    /// Enabled Indexer for Explorer debug level of logs
    #[clap(long)]
    pub debug: bool,
    /// Switches indexer to non-strict mode (skips Receipts without parent Transaction hash, stops storing AccountChanges and AccessKeys)
    #[clap(long)]
    pub non_strict_mode: bool,
    /// Sets the concurrency for indexing. Note: concurrency (set to 2+) may lead to warnings due to tight constraints between transactions and receipts (those will get resolved eventually, but unless it is the second pass of indexing, concurrency won't help at the moment).
    #[clap(long, default_value = "1")]
    pub concurrency: std::num::NonZeroU16,
    /// Store initial data from genesis like Accounts, AccessKeys
    #[clap(long)]
    pub store_genesis: bool,
    /// AWS S3 bucket name to get the stream from
    #[clap(long)]
    pub s3_bucket_name: String,
    /// AWS S3 bucket region
    #[clap(long)]
    pub s3_region_name: String,
    /// Block height to start the stream from
    #[clap(long, short)]
    pub start_block_height: u64,
}
