use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::PgConnection;
use futures::future::join_all;

use crate::models;
use crate::schema;

/// Saves Transaction to database
pub(crate) async fn store_transactions(
    pool: &actix_diesel::Database<PgConnection>,
    shards: &[near_indexer::IndexerShard],
    block_hash: &str,
    block_timestamp: u64,
) {
    if shards.is_empty() {
        return;
    }

    let futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| {
            store_chunk_transactions(
                pool,
                chunk
                    .transactions
                    .iter()
                    .collect::<Vec<&near_indexer::IndexerTransactionWithOutcome>>(),
                &chunk.header.chunk_hash,
                block_hash,
                block_timestamp,
            )
        });

    join_all(futures).await;
}

async fn store_chunk_transactions(
    pool: &actix_diesel::Database<PgConnection>,
    transactions: Vec<&near_indexer::IndexerTransactionWithOutcome>,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_hash: &str,
    block_timestamp: u64,
) -> anyhow::Result<()> {
    let transaction_models: Vec<models::transactions::Transaction> = transactions
        .iter()
        .enumerate()
        .map(|(index, tx)| {
            models::transactions::Transaction::from_indexer_transaction(
                tx,
                block_hash,
                chunk_hash,
                block_timestamp,
                index as i32,
            )
        })
        .collect();

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::transactions::table)
            .values(transaction_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "Transactions were stored in database".to_string(),
        &transaction_models
    );

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

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::transaction_actions::table)
            .values(transaction_action_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "TransactionActions were stored in database".to_string(),
        &transaction_action_models
    );

    Ok(())
}
