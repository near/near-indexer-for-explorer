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
$ diesel migration run
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
 - `sync-from-interruption --delta <number_of_blocks>` - start indexing blocks from the block NEAR Indexer was interrupted last time but earlier for `<number_of_blocks>` if provided
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

Sometimes you may want to index block while sync process is happening, by default an indexer node is waiting for full sync to complete but you can enable indexing while the node is syncing by passing `--stream-while-syncing`

By default NEAR Indexer for Explorer processes only a single block at a time. You can adjust this with the `--concurrency` argument (when the blocks are mostly empty, it is fine to go with as many as 100 blocks of concurrency).

So final command to run NEAR Indexer for Explorer can look like:

```bash
$ cargo run --release -- --home-dir ~/.near/testnet run --store-genesis --stream-while-syncing --allow-missing-relations-in-first-blocks 1000 --concurrency 1 sync-from-latest
```

After the network is synced, you should see logs of every block height currently received by NEAR Indexer for Explorer.

## Database structure

![database structure](docs/near-indexer-for-explorer-db.png)


## Creating read-only PostgreSQL user

We highly recommend using a separate read-only user to access the data to avoid unexcepted corruption of the indexed data.

We use `public` schema for all tables. By default, new users have the possibility to create new tables/views/etc there. If you want to restrict that, you have to revoke these rights:

```sql
REVOKE CREATE ON SCHEMA PUBLIC FROM PUBLIC;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA PUBLIC FROM PUBLIC;
```

After that, you could create read-only user in PostgreSQL:

```sql
CREATE USER explorer with password 'password';
GRANT readonly TO explorer;
```

```bash
$ PGPASSWORD="password" psql -h 127.0.0.1 -U explorer databasename
```

## Syncing

Whenever you run NEAR Indexer for Explorer for any network except localnet you'll need to sync with the network. This is required because it's a natural behavior of `nearcore` node and NEAR Indexer for Explorer is a wrapper for the regular `nearcore` node. In order to work and index the data your node must be synced with the network. This process can take a while, so we suggest to download a fresh backup of the `data` folder and put it in you `--home-dir` of your choice (by default it is `~/.near`)

Running your NEAR Indexer for Explorer node on top of a backup data will reduce the time of syncing process because your node will download only missing data and it will take reasonable time.

All the backups can be downloaded from the public S3 bucket which contains latest daily snapshots:

* [Recent 5-epoch Mainnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/mainnet/rpc/data.tar)
* [Recent 5-epoch Testnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/testnet/rpc/data.tar)


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

* [Archival Mainnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/mainnet/archive/data.tar)
* [Archival Testnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/testnet/archive/data.tar)

See https://docs.near.org/docs/roles/integrator/exchange-integration#running-an-archival-node for reference

## Local debugging

If you want to play with the code locally, it's better not to copy existing mainnet/testnet (it requires LOTS of memory), but to have your own small example.
You need to have empty DB (we suggest to use [Docker](https://hub.docker.com/_/postgres) for that).
Go through steps [above](https://github.com/near/near-indexer-for-explorer#self-hosting) until (including) diesel migration.
Then,

```bash
$ cargo run --release -- --home-dir ~/.near/localnet init --chain-id localnet
```

Edit `~/.near/localnet/config.json` by adding tracking shards and archiving option (see [example above](https://github.com/near/near-indexer-for-explorer#running-near-indexer-for-explorer-as-archival-node)).

```bash
$ cargo run -- --home-dir ~/.near/localnet run --store-genesis sync-from-latest
```

Congrats, the blocks are being produced right now!
There should be some lines in the DB.
Now, we need to generate some activity to add new examples.

```bash
$ npm i -g near-cli
$ NEAR_ENV=local near create-account awesome.test.near --initialBalance 30 --masterAccount test.near --keyPath=~/.near/localnet/validator_key.json
$ NEAR_ENV=local near send test.near awesome.test.near 5
```

All available commands are [here](https://github.com/near/near-cli#near-cli-command-line-interface).

You can stop and re-run the example at any time.
Blocks will continue producing from the last state.

## Troubleshooting

When operating normally, you should see "INFO indexer_for_explorer: Block height ..." messages in the logs.

### The node is fully synced and running, but no indexer messages and no transactions in the database (not indexing)

Make sure the blocks you want to save exist on the node. Check them via [JSON RPC](https://docs.near.org/docs/api/rpc#block-details):

```
curl http://127.0.0.1:3030/ -X POST --header 'Content-type: application/json' --data '{"jsonrpc": "2.0", "id": "dontcare", "method": "block", "params": {"block_id": 9820214}}'
```

NOTE: Block #9820214 is the first block after genesis block (#9820210) on Mainnet.

If it returns an error that the block does not exist or missing, it means that your node does not have the necessary data. Your options here are to start from the blocks that are recorded on the node or start an archival node (see above) and make sure you have the full network history (either use a backup or let the node sync from scratch (it is quite slow, so backup is recommended))
