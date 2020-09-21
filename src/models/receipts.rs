use num_traits::cast::FromPrimitive;
use std::convert::TryFrom;

use bigdecimal::BigDecimal;
use serde_json::{json, Value};

use near_indexer::near_primitives::views::{ActionView, DataReceiverView};

use crate::models::enums::{ActionType, ReceiptType};
use crate::schema;
use schema::{
    receipt_action_actions, receipt_action_input_data, receipt_action_output_data, receipt_actions,
    receipt_data, receipts,
};

#[derive(Insertable)]
pub struct Receipt {
    pub receipt_id: String,
    pub block_height: BigDecimal,
    pub predecessor_id: String,
    pub receiver_id: String,
    pub type_: ReceiptType,
}

impl Receipt {
    pub fn from_receipt_view(
        receipt: &near_indexer::near_primitives::views::ReceiptView,
        block_height: u64,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.to_string(),
            block_height: block_height.into(),
            predecessor_id: receipt.predecessor_id.to_string(),
            receiver_id: receipt.receiver_id.to_string(),
            type_: match receipt.receipt {
                near_indexer::near_primitives::views::ReceiptEnumView::Action { .. } => {
                    ReceiptType::Action
                }
                near_indexer::near_primitives::views::ReceiptEnumView::Data { .. } => {
                    ReceiptType::Data
                }
            },
        }
    }
}

#[derive(Insertable)]
#[table_name = "receipt_data"]
pub struct ReceiptData {
    pub receipt_id: String,
    pub data_id: String,
    pub data: Option<Vec<u8>>,
}

impl TryFrom<&near_indexer::near_primitives::views::ReceiptView> for ReceiptData {
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
            Err("Given ReceiptView is not Data type")
        }
    }
}

#[derive(Insertable)]
pub struct ReceiptAction {
    pub receipt_id: String,
    pub signer_id: String,
    pub signer_public_key: String,
    pub gas_price: BigDecimal,
}

impl TryFrom<&near_indexer::near_primitives::views::ReceiptView> for ReceiptAction {
    type Error = &'static str;

    fn try_from(
        receipt_view: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Result<Self, Self::Error> {
        if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
            signer_id,
            signer_public_key,
            gas_price,
            ..
        } = &receipt_view.receipt {
            Ok(
                Self {
                    receipt_id: receipt_view.receipt_id.to_string(),
                    signer_id: signer_id.to_string(),
                    signer_public_key: signer_public_key.to_string(),
                    gas_price: BigDecimal::from_u128(*gas_price).unwrap_or_else(||0.into()),
                }
            )
        } else {
            Err("Given ReceiptView is not Action type")
        }
    }
}

#[derive(Insertable)]
#[table_name = "receipt_action_actions"]
pub struct ReceiptActionAction {
    pub receipt_id: String,
    pub index: i32,
    pub type_: ActionType,
    pub args: Option<serde_json::Value>,
}

impl ReceiptActionAction {
    pub fn from_action_view(
        receipt_id: String,
        index: i32,
        action_view: &near_indexer::near_primitives::views::ActionView,
    ) -> Self {
        let (type_, args): (ActionType, Option<Value>) = match &action_view {
            ActionView::CreateAccount => (ActionType::CreateAccount, None),
            ActionView::DeployContract { code } => {
                (ActionType::DeployContract, Some(json!({ "code": code })))
            }
            ActionView::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => (
                ActionType::FunctionCall,
                Some(json!({
                    "method_name": method_name,
                    "args": args,
                    "gas": gas,
                    "deposit": deposit.to_string(),
                })),
            ),
            ActionView::Transfer { deposit } => (
                ActionType::Transfer,
                Some(json!({ "deposit": deposit.to_string() })),
            ),
            ActionView::Stake { stake, public_key } => (
                ActionType::Stake,
                Some(json!({
                    "stake": stake.to_string(),
                    "public_key": public_key,
                })),
            ),
            ActionView::AddKey {
                public_key,
                access_key,
            } => (
                ActionType::AddKey,
                Some(json!({
                    "public_key": public_key,
                    "access_key": access_key,
                })),
            ),
            ActionView::DeleteKey { public_key } => (
                ActionType::DeleteKey,
                Some(json!({
                    "public_key": public_key,
                })),
            ),
            ActionView::DeleteAccount { beneficiary_id } => (
                ActionType::DeleteAccount,
                Some(json!({
                    "beneficiary_id": beneficiary_id,
                })),
            ),
        };
        Self {
            receipt_id,
            index,
            args,
            type_,
        }
    }
}

#[derive(Insertable)]
#[table_name = "receipt_action_input_data"]
pub struct ReceiptActionInputData {
    pub receipt_id: String,
    pub data_id: String,
}

impl ReceiptActionInputData {
    pub fn from_data_id(receipt_id: String, data_id: String) -> Self {
        Self {
            receipt_id,
            data_id,
        }
    }
}

#[derive(Insertable)]
#[table_name = "receipt_action_output_data"]
pub struct ReceiptActionOutputData {
    pub receipt_id: String,
    pub data_id: String,
    pub receiver_id: String,
}

impl ReceiptActionOutputData {
    pub fn from_data_receiver(receipt_id: String, data_receiver: &DataReceiverView) -> Self {
        Self {
            receipt_id,
            data_id: data_receiver.data_id.to_string(),
            receiver_id: data_receiver.receiver_id.to_string(),
        }
    }
}
