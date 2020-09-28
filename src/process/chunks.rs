use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use crate::models;
use crate::schema;

/// Saves chunks to database
pub(crate) async fn process_chunks(
    pool: &Pool<ConnectionManager<PgConnection>>,
    chunks: &[near_indexer::IndexerChunkView],
    block_height: u64,
) {
    let chunk_models: Vec<models::chunks::Chunk> = chunks
        .iter()
        .map(|chunk| models::chunks::Chunk::from_chunk_view(block_height, chunk))
        .collect();

    loop {
        match diesel::insert_into(schema::chunks::table)
            .values(chunk_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while Chunks were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &chunk_models
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }
}
