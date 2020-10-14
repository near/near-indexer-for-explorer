use crate::schema;
use schema::accounts;

#[derive(Insertable, Debug, Clone)]
pub struct Account {
    pub account_id: String,
    pub created_by_receipt_id: Option<String>,
    pub deleted_by_receipt_id: Option<String>,
}

impl Account {
    pub fn new(
        account_id: String,
        created_by_receipt_id: Option<&near_indexer::near_primitives::hash::CryptoHash>,
    ) -> Self {
        Self {
            account_id,
            created_by_receipt_id: match created_by_receipt_id {
                Some(receipt_id) => Some(receipt_id.to_string()),
                None => None,
            },
            deleted_by_receipt_id: None,
        }
    }
}
