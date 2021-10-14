use bigdecimal::BigDecimal;

use crate::schema;
use schema::aggregated__circulating_supply;

#[derive(Insertable, Queryable, Clone, Debug)]
#[table_name = "aggregated__circulating_supply"]
pub struct CirculatingSupply {
    pub computed_at_block_timestamp: BigDecimal,
    pub computed_at_block_hash: String,
    pub circulating_tokens_supply: BigDecimal,
    pub total_tokens_supply: BigDecimal,
    pub total_lockup_contracts_count: i32,
    pub unfinished_lockup_contracts_count: i32,
    pub foundation_locked_tokens: BigDecimal,
    pub lockups_locked_tokens: BigDecimal,
}
