use bigdecimal::BigDecimal;

use crate::schema;
use schema::accounts;

#[derive(Insertable)]
pub struct Account {
    pub account_id: String,
    pub index: i32,
    pub created_by_receipt_id: String,
    pub created_at_timestamp: BigDecimal,
}

impl Account {
    pub fn new(
        account_id: String,
        index: i32,
        receipt_id: String,
        timestamp: BigDecimal
    ) -> Self {
        Self {
            account_id,
            index,
            created_by_receipt_id: receipt_id,
            created_at_timestamp: timestamp
        }
    }
}
