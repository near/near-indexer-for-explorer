# NEAR Indexer for Explorer

NEAR Indexer for Explorer is built on top of [NEAR Indexer microframework](https://github.com/nearprotocol/nearcore/tree/master/chain/indexer) to watch the network and store all the events in the PostgreSQL database.

## Shared Public Access

NEAR runs the indexer and maintains it for [NEAR Explorer](https://github.com/near/near-explorer), [NEAR Wallet](https://github.com/near/near-wallet), and some other internal services. It proved to be a great source of data for various analysis and services, so we decided to give a shared read-only public access to the data:

* testnet credentials: `postgres://public_readonly:nearprotocol@35.184.214.98/testnet_explorer`
* mainnet credentials: `postgres://public_readonly:nearprotocol@104.199.89.51/mainnet_explorer`

WARNING: We may evolve the data schemas, so make sure you follow the release notes of this repository.

NOTE: Please, keep in mind that the access to the database is shared across everyone in the world, so it is better to make sure you limit the amount of queris and individual queries are efficient.

## Self-hosting

Before you proceed, make sure you have the following software installed:
* [rustup](https://rustup.rs/) or Rust version that is mentioned in `rust-toolchain` file in the root of [nearcore](https://github.com/nearprotocol/nearcore) project.

Install `libpq-dev` dependency

```bash
$ sudo apt install libpq-dev
```

Clone this repository and open the project folder

```bash
$ git clone git@github.com:near/near-indexer-for-explorer.git
$ cd near-indexer-for-explorer
```

You need to provide database credentials in `.env` file like below (replace `user`, `password`, `host` and `db_name` with yours):

```bash
$ echo "DATABASE_URL=postgres://user:password@host/db_name" > .env
```

Then you need to apply migrations to create necessary database structure. For this you'll need `diesel-cli`, you can install it like so:


```bash
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
 - `sync-from-latest` - start indexing blocks from the latest finalized block
 - `sync-from-interruption` - start indexing blocks from the block NEAR Indexer was interrupted last time
 - `sync-from-block --height <block_height>` - start indexing blocks from the specific block height

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


## Creating read-only PostgreSQL user

We highly recommend using a separate read-only user to access the data to avoid unexcepted corruption of the indexed data.

Here's how to create read-only user in PostgreSQL:

```sql
CREATE USER explorer with password 'password';
GRANT CONNECT ON DATABASE databasename TO explorer;
GRANT USAGE ON SCHEMA public TO explorer;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO explorer;
```

```bash
$ PGPASSWORD="password" psql -h 127.0.0.1 -U explorer databasename
```

## Syncing

Whenever you run NEAR Indexer for Explorer for any network except localnet you'll need to sync with the network. This is required because it's a natural behavior of `nearcore` node and NEAR Indexer for Explorer is a wrapper for the regular `nearcore` node. In order to work and index the data your node must be synced with the network. This process can take a while, so we suggest to download a fresh backup of the `data` folder and put it in you `--home-dir` of your choice (by default it is `~/.near`)

Running your NEAR Indexer for Explorer node on top of a backup data will reduce the time of syncing process because your node will download only missing data and it will take reasonable time.

All the backups can be downloaded from the public S3 bucket which contains latest daily snapshots:

* [Mainnet](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/mainnet/rpc/data.tar)
* [Testnet](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/testnet/rpc/data.tar)


## Running NEAR Indexer for Explorer as archival node

It's not necessary but in order to index everything in the network it is better to do it from the genesis. `nearcore` node is running in non-archival mode by default. That means that the node keeps data only for [5 last epochs](https://docs.near.org/docs/concepts/epoch). In order to index data from the genesis we need to turn the node in archival mode.

To do it we need to update `config.json` located in `--home-dir` or your choice (by default it is `~/.near`).

Find next keys in the config and update them as following:

```json
{
  ...
  "archive": true,
  "tracked_shards": [0],
  ...
}
```

The syncing process in archival mode can take a lot of time, so it's better to download a backup provided by NEAR and put it in your `data` folder. After that your node will need to sync only missing data and it should take reasonable time.

All the backups can be downloaded from the public S3 bucket which contains latest daily snapshots:

* [Mainnet](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/mainnet/archive/data.tar)
* [Testnet](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/testnet/archive/data.tar)

See https://docs.near.org/docs/roles/integrator/exchange-integration#running-an-archival-node for reference
