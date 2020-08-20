use num_traits::cast::FromPrimitive;

use bigdecimal::BigDecimal;

use near_indexer::near_primitives;

use crate::schema;
use schema::blocks;

#[derive(Insertable)]
pub struct Block {
    pub height: BigDecimal,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: BigDecimal,
    pub total_supply: BigDecimal,
    pub gas_limit: BigDecimal,
    pub gas_used: BigDecimal,
    pub gas_price: BigDecimal,
}

impl Block {
    pub fn from_block_view(block_view: &near_primitives::views::BlockView) -> Self {
        Self {
            height: block_view.header.height.into(),
            hash: block_view.header.hash.to_string(),
            prev_hash: block_view.header.prev_hash.to_string(),
            timestamp: BigDecimal::from_u64(block_view.header.timestamp).unwrap_or(0.into()),
            total_supply: BigDecimal::from_u128(block_view.header.total_supply).unwrap_or(0.into()),
            gas_limit: 0.into(),
            gas_used: 0.into(),
            gas_price: BigDecimal::from_u128(block_view.header.gas_price).unwrap_or(0.into()),
        }
    }
}
