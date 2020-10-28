use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use futures::future::join_all;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use crate::models;
use crate::schema;

/// Saves Transaction to database
pub(crate) async fn store_transactions(
    pool: &Pool<ConnectionManager<PgConnection>>,
    chunks: &[near_indexer::IndexerChunkView],
    block_hash: &str,
) {
    if chunks.is_empty() { return; }
    let futures = chunks.iter().map(|chunk| {
        store_chunk_transactions(
            &pool,
            chunk
                .transactions
                .iter()
                .collect::<Vec<&near_indexer::IndexerTransactionWithOutcome>>(),
            &chunk.header.chunk_hash,
            block_hash,
        )
    });

    join_all(futures).await;
}

async fn store_chunk_transactions(
    pool: &Pool<ConnectionManager<PgConnection>>,
    transactions: Vec<&near_indexer::IndexerTransactionWithOutcome>,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_hash: &str,
) {
    let transaction_models: Vec<models::transactions::Transaction> = transactions
        .iter()
        .map(|tx| {
            models::transactions::Transaction::from_indexer_transaction(tx, block_hash, chunk_hash)
        })
        .collect();

    loop {
        match diesel::insert_into(schema::transactions::table)
            .values(transaction_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while Transaction were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &transaction_models
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }

    let transaction_action_models: Vec<models::TransactionAction> = transactions
        .into_iter()
        .flat_map(|tx| {
            tx.transaction
                .actions
                .iter()
                .enumerate()
                .map(move |(index, action)| {
                    models::transactions::TransactionAction::from_action_view(
                        tx.transaction.hash.to_string(),
                        index as i32,
                        action,
                    )
                })
        })
        .collect();

    loop {
        match diesel::insert_into(schema::transaction_actions::table)
            .values(transaction_action_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while TransactionAction were adding to database. Retrying in {} milliseconds... \n {:#?} \n{:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &transaction_action_models,
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }
}
