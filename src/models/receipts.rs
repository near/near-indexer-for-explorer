use num_traits::cast::FromPrimitive;

use bigdecimal::BigDecimal;

use near_indexer;
use near_indexer::near_primitives::views::DataReceiverView;

use crate::schema;
use schema::{actions_input_data, actions_output_data, receipt_action, receipt_data, receipts};

#[derive(Insertable, Queryable, AsChangeset)]
#[primary_key("receipt_id")]
pub struct Receipt {
    pub receipt_id: String,
    pub predecessor_id: Option<String>,
    pub receiver_id: Option<String>,
    pub status: String,
    pub type_: Option<String>,
}

#[derive(Insertable)]
#[table_name = "receipt_data"]
pub struct ReceiptData {
    pub receipt_id: String,
    pub data_id: String,
    pub data: Option<String>,
}

#[derive(Insertable)]
#[table_name = "receipt_action"]
pub struct ReceiptAction {
    pub receipt_id: String,
    pub signer_id: String,
    pub signer_public_key: String,
    pub gas_price: Option<BigDecimal>,
}

#[derive(Insertable)]
#[table_name = "actions_input_data"]
pub struct ReceiptActionInputData {
    pub receipt_id: String,
    pub data_id: String,
}

#[derive(Insertable)]
#[table_name = "actions_output_data"]
pub struct ReceiptActionOutputData {
    pub receipt_id: String,
    pub data_id: String,
    pub receiver_id: String,
}

impl Receipt {
    pub fn from_receipt(receipt_view: &near_indexer::near_primitives::views::ReceiptView) -> Self {
        Self {
            receipt_id: receipt_view.receipt_id.to_string(),
            predecessor_id: Some(receipt_view.predecessor_id.to_string()),
            receiver_id: Some(receipt_view.receiver_id.to_string()),
            status: "empty".to_string(),
            type_: match &receipt_view.receipt {
                ref
                _entity
                @
                near_indexer::near_primitives::views::ReceiptEnumView::Action {
                    ..
                } => Some("action".to_string()),
                ref
                _entity
                @
                near_indexer::near_primitives::views::ReceiptEnumView::Data {
                    ..
                } => Some("data".to_string()),
            },
        }
    }

    pub fn from_receipt_id(receipt_id: String) -> Self {
        Self {
            receipt_id: receipt_id,
            predecessor_id: None,
            receiver_id: None,
            status: "empty".to_string(),
            type_: None,
        }
    }
}

impl ReceiptData {
    pub fn from_receipt(
        receipt_view: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Result<Self, &str> {
        match &receipt_view.receipt {
            near_indexer::near_primitives::views::ReceiptEnumView::Data { data_id, data } => {
                Ok(Self {
                    receipt_id: receipt_view.receipt_id.to_string(),
                    data_id: data_id.to_string(),
                    data: if let Some(data_) = data {
                        Some(std::str::from_utf8(&data_[..]).unwrap().to_string())
                    } else {
                        None
                    },
                })
            }
            _ => Err("This Receipt is not Data"),
        }
    }
}

impl ReceiptAction {
    pub fn from_receipt(
        receipt_view: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Result<Self, &str> {
        match &receipt_view.receipt {
            near_indexer::near_primitives::views::ReceiptEnumView::Action {
                signer_id,
                signer_public_key,
                gas_price,
                output_data_receivers: _,
                input_data_ids: _,
                actions: _,
            } => Ok(Self {
                receipt_id: receipt_view.receipt_id.to_string(),
                signer_id: signer_id.to_string(),
                signer_public_key: signer_public_key.to_string(),
                gas_price: BigDecimal::from_u128(*gas_price),
            }),
            _ => Err("This Receipt is not Action"),
        }
    }
}

impl ReceiptActionInputData {
    pub fn from_data_id(receipt_id: String, data_id: String) -> Self {
        Self {
            receipt_id,
            data_id,
        }
    }
}

impl ReceiptActionOutputData {
    pub fn from_data_receiver(receipt_id: String, data_receiver: &DataReceiverView) -> Self {
        Self {
            receipt_id,
            data_id: data_receiver.data_id.to_string().clone(),
            receiver_id: data_receiver.receiver_id.to_string().clone(),
        }
    }
}
