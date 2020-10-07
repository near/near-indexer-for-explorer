use crate::schema;
use schema::accounts;

#[derive(Insertable, Debug, Clone)]
pub struct Account {
    pub account_id: String,
    pub created_by_receipt_id: String,
    pub deleted_by_receipt_id: Option<String>,
}

impl Account {
    pub fn new(
        account_id: String,
        created_by_receipt_id: &near_indexer::near_primitives::hash::CryptoHash,
    ) -> Self {
        Self {
            account_id,
            created_by_receipt_id: created_by_receipt_id.to_string(),
            deleted_by_receipt_id: None,
        }
    }
}
