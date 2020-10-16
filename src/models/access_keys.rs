use std::convert::TryFrom;

use crate::models::enums::AccessKeyPermission;
use crate::schema;
use schema::access_keys;

#[derive(Insertable, Clone, Debug)]
pub struct AccessKey {
    pub public_key: String,
    pub account_id: String,
    pub created_by_receipt_id: Option<String>,
    pub deleted_by_receipt_id: Option<String>,
    pub permission: AccessKeyPermission,
}

impl AccessKey {
    pub fn from_receipt_view(
        receipt: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Vec<Self> {
        let mut access_keys: Vec<Self> = vec![];
        if let near_indexer::near_primitives::views::ReceiptEnumView::Action { actions, .. } =
            &receipt.receipt
        {
            for action in actions {
                let access_key = match action {
                    near_indexer::near_primitives::views::ActionView::AddKey {
                        public_key,
                        access_key,
                    } => Self {
                        public_key: public_key.to_string(),
                        account_id: receipt.receiver_id.to_string(),
                        created_by_receipt_id: Some(receipt.receipt_id.to_string()),
                        deleted_by_receipt_id: None,
                        permission: (&access_key.permission).into(),
                    },
                    near_indexer::near_primitives::views::ActionView::Transfer { .. } => {
                        if receipt.receiver_id.len() != 64usize {
                            continue;
                        }
                        if let Ok(public_key_bytes) = hex::decode(&receipt.receiver_id) {
                            if let Ok(public_key) =
                                near_crypto::ED25519PublicKey::try_from(&public_key_bytes[..])
                            {
                                Self {
                                    public_key: near_crypto::PublicKey::from(public_key)
                                        .to_string(),
                                    account_id: receipt.receiver_id.to_string(),
                                    created_by_receipt_id: Some(receipt.receipt_id.to_string()),
                                    deleted_by_receipt_id: None,
                                    permission: AccessKeyPermission::FullAccess,
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };
                access_keys.push(access_key);
            }
        }
        access_keys
    }

    pub fn from_action_view(
        public_key: &near_crypto::PublicKey,
        account_id: &str,
        access_key: &near_indexer::near_primitives::views::AccessKeyView,
        create_by_receipt_id: &near_indexer::near_primitives::hash::CryptoHash,
    ) -> Self {
        Self {
            public_key: public_key.to_string(),
            account_id: account_id.to_string(),
            created_by_receipt_id: Some(create_by_receipt_id.to_string()),
            deleted_by_receipt_id: None,
            permission: (&access_key.permission).into(),
        }
    }
}
