use bigdecimal::BigDecimal;

use crate::schema;
use schema::assets__fungible_token_operations;

#[table_name = "assets__fungible_token_operations"]
#[derive(Insertable, Queryable, Clone, Debug)]
pub struct FungibleTokenOperation {
    pub processed_in_receipt_id: String,
    pub processed_in_block_timestamp: BigDecimal,
    pub called_method: String,
    pub ft_contract_account_id: String,
    pub ft_sender_account_id: String,
    pub ft_receiver_account_id: String,
    pub ft_amount: BigDecimal,
    pub args: serde_json::Value,
}
