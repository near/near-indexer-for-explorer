use bigdecimal::BigDecimal;

use crate::schema;
use schema::circulating_supply;

#[table_name = "circulating_supply"]
#[derive(Insertable, Queryable, Clone, Debug)]
pub struct CirculatingSupply {
    pub block_timestamp: BigDecimal,
    pub block_hash: String,
    pub value: BigDecimal,
    pub total_supply: BigDecimal,
    pub lockups_number: BigDecimal,
    pub active_lockups_number: BigDecimal,
    pub foundation_locked_supply: BigDecimal,
    pub lockups_locked_supply: BigDecimal,
}
