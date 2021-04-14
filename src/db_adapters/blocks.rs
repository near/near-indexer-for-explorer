use actix_diesel::dsl::AsyncRunQueryDsl;
use bigdecimal::ToPrimitive;
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use tracing::error;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

/// Saves block to database
pub(crate) async fn store_block(
    pool: &actix_diesel::Database<PgConnection>,
    block: &near_primitives::views::BlockView,
) {
    let block_model = models::blocks::Block::from(block);

    let mut interval = crate::INTERVAL;
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
                    interval.as_millis(),
                    async_error,
                    &block_model
                );
                tokio::time::sleep(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}

/// Gets the latest block's height from database
pub(crate) async fn latest_block_height(
    pool: &actix_diesel::Database<PgConnection>,
) -> Result<u64, String> {
    let result: Vec<models::blocks::Block> = schema::blocks::table
        .limit(1)
        .order(schema::blocks::dsl::block_height.desc())
        .load_async(&pool)
        .await
        .unwrap_or_else(|_| vec![]);

    if result.is_empty() {
        return Err("No blocks found in database".to_string());
    }

    Ok(result[0]
        .block_height
        .clone()
        .to_u64()
        .expect("Failed to convert block_height from BigDecimal to u64"))
}
