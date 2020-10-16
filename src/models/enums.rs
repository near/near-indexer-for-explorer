use diesel_derive_enum::DbEnum;

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Receipt_type"]
#[PgType = "receipt_type"]
pub enum ReceiptKind {
    Action,
    Data,
}

impl From<&near_indexer::near_primitives::views::ReceiptEnumView> for ReceiptKind {
    fn from(receipt_enum_view: &near_indexer::near_primitives::views::ReceiptEnumView) -> Self {
        match receipt_enum_view {
            near_indexer::near_primitives::views::ReceiptEnumView::Action { .. } => Self::Action,
            near_indexer::near_primitives::views::ReceiptEnumView::Data { .. } => Self::Data,
        }
    }
}

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Action_type"]
#[PgType = "action_type"]
pub enum ActionKind {
    CreateAccount,
    DeployContract,
    FunctionCall,
    Transfer,
    Stake,
    AddKey,
    DeleteKey,
    DeleteAccount,
}

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Execution_outcome_status"]
#[PgType = "execution_outcome_status"]
pub enum ExecutionOutcomeStatus {
    Unknown,
    Failure,
    SuccessValue,
    SuccessReceiptId,
}

impl From<near_indexer::near_primitives::views::ExecutionStatusView> for ExecutionOutcomeStatus {
    fn from(status: near_indexer::near_primitives::views::ExecutionStatusView) -> Self {
        match status {
            near_indexer::near_primitives::views::ExecutionStatusView::Unknown => Self::Unknown,
            near_indexer::near_primitives::views::ExecutionStatusView::Failure(_) => Self::Failure,
            near_indexer::near_primitives::views::ExecutionStatusView::SuccessValue(_) => {
                Self::SuccessValue
            }
            near_indexer::near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => {
                Self::SuccessReceiptId
            }
        }
    }
}

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Access_key_permission_type"]
#[PgType = "access_key_permission_type"]
pub enum AccessKeyPermission {
    /// Used only with AccessKeyAction::Delete
    NotApplicable,
    /// Used only with AccessKeyAction::Add
    FullAccess,
    /// Used only with AccessKeyAction::Add
    FunctionCall,
}

impl From<&near_indexer::near_primitives::views::AccessKeyPermissionView> for AccessKeyPermission {
    fn from(item: &near_indexer::near_primitives::views::AccessKeyPermissionView) -> Self {
        match item {
            near_indexer::near_primitives::views::AccessKeyPermissionView::FunctionCall {
                ..
            } => Self::FunctionCall,
            near_indexer::near_primitives::views::AccessKeyPermissionView::FullAccess => {
                Self::FullAccess
            }
        }
    }
}

impl From<&near_indexer::near_primitives::account::AccessKeyPermission> for AccessKeyPermission {
    fn from(item: &near_indexer::near_primitives::account::AccessKeyPermission) -> Self {
        match item {
            near_indexer::near_primitives::account::AccessKeyPermission::FunctionCall(_) => {
                Self::FunctionCall
            }
            near_indexer::near_primitives::account::AccessKeyPermission::FullAccess => {
                Self::FullAccess
            }
        }
    }
}
