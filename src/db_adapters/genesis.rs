use actix_diesel::Database;
use diesel::PgConnection;

use crate::db_adapters::access_keys::store_access_keys_from_genesis;
use crate::db_adapters::accounts::store_accounts_from_genesis;

/// Iterates over GenesisRecords and stores selected ones (Accounts, AccessKeys)
/// to database.
/// Separately stores records divided in portions by 5000 to optimize
/// memory usage and minimize database queries
pub(crate) async fn store_genesis_records(
    pool: Database<PgConnection>,
    near_config: near_indexer::NearConfig,
) {
    tracing::info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Storing genesis records to database...",
    );
    let tokio_runtime = tokio::runtime::Handle::current();
    let genesis_height = near_config.genesis.config.genesis_height;

    let mut accounts_to_store: Vec<crate::models::accounts::Account> = vec![];
    let mut access_keys_to_store: Vec<crate::models::access_keys::AccessKey> = vec![];

    near_config.genesis.for_each_record(|record| {
        if accounts_to_store.len() == 5_000 {
            let accounts_to_store_copy = accounts_to_store.clone();
            let pool_copy = pool.clone();
            accounts_to_store.clear();
            tokio_runtime.block_on(async move {
                store_accounts_from_genesis(pool_copy, accounts_to_store_copy).await;
            });
        }
        if access_keys_to_store.len() == 5_000 {
            let access_keys_to_store_copy = access_keys_to_store.clone();
            let pool_copy = pool.clone();
            access_keys_to_store.clear();
            tokio_runtime.block_on(async move {
                store_access_keys_from_genesis(pool_copy, access_keys_to_store_copy).await;
            });
        }

        match record {
            near_indexer::near_primitives::state_record::StateRecord::Account {
                account_id,
                ..
            } => {
                accounts_to_store.push(crate::models::accounts::Account::new_from_genesis(
                    &account_id,
                    genesis_height,
                ));
            }
            near_indexer::near_primitives::state_record::StateRecord::AccessKey {
                account_id,
                public_key,
                access_key,
            } => {
                access_keys_to_store.push(crate::models::access_keys::AccessKey::from_genesis(
                    &public_key,
                    &account_id,
                    &access_key,
                    genesis_height,
                ));
            }
            _ => {}
        };
    });

    // Store leftovers vectors if their sizes are less than 5_000
    store_accounts_from_genesis(pool.clone(), accounts_to_store).await;
    store_access_keys_from_genesis(pool, access_keys_to_store).await;

    tracing::info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Genesis records has been stored.",
    );
}
