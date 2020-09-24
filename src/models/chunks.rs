use num_traits::cast::FromPrimitive;

use bigdecimal::BigDecimal;

use crate::schema;
use schema::chunks;

#[derive(Insertable, Clone)]
pub struct Chunk {
    pub block_id: BigDecimal,
    pub hash: String,
    pub shard_id: BigDecimal,
    pub signature: String,
    pub gas_limit: BigDecimal,
    pub gas_used: BigDecimal,
    pub height_created: BigDecimal,
    pub height_included: BigDecimal,
}

impl Chunk {
    pub fn from_chunk_view(block_height: u64, chunk_view: &near_indexer::IndexerChunkView) -> Self {
        eprintln!(
            "{:?} | {:?} | {:?}",
            chunk_view.header.height_created,
            chunk_view.header.height_included,
            chunk_view.header.gas_used,
        );
        Self {
            block_id: BigDecimal::from_u64(block_height).unwrap_or_else(|| 0.into()),
            hash: chunk_view.header.chunk_hash.to_string(),
            shard_id: BigDecimal::from_u64(chunk_view.header.shard_id).unwrap_or_else(|| 0.into()),
            signature: chunk_view.header.signature.to_string(),
            gas_limit: BigDecimal::from_u64(chunk_view.header.gas_limit)
                .unwrap_or_else(|| 0.into()),
            gas_used: BigDecimal::from_u64(chunk_view.header.gas_used).unwrap_or_else(|| 0.into()),
            height_created: BigDecimal::from_u64(chunk_view.header.height_created)
                .unwrap_or_else(|| 0.into()),
            height_included: BigDecimal::from_u64(chunk_view.header.height_included)
                .unwrap_or_else(|| 0.into()),
        }
    }
}
