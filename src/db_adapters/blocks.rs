use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

/// Saves block to database
pub(crate) async fn store_block(
    pool: &Pool<ConnectionManager<PgConnection>>,
    block: &near_primitives::views::BlockView,
) {
    let block_model = models::blocks::Block::from(block);
    loop {
        match diesel::insert_into(schema::blocks::table)
            .values(block_model.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while Block was adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &block_model
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }
}
