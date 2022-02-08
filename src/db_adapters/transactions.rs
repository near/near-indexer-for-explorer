use actix_diesel::dsl::AsyncRunQueryDsl;
use anyhow::Context;
use cached::Cached;
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
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let mut tried_to_insert_transactions_count = 0;
    let tx_futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| {
            tried_to_insert_transactions_count += chunk.transactions.len();
            store_chunk_transactions(
                pool,
                chunk
                    .transactions
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &near_indexer::IndexerTransactionWithOutcome)>>(),
                &chunk.header.chunk_hash,
                block_hash,
                block_timestamp,
                "",
                receipts_cache.clone(),
            )
        });

    try_join_all(tx_futures).await?;

    let inserted_receipt_ids = collect_converted_to_receipt_ids(pool, block_hash).await?;
    // If the number is the same, I see no chance if there's something wrong, so we can return here
    if inserted_receipt_ids.len() == tried_to_insert_transactions_count {
        return Ok(());
    }

    // https://github.com/near/near-indexer-for-explorer/issues/84
    // TLDR: it's the hack to store transactions with collided hashes
    // It should not happen, but unfortunately it did,
    // we have ~10 such transactions in Mainnet for now
    let transaction_hash_suffix = "_issue84_".to_owned() + &block_height.to_string();

    let collided_tx_futures = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| {
            store_chunk_transactions(
                pool,
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
                    .collect::<Vec<(usize, &near_indexer::IndexerTransactionWithOutcome)>>(),
                &chunk.header.chunk_hash,
                block_hash,
                block_timestamp,
                &transaction_hash_suffix,
                receipts_cache.clone(),
            )
        });

    try_join_all(collided_tx_futures).await.map(|_| ())
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

async fn store_chunk_transactions(
    pool: &actix_diesel::Database<PgConnection>,
    transactions: Vec<(usize, &near_indexer::IndexerTransactionWithOutcome)>,
    chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
    block_timestamp: u64,
    // hack for supporting duplicated transaction hashes. Empty for most of transactions
    transaction_hash_suffix: &str,
    receipts_cache: crate::ReceiptsCache,
) -> anyhow::Result<()> {
    let mut receipts_cache_lock = receipts_cache.lock().await;

    let transaction_models: Vec<models::transactions::Transaction> = transactions
        .iter()
        .map(|(index, tx)| {
            let transaction_hash = tx.transaction.hash.to_string() + transaction_hash_suffix;
            let converted_into_receipt_id = tx
                .outcome
                .execution_outcome
                .outcome
                .receipt_ids
                .first()
                .expect("`receipt_ids` must contain one Receipt Id");

            // Save this Transaction hash to ReceiptsCache
            // we use the Receipt ID to which this transaction was converted
            // and the Transaction hash as a value.
            // Later, while Receipt will be looking for a parent Transaction hash
            // it will be able to find it in the ReceiptsCache
            receipts_cache_lock.cache_set(
                crate::ReceiptOrDataId::ReceiptId(*converted_into_receipt_id),
                transaction_hash.clone(),
            );

            models::transactions::Transaction::from_indexer_transaction(
                tx,
                &transaction_hash,
                &converted_into_receipt_id.to_string(),
                block_hash,
                chunk_hash,
                block_timestamp,
                *index as i32,
            )
        })
        .collect();

    // releasing the lock
    drop(receipts_cache_lock);

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
        .flat_map(|(_, tx)| {
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
