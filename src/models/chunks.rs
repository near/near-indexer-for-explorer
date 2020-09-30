use bigdecimal::BigDecimal;

use crate::schema;
use schema::chunks;

#[derive(Insertable, Clone, Debug)]
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
    pub fn from_chunk_view(
        block_height: near_indexer::near_primitives::types::BlockHeight,
        chunk_view: &near_indexer::IndexerChunkView,
    ) -> Self {
        Self {
            block_id: block_height.into(),
            hash: chunk_view.header.chunk_hash.to_string(),
            shard_id: chunk_view.header.shard_id.into(),
            signature: chunk_view.header.signature.to_string(),
            gas_limit: chunk_view.header.gas_limit.into(),
            gas_used: chunk_view.header.gas_used.into(),
            height_created: chunk_view.header.height_created.into(),
            height_included: chunk_view.header.height_included.into(),
        }
    }
}
