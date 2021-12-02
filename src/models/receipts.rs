use std::convert::TryFrom;
use std::str::FromStr;

use bigdecimal::BigDecimal;

use near_indexer::near_primitives::views::DataReceiverView;

use crate::models::enums::{ActionKind, ReceiptKind};
use crate::schema;
use schema::{
    action_receipt_actions, action_receipt_input_data, action_receipt_output_data, action_receipts,
    data_receipts, receipts,
};

#[derive(Insertable, Queryable, Clone, Debug)]
pub struct Receipt {
    pub receipt_id: String,
    pub included_in_block_hash: String,
    pub included_in_chunk_hash: String,
    pub index_in_chunk: i32,
    pub included_in_block_timestamp: BigDecimal,
    pub predecessor_account_id: String,
    pub receiver_account_id: String,
    pub receipt_kind: ReceiptKind,
    pub originated_from_transaction_hash: String,
}

impl Receipt {
    pub fn from_receipt_view(
        receipt: &near_indexer::near_primitives::views::ReceiptView,
        block_hash: &near_indexer::near_primitives::hash::CryptoHash,
        transaction_hash: &str,
        chunk_hash: &near_indexer::near_primitives::hash::CryptoHash,
        index_in_chunk: i32,
        block_timestamp: u64,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            included_in_block_hash: block_hash.to_string(),
            included_in_chunk_hash: chunk_hash.to_string(),
            predecessor_account_id: receipt.predecessor_id.to_string(),
            receiver_account_id: receipt.receiver_id.to_string(),
            receipt_kind: (&receipt.receipt).into(),
            originated_from_transaction_hash: transaction_hash.to_string(),
            index_in_chunk,
            included_in_block_timestamp: block_timestamp.into(),
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "data_receipts"]
pub struct DataReceipt {
    pub data_id: String,
    pub receipt_id: String,
    pub data: Option<Vec<u8>>,
}

impl TryFrom<&near_indexer::near_primitives::views::ReceiptView> for DataReceipt {
    type Error = &'static str;

    fn try_from(
        receipt_view: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Result<Self, Self::Error> {
        if let near_indexer::near_primitives::views::ReceiptEnumView::Data { data_id, data } =
            &receipt_view.receipt
        {
            Ok(Self {
                receipt_id: receipt_view.receipt_id.to_string(),
                data_id: data_id.to_string(),
                data: data.clone(),
            })
        } else {
            Err("Given ReceiptView is not of Data variant")
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "action_receipts"]
pub struct ActionReceipt {
    pub receipt_id: String,
    pub signer_account_id: String,
    pub signer_public_key: String,
    pub gas_price: BigDecimal,
}

impl TryFrom<&near_indexer::near_primitives::views::ReceiptView> for ActionReceipt {
    type Error = &'static str;

    fn try_from(
        receipt_view: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Result<Self, Self::Error> {
        if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
            signer_id,
            signer_public_key,
            gas_price,
            ..
        } = &receipt_view.receipt
        {
            Ok(Self {
                receipt_id: receipt_view.receipt_id.to_string(),
                signer_account_id: signer_id.to_string(),
                signer_public_key: signer_public_key.to_string(),
                gas_price: BigDecimal::from_str(gas_price.to_string().as_str())
                    .expect("gas_price expected to be u128"),
            })
        } else {
            Err("Given ReceiptView is not of Action variant")
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "action_receipt_actions"]
pub struct ActionReceiptAction {
    pub receipt_id: String,
    pub index_in_action_receipt: i32,
    pub action_kind: ActionKind,
    pub args: serde_json::Value,
    pub receipt_predecessor_account_id: String,
    pub receipt_receiver_account_id: String,
    pub receipt_included_in_block_timestamp: BigDecimal,
}

impl ActionReceiptAction {
    pub fn from_action_view(
        receipt_id: String,
        index: i32,
        action_view: &near_indexer::near_primitives::views::ActionView,
        predecessor_account_id: String,
        receiver_account_id: String,
        block_timestamp: u64,
    ) -> Self {
        let (action_kind, args) =
            crate::models::extract_action_type_and_value_from_action_view(action_view);

        Self {
            receipt_id,
            index_in_action_receipt: index,
            args,
            action_kind,
            receipt_predecessor_account_id: predecessor_account_id,
            receipt_receiver_account_id: receiver_account_id,
            receipt_included_in_block_timestamp: block_timestamp.into(),
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "action_receipt_input_data"]
pub struct ActionReceiptInputData {
    pub input_to_receipt_id: String,
    pub input_data_id: String,
}

impl ActionReceiptInputData {
    pub fn from_data_id(receipt_id: String, data_id: String) -> Self {
        Self {
            input_to_receipt_id: receipt_id,
            input_data_id: data_id,
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "action_receipt_output_data"]
pub struct ActionReceiptOutputData {
    pub output_from_receipt_id: String,
    pub output_data_id: String,
    pub receiver_account_id: String,
}

impl ActionReceiptOutputData {
    pub fn from_data_receiver(receipt_id: String, data_receiver: &DataReceiverView) -> Self {
        Self {
            output_from_receipt_id: receipt_id,
            output_data_id: data_receiver.data_id.to_string(),
            receiver_account_id: data_receiver.receiver_id.to_string(),
        }
    }
}
