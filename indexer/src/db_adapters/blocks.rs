use actix_diesel::dsl::AsyncRunQueryDsl;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};

use crate::models;
use crate::schema;

/// Saves block to database
pub(crate) async fn store_block(
    pool: &actix_diesel::Database<PgConnection>,
    block: &near_lake_framework::near_indexer_primitives::views::BlockView,
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
) -> anyhow::Result<Option<u64>> {
    tracing::debug!(target: crate::INDEXER_FOR_EXPLORER, "fetching latest");
    Ok(schema::blocks::table
        .select((schema::blocks::dsl::block_height,))
        .order(schema::blocks::dsl::block_height.desc())
        .limit(1)
        .get_optional_result_async::<(BigDecimal,)>(pool)
        .await?
        .and_then(|(block_height,)| block_height.to_u64()))
}
