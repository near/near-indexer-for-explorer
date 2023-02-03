# NEAR Indexer for Explorer

NEAR Indexer for Explorer is built on top of [NEAR Lake Framework](https://github.com/near/near-lake-framework-rs) to watch the network and store all the events in the PostgreSQL database.

## Shared Public Access

NEAR runs the indexer and maintains it for [NEAR Explorer](https://github.com/near/near-explorer), [NEAR Wallet](https://github.com/near/near-wallet), and some other internal services. It proved to be a great source of data for various analysis and services, so we decided to give a shared read-only public access to the data:

* testnet credentials: `postgres://public_readonly:nearprotocol@testnet.db.explorer.indexer.near.dev/testnet_explorer`
* mainnet credentials: `postgres://public_readonly:nearprotocol@mainnet.db.explorer.indexer.near.dev/mainnet_explorer`

WARNING: We may evolve the data schemas, so make sure you follow the release notes of this repository.

NOTE: Please, keep in mind that the access to the database is shared across everyone in the world, so it is better to make sure you limit the amount of queries and individual queries are efficient.

## Self-hosting

The final setup consists of the following components:
* PostgreSQL database (you can run it locally or in the cloud), which can hold the whole history of the blockchain (as of August 2022, mainnet takes 3TB of data in PostgreSQL storage, and testnet takes 1TB)
* NEAR Indexer for Explorer binary that operates as a NEAR Lake Framework based indexer, it requires [AWS S3 credentials](https://docs.near.org/tutorials/indexer/credentials)

### Prepare Development Environment

Before you proceed, make sure you have the following software installed:
* [Rust compiler](https://rustup.rs/) of the version that is mentioned in `rust-toolchain` file in the root of [nearcore](https://github.com/nearprotocol/nearcore) project.
* `libpq-dev` dependency

    On Debian/Ubuntu:
    
    ```bash
    $ sudo apt install libpq-dev
    ```


### Prepare Database

Setup PostgreSQL database, create a database with the regular tools, and note the connection string (database host, credentials, and the database name).

Clone this repository and open the project folder

```bash
$ git clone https://github.com/near/near-indexer-for-explorer.git
$ cd near-indexer-for-explorer
```

You need to provide credentials via `.env` file for:
- database

  (replace `user`, `password`, `host` and `db_name` with yours)
  ```bash
  $ echo "DATABASE_URL=postgres://user:password@host/db_name" > .env
  ```
- AWS S3 (permission to read from buckets):
  ```bash
  $ echo "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE" >> .env
  $ echo "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY" >> .env
  ```

Then you need to apply migrations to create necessary database structure. For this you'll need `diesel-cli`, you can install it like so:


```bash
$ cargo install diesel_cli --no-default-features --features "postgres"
```

And apply migrations

```bash
$ cd database && diesel migration run
```

If you have the DB with some data collected, and you need to apply the next migration, we highly recommend to read the migration contents.  
Some migrations have the explanations what should be done, e.g. [[1]](database/migrations/2021-08-06-123500_account_changes_ordering_column/up.sql), [[2]](database/migrations/2023-02-02-100000_fungible_token_events_pk_changed/up.sql), [[3]](database/migrations/2023-02-02-110000_non_fungible_token_events_pk_changed/up.sql).  
General advice is to add [`CONCURRENTLY` option](https://www.postgresql.org/docs/current/sql-createindex.html#SQL-CREATEINDEX-CONCURRENTLY) to all indexes creation and apply such changes manually.

### Compile NEAR Indexer for Explorer

```bash
$ cargo build --release
```

### Run NEAR Indexer for Explorer

Command to run NEAR Indexer for Explorer have to include the chain-id and start options:

You can choose NEAR Indexer for Explorer start options:
 - `from-latest` - start indexing blocks from the latest finalized block
 - `from-interruption` - start indexing blocks from the block NEAR Indexer was interrupted last time but earlier for `<number_of_blocks>` if provided
 - `from-block --height <block_height>` - start indexing blocks from the specific block height

#### Storing genesis file
Unlike the original NEAR Indexer for Explorer you **can't** tell Indexer to store data from genesis (Accounts and Access Keys) by adding key `--store-genesis` to the `run` command. So please, ensure you took care about the genesis data in your database in order this indexer to work properly. This capability will be implemented eventually, it's progress can be tracked here: #327.

#### Strict mode
NEAR Indexer for Explorer works in strict mode by default. In strict mode, the Indexer will ensure parent data exists before storing children, infinitely retrying until this condition is met. This is necessary as a parent (i.e. `block`) may still be processing while a child (i.e. `receipt`) is ready to be stored. This scenario will likely occur if you have not stored the genesis file or do not have all data prior to the block you start indexing from. In this case, you can disable strict mode to store data prior to the block you are concerned about, and then re-enable it once you have passed this block.

To disable strict mode provide the following command arugment:

```
--non-strict-mode
```

#### Concurrency
By default NEAR Indexer for Explorer processes only a single block at a time. You can adjust this with the `--concurrency` argument (when the blocks are mostly empty, it is fine to go with as many as 100 blocks of concurrency).

#### Starting
So final command to run NEAR Indexer for Explorer can look like:

```bash
$ ./target/release/indexer-explorer \
  --non-strict-mode \
  --concurrency 1 \
  mainnet \
  from-latest
```

After the network is synced, you should see logs of every block height currently received by NEAR Indexer for Explorer.

### Troubleshoot NEAR Indexer for Explorer

Refer to a separate [TROBLESHOOTING.md](./TROBLESHOOTING.md) document.

## Database structure

![database structure](docs/near-indexer-for-explorer-db.png)


## Creating read-only PostgreSQL user

We highly recommend using a separate read-only user to access the data to avoid unexcepted corruption of the indexed data.

We use `public` schema for all tables. By default, new users have the possibility to create new tables/views/etc there. If you want to restrict that, you have to revoke these rights:

```sql
REVOKE CREATE ON SCHEMA PUBLIC FROM PUBLIC;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA PUBLIC FROM PUBLIC;
ALTER DEFAULT PRIVILEGES IN SCHEMA PUBLIC GRANT SELECT ON TABLES TO PUBLIC;
```

After that, you could create read-only user in PostgreSQL:

```sql
CREATE ROLE readonly;
GRANT USAGE ON SCHEMA public TO readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public to readonly;
-- Put here your limit or just ignore this command
ALTER ROLE readonly SET statement_timeout = '30s';

CREATE USER explorer with login password 'password';
GRANT readonly TO explorer;
```

```bash
$ PGPASSWORD="password" psql -h 127.0.0.1 -U explorer databasename
```

## Deployments
Both `indexer-explorer` and `circulating-supply` binaries are run within Docker, their `Dockerfile`s can be found within their respective directoires/workspaces. Docker images are built using Google Cloud Build and then deployed to Google Cloud Run. The following commands can be used to build the Docker images:

```bash
$ docker build -f ./indexer/Dockerfile .
$ docker build -f ./circulating-supply/Dockerfile .
```
