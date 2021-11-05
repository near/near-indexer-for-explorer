use actix_diesel::Database;
use diesel::PgConnection;

use crate::db_adapters::access_keys::store_access_keys_from_genesis;
use crate::db_adapters::accounts::store_accounts_from_genesis;

/// This is an ugly hack that allows to execute an async body on a specified actix runtime.
/// You should only call it from a separate thread!
///
/// ```ignore
/// async fn some_async_function() {
///     let current_actix_system = actix::System::current();
///     tokio::tasks::spawn_blocking(move || {
///         let x = vec![0, 1, 2];
///         x.map(|i| {
///             block_on(current_actix_system, async move {
///                 reqwest::get(...).await.text().await
///             })
///         });
///     }
/// }
fn block_on<Fut, T>(
    actix_arbiter: &actix_rt::ArbiterHandle,
    f: Fut,
) -> Result<T, std::sync::mpsc::RecvError>
where
    T: Send + 'static,
    Fut: std::future::Future<Output = T> + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    actix_arbiter.spawn(async move {
        let result = f.await;
        let _ = tx.send(result);
    });
    rx.recv()
}

/// Iterates over GenesisRecords and stores selected ones (Accounts, AccessKeys)
/// to database.
/// Separately stores records divided in portions by 5000 to optimize
/// memory usage and minimize database queries
pub(crate) async fn store_genesis_records(
    pool: Database<PgConnection>,
    near_config: near_indexer::NearConfig,
) -> anyhow::Result<()> {
    tracing::info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Storing genesis records to database...",
    );
    let genesis_height = near_config.genesis.config.genesis_height;

    // Remember the current actix runtime thread in order to be able to
    // schedule async function on it from the thread that processes genesis in
    // a blocking way.
    let actix_system = actix::System::current();
    // Spawn the blocking genesis processing on a separate thread
    tokio::task::spawn_blocking(move || {
        let actix_arbiter = actix_system.arbiter();

        let mut accounts_to_store: Vec<crate::models::accounts::Account> = vec![];
        let mut access_keys_to_store: Vec<crate::models::access_keys::AccessKey> = vec![];

        near_config.genesis.for_each_record(|record| {
            if accounts_to_store.len() == 5_000 {
                let mut accounts_to_store_chunk = vec![];
                std::mem::swap(&mut accounts_to_store, &mut accounts_to_store_chunk);
                let pool = pool.clone();
                block_on(
                    actix_arbiter,
                    store_accounts_from_genesis(pool, accounts_to_store_chunk),
                )
                .expect("storing accounts from genesis failed")
                .expect("storing accounts from genesis failed");
            }
            if access_keys_to_store.len() == 5_000 {
                let mut access_keys_to_store_chunk = vec![];
                std::mem::swap(&mut access_keys_to_store, &mut access_keys_to_store_chunk);
                let pool = pool.clone();
                block_on(
                    actix_arbiter,
                    store_access_keys_from_genesis(pool, access_keys_to_store_chunk),
                )
                .expect("storing access keys from genesis failed")
                .expect("storing access keys from genesis failed");
            }

            match record {
                near_indexer::near_primitives::state_record::StateRecord::Account {
                    account_id,
                    ..
                } => {
                    accounts_to_store.push(crate::models::accounts::Account::new_from_genesis(
                        account_id,
                        genesis_height,
                    ));
                }
                near_indexer::near_primitives::state_record::StateRecord::AccessKey {
                    account_id,
                    public_key,
                    access_key,
                } => {
                    access_keys_to_store.push(crate::models::access_keys::AccessKey::from_genesis(
                        public_key,
                        account_id,
                        access_key,
                        genesis_height,
                    ));
                }
                _ => {}
            };
        });

        let fut = || async move {
            store_accounts_from_genesis(pool.clone(), accounts_to_store).await?;
            store_access_keys_from_genesis(pool, access_keys_to_store).await?;
            anyhow::Result::<()>::Ok(())
        };
        block_on(actix_arbiter, fut())
            .expect("storing leftover accounts and access keys from genesis failed")
            .expect("storing leftover accounts and access keys from genesis failed");
    })
    .await?;

    tracing::info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Genesis records has been stored.",
    );
    Ok(())
}
