use bigdecimal::BigDecimal;

#[derive(Queryable, Clone, Debug)]
pub struct Lockup {
    pub account_id: String,
    pub creation_block_height: BigDecimal,
    pub deletion_block_height: BigDecimal,
}
