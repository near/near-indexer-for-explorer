use actix_diesel::dsl::AsyncRunQueryDsl;
use diesel::PgConnection;

use crate::models;
use crate::schema;

/// Saves chunks to database
pub(crate) async fn store_chunks(
    pool: &actix_diesel::Database<PgConnection>,
    shards: &[near_indexer::IndexerShard],
    block_hash: &near_indexer::near_primitives::hash::CryptoHash,
) -> anyhow::Result<()> {
    if shards.is_empty() {
        return Ok(());
    }
    let chunk_models: Vec<models::chunks::Chunk> = shards
        .iter()
        .filter_map(|shard| shard.chunk.as_ref())
        .map(|chunk| models::chunks::Chunk::from_chunk_view(chunk, block_hash))
        .collect();

    if chunk_models.is_empty() {
        return Ok(());
    }

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::chunks::table)
            .values(chunk_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "Chunks were stored to database".to_string(),
        &chunk_models
    );
    Ok(())
}
