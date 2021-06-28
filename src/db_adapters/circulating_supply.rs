use actix_diesel::dsl::AsyncRunQueryDsl;
use bigdecimal::BigDecimal;
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use tracing::error;

use crate::models::circulating_supply::CirculatingSupply;
use crate::schema;

pub(crate) async fn add_circulating_supply(
    pool: &actix_diesel::Database<PgConnection>,
    stats: &CirculatingSupply,
) {
    let mut interval = crate::INTERVAL;
    loop {
        match diesel::insert_into(schema::circulating_supply::table)
            .values(stats.to_owned())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(_) => {
                break;
            }
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while Circulating Supply was adding to database. Retrying in {} milliseconds... \n {:#?}",
                    interval.as_millis(),
                    async_error
                );
                tokio::time::sleep(interval).await;
                if interval < crate::MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}

pub(crate) async fn get_precomputed_circulating_supply(
    timestamp: u64,
    pool: &actix_diesel::Database<PgConnection>,
) -> Result<Option<u128>, String> {
    let supply = schema::circulating_supply::table
        .select(schema::circulating_supply::dsl::value)
        .filter(schema::circulating_supply::dsl::block_timestamp.eq(BigDecimal::from(timestamp)))
        .get_optional_result_async::<bigdecimal::BigDecimal>(&pool)
        .await;

    return match supply {
        Ok(Some(value)) => Ok(Some(
            u128::from_str_radix(&value.to_string(), 10).expect("`value` expected to be u128"),
        )),
        Ok(None) => Ok(None),
        Err(err) => Err(format!("DB Error: {}", err)),
    };
}
