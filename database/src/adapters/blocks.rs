use actix_diesel::dsl::AsyncRunQueryDsl;
use anyhow::Context;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::{ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl};

use crate::models;
use crate::schema;

/// Saves block to database
pub async fn store_block(
    pool: &actix_diesel::Database<PgConnection>,
    block: &near_indexer_primitives::views::BlockView,
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
/// Hacked version to track deprecated tables
pub async fn latest_block_height(
    pool: &actix_diesel::Database<PgConnection>,
) -> anyhow::Result<Option<u64>> {
    tracing::debug!(target: crate::EXPLORER_DATABASE, "fetching latest");
    Ok(schema::blocks::table
        .inner_join(
            schema::assets__fungible_token_events::table.on(schema::blocks::dsl::block_timestamp
                .eq(schema::assets__fungible_token_events::dsl::emitted_at_block_timestamp)),
        )
        .select((schema::blocks::dsl::block_height,))
        .order(schema::blocks::dsl::block_height.desc())
        .limit(1)
        .get_optional_result_async::<(BigDecimal,)>(pool)
        .await?
        .and_then(|(block_height,)| block_height.to_u64()))
}

pub async fn get_latest_block_before_timestamp(
    pool: &actix_diesel::Database<PgConnection>,
    timestamp: u64,
) -> anyhow::Result<models::Block> {
    schema::blocks::table
        .filter(schema::blocks::dsl::block_timestamp.le(BigDecimal::from(timestamp)))
        .order(schema::blocks::dsl::block_timestamp.desc())
        .first_async::<models::Block>(pool)
        .await
        .context("DB Error")
}
