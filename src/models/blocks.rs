use std::str::FromStr;

use bigdecimal::BigDecimal;

use near_indexer::near_primitives;

use crate::schema;
use schema::blocks;

#[derive(Insertable, Clone, Debug)]
pub struct Block {
    pub height: BigDecimal,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: BigDecimal,
    pub total_supply: BigDecimal,
    pub gas_price: BigDecimal,
}

impl From<&near_primitives::views::BlockView> for Block {
    fn from(block_view: &near_primitives::views::BlockView) -> Self {
        Self {
            height: block_view.header.height.into(),
            hash: block_view.header.hash.to_string(),
            prev_hash: block_view.header.prev_hash.to_string(),
            timestamp: block_view.header.timestamp.into(),
            total_supply: BigDecimal::from_str(block_view.header.total_supply.to_string().as_str())
                .expect("`total_supply` expected to be u128"),
            gas_price: BigDecimal::from_str(block_view.header.gas_price.to_string().as_str())
                .expect("`gas_price` expected to be u128"),
        }
    }
}
