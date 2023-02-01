use actix_diesel::Database;
use diesel::PgConnection;
use near_chain_configs::{Genesis, GenesisValidationMode};

use crate::adapters::access_keys::store_access_keys_from_genesis;
use crate::adapters::accounts::store_accounts_from_genesis;

pub async fn store_genesis_records(
    pool: &Database<PgConnection>,
    genesis_file_path: String,
) -> anyhow::Result<()> {
    let mut accounts_to_store: Vec<crate::models::accounts::Account> = vec![];
    let mut access_keys_to_store: Vec<crate::models::access_keys::AccessKey> = vec![];

    let genesis = Genesis::from_file(genesis_file_path, GenesisValidationMode::Full);

    let genesis_height = genesis.config.genesis_height;

    for record in genesis.records.0 {
        if accounts_to_store.len() == 5_000 {
            let mut accounts_to_store_chunk = vec![];
            std::mem::swap(&mut accounts_to_store, &mut accounts_to_store_chunk);
            store_accounts_from_genesis(pool, accounts_to_store_chunk).await?;
        }

        if access_keys_to_store.len() == 5_000 {
            let mut access_keys_to_store_chunk = vec![];
            std::mem::swap(&mut access_keys_to_store, &mut access_keys_to_store_chunk);
            store_access_keys_from_genesis(pool, access_keys_to_store_chunk).await?;
        }

        match record {
            near_primitives::state_record::StateRecord::Account { account_id, .. } => {
                accounts_to_store.push(crate::models::accounts::Account::new_from_genesis(
                    &account_id,
                    genesis_height,
                ));
            }
            near_primitives::state_record::StateRecord::AccessKey {
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
    }

    store_accounts_from_genesis(pool, accounts_to_store).await?;
    store_access_keys_from_genesis(pool, access_keys_to_store).await?;

    Ok(())
}
