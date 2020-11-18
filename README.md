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

Then you need to apply migrations to create necessary database structure. For this you'll need `diesel-cli`, you can install it like so (you need to install `libpq-dev` dependency):


```bash
$ sudo apt install libpq-dev
$ cargo install diesel_cli --no-default-features --features "postgres"
```

And apply migrations

```bash
$ diesel migation run
```

To connect NEAR Indexer for Explorer to the specific chain you need to have necessary configs, you can generate it as follows:

```bash
$ cargo run --release -- --home-dir ~/.near/testnet init --chain-id testnet --download
```

The above code will download the official genesis config and generate necessary configs. You can replace `testnet` in the command above to different network ID (`betanet`, `mainnet`).

**NB!** According to changes in `nearcore` config generation we don't fill all the necessary fields in the config file. 
While this issue is open https://github.com/nearprotocol/nearcore/issues/3156 you need to download config you want and replace the generated one manually.
 - [testnet config.json](https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/testnet/config.json)
 - [betanet config.json](https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/betanet/config.json)
 - [mainnet config.json](https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/mainnet/config.json)

Configs for the specified network are in the `--home-dir` provided folder. We need to ensure that NEAR Indexer for Explorer follows
all the necessary shards, so `"tracked_shards"` parameters in `~/.near/testnet/config.json` needs to be configured properly.
For example, with a single shared network, you just add the shard #0 to the list:

```
...
"tracked_shards": [0],
...
```

## Running NEAR Indexer for Explorer:

Command to run NEAR Indexer for Explorer have to contain sync mode.

You can choose NEAR Indexer for Explorer sync mode by setting what to stream:
 - `sync-from-latest` - Real-time syncing, always taking the latest finalized block to stream
 - `sync-from-interruption` - Starts syncing from the block NEAR Indexer was interrupted last time 
 - `sync-from-block --height <block_height>` - Specific block height to start syncing from
 
Optionally you can tell Indexer to store data from genesis (Accounts and Access Keys) by adding key `--store-genesis` to the `run` command.

NEAR Indexer for Explorer works in strict mode by default, but you can disable it for specific amount of blocks. The strict mode means that every piece of data
will be retried to store to database in case of error. Errors may occur when the parent piece of data is still processed but the child piece is already 
trying to be stored. So Indexer keeps retrying to store the data until success. However if you're running Indexer not from the genesis it is possible that you
really miss some of parent data and it'll be impossible to store child one, so you can disable strict mode for 1000 blocks to ensure you've passed the strong
relation data area and you're running Indexer where it is impossible to loose any piece of data.

To disable strict mode you need to provide:

```
--allow-missing-relations-in-first-blocks <amount of blocks>
```

So final command to run NEAR Indexer for Explorer can look like:

```bash
$ cargo run --release -- --home-dir ~/.near/testnet run --store-genesis --allow-missing-relations-in-first-blocks 1000 sync-from-latest
```

After the network is synced, you should see logs of every block height currently received by NEAR Indexer for Explorer.

## Database structure

![database structure](docs/near-indexer-for-explorer-db.png)
