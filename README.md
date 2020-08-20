# NEAR Indexer for Explorer

NEAR Indexer for Explorer is built on top of [NEAR Indexer microframework](https://github.com/nearprotocol/nearcore/tree/master/chain/indexer) to watch the network and store all the events in the PostgreSQL database.


## Getting started

Before you proceed, make sure you have the following software installed:
* [rustup](https://rustup.rs/) or Rust version that is mentioned in `rust-toolchain` file in the root of [nearcore](https://github.com/nearprotocol/nearcore) project.

Clone this repository and open the project folder

```bash
$ git clone git@github.com:near/near-indexer-for-explorer.git
$ cd near-indexer-for-explorer
```

You need to provide database credentials in `.env` file like below (replace `user`, `password`, `host` and `db_name` with yours):

```bash
$ echo "DATABASE_URL=postgres://user:password@host/db_name" > .env
```

Then you need to apply migrations to create necessary database structure, for this you'll need `diesel-cli`, you can install it like so:

```bash
$ cargo install diesel_cli --no-default-features --features "postgres"
```

And apply migrations

```bash
$ diesel migation run
```

To connect NEAR Indexer for Wallet to the specific chain you need to have necessary configs, you can generate it as follows:

```bash
$ cargo run --release -- --home-dir ~/.near/testnet init --chain-id testnet --download
```

Replace `testnet` in the command above to choose different chain: `betanet` or `mainnet`.
This will generate keys and configs and download official genesis config.

Configs for the specified network are in the `--home-dir` provided folder. We need to ensure that NEAR Indexer for Explorer follows
all the necessary shards, so `"tracked_shards"` parameters in `~/.near/testnet/config.json` needs to be configured properly.
For example, with a single shared network, you just add the shard #0 to the list:

```
...
"tracked_shards": [0],
...
```

To run NEAR Indexer for Explorer:

```bash
$ cargo run --release -- --home-dir ~/.near/testnet run
```

After the network is synced, you should see logs of every block height currently received by NEAR Indexer for Explorer.
