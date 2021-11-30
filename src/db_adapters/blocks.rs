use actix_diesel::dsl::AsyncRunQueryDsl;
use anyhow::Context;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

/// Saves block to database
pub(crate) async fn store_block(
    pool: &actix_diesel::Database<PgConnection>,
    block: &near_primitives::views::BlockView,
) -> anyhow::Result<()> {
    let block_model = models::blocks::Block::from(block);

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::blocks::table)
            .values(block_model.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "Block was stored to database".to_string(),
        &block_model
    );
    Ok(())
}

/// Gets the latest block's height from database
pub(crate) async fn latest_block_height(
    pool: &actix_diesel::Database<PgConnection>,
) -> Result<Option<u64>, String> {
    tracing::debug!(target: crate::INDEXER_FOR_EXPLORER, "fetching latest");
    Ok(schema::blocks::table
        .select((schema::blocks::dsl::block_height,))
        .order(schema::blocks::dsl::block_height.desc())
        .limit(1)
        .get_optional_result_async::<(bigdecimal::BigDecimal,)>(pool)
        .await
        .map_err(|err| format!("DB Error: {}", err))?
        .and_then(|(block_height,)| block_height.to_u64()))
}

pub(crate) async fn get_latest_block_before_timestamp(
    pool: &actix_diesel::Database<PgConnection>,
    timestamp: u64,
) -> anyhow::Result<models::Block> {
    Ok(schema::blocks::table
        .filter(schema::blocks::dsl::block_timestamp.le(BigDecimal::from(timestamp)))
        .order(schema::blocks::dsl::block_timestamp.desc())
        .first_async::<models::Block>(pool)
        .await
        .context("DB Error")?)
}
