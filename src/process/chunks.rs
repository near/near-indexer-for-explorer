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
    match diesel::insert_into(schema::chunks::table)
        .values(
            chunks
                .iter()
                .map(|chunk| models::chunks::Chunk::from_chunk_view(block_height, chunk))
                .collect::<Vec<models::chunks::Chunk>>(),
        )
        .execute_async(&pool)
        .await
    {
        Ok(_) => (),
        Err(async_error) => {
            error!(target: "indexer_for_explorer", "Error occurred while Chunks were adding to database... \n {:#?}", async_error)
        }
    }
}
