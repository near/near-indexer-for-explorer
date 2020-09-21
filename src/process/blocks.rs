use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

/// Saves block to database
/// Returns `AsyncError` in case if something go wrong
pub(crate) async fn process_block(
    pool: &Pool<ConnectionManager<PgConnection>>,
    block: &near_primitives::views::BlockView,
) {
    match diesel::insert_into(schema::blocks::table)
        .values(models::blocks::Block::from(block))
        .execute_async(&pool)
        .await
    {
        Ok(_) => (),
        Err(async_error) => {
            error!(target: "indexer_for_explorer", "Error occurred while Block was adding to database... \n {:#?}", async_error)
        }
    }
}
