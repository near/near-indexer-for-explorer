use actix_diesel::dsl::AsyncRunQueryDsl;
use anyhow::Context;
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use futures::future::try_join_all;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

/// Saves Transactions to database
pub(crate) async fn store_transactions(
    pool: &actix_diesel::Database<PgConnection>,
    shards: &[near_indexer::IndexerShard],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
    block_height: near_primitives::types::BlockHeight,
) -> anyhow::Result<()> {
    // TODO not sure it was a great idea
    let block_hash_str = block_hash.to_string();
    let mut tried_to_insert_transactions_count = 0;
    let futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| {
            tried_to_insert_transactions_count += chunk.transactions.len();
            store_chunk_transactions(
                pool,
                chunk
                    .transactions
                    .iter()
                    .collect::<Vec<&near_indexer::IndexerTransactionWithOutcome>>(),
                &chunk.header.chunk_hash,
                &block_hash_str,
                block_timestamp,
            )
        });

    try_join_all(futures).await?;

    let inserted_receipt_ids = collect_converted_to_receipt_ids(pool, block_hash).await?;
    // If the number is the same, I see no chance if there's something wrong, so we can return here
    if inserted_receipt_ids.len() == tried_to_insert_transactions_count {
        return Ok(());
    }

    let collided_tx_futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .flat_map(|chunk| {
            chunk
                .transactions
                .iter()
                .enumerate()
                .filter(|(_, transaction)| {
                    let converted_into_receipt_id = &transaction
                        .outcome
                        .execution_outcome
                        .outcome
                        .receipt_ids
                        .first()
                        .expect("`receipt_ids` must contain one Receipt Id")
                        .to_string();
                    !inserted_receipt_ids.contains(converted_into_receipt_id)
                })
                .map(|(index, transaction)| {
                    store_collided_transaction(
                        pool,
                        transaction,
                        index,
                        &chunk.header.chunk_hash,
                        &block_hash_str,
                        block_timestamp,
                        block_height,
                    )
                })
        });

    match try_join_all(collided_tx_futures).await {
        Ok(_) => Ok(()),
        Err(err) => {
            anyhow::bail!(err)
        }
    }
}

async fn collect_converted_to_receipt_ids(
    pool: &actix_diesel::Database<PgConnection>,
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
) -> anyhow::Result<Vec<String>> {
    Ok(schema::transactions::table
        .select(schema::transactions::dsl::converted_into_receipt_id)
        .filter(schema::transactions::dsl::included_in_block_hash.eq(block_hash.to_string()))
        .get_results_async::<String>(pool)
        .await
        .context("DB Error")?)
}

// TODO try to reuse
// tx hash suffix
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
                None,
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

async fn store_collided_transaction(
    pool: &actix_diesel::Database<PgConnection>,
    transaction: &near_indexer::IndexerTransactionWithOutcome,
    index: usize,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_hash: &str,
    block_timestamp: u64,
    block_height: near_primitives::types::BlockHeight,
) -> anyhow::Result<()> {
    // https://github.com/near/near-indexer-for-explorer/issues/84
    // TLDR: it's the hack to store transactions with collided hashes
    // It should not happen, but unfortunately it did,
    // we have ~10 such transactions in Mainnet for now
    let new_transaction_hash =
        transaction.transaction.hash.to_string() + "_issue84_" + &block_height.to_string();

    let transaction_model = models::transactions::Transaction::from_indexer_transaction(
        transaction,
        block_hash,
        chunk_hash,
        block_timestamp,
        index as i32,
        Some(new_transaction_hash.clone()),
    );

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::transactions::table)
            .values(transaction_model.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "Transactions were stored in database".to_string(),
        &transaction_model
    );

    let transaction_action_model: Vec<models::TransactionAction> = transaction
        .transaction
        .actions
        .iter()
        .enumerate()
        .map(move |(index, action)| {
            models::transactions::TransactionAction::from_action_view(
                new_transaction_hash.to_string(),
                index as i32,
                action,
            )
        })
        .collect();

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::transaction_actions::table)
            .values(transaction_action_model.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "TransactionActions were stored in database".to_string(),
        &transaction_action_model
    );

    Ok(())
}
