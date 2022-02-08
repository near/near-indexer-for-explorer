use diesel_derive_enum::DbEnum;

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Receipt_kind"]
#[PgType = "receipt_kind"]
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
#[DieselType = "Action_kind"]
#[PgType = "action_kind"]
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
#[DieselType = "Access_key_permission_kind"]
#[PgType = "access_key_permission_kind"]
pub enum AccessKeyPermission {
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
            near_indexer::near_primitives::account::AccessKeyPermission::FunctionCall {
                ..
            } => Self::FunctionCall,
            near_indexer::near_primitives::account::AccessKeyPermission::FullAccess => {
                Self::FullAccess
            }
        }
    }
}

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "State_change_reason_kind"]
#[PgType = "state_change_reason_kind"]
pub enum StateChangeReasonKind {
    TransactionProcessing,
    ActionReceiptProcessingStarted,
    ActionReceiptGasReward,
    ReceiptProcessing,
    PostponedReceipt,
    UpdatedDelayedReceipts,
    ValidatorAccountsUpdate,
    Migration,
    Resharding,
}

impl From<&near_indexer::near_primitives::views::StateChangeCauseView> for StateChangeReasonKind {
    fn from(
        state_change_cause_view: &near_indexer::near_primitives::views::StateChangeCauseView,
    ) -> Self {
        match state_change_cause_view {
            near_indexer::near_primitives::views::StateChangeCauseView::TransactionProcessing { .. } => Self::TransactionProcessing,
            near_indexer::near_primitives::views::StateChangeCauseView::ActionReceiptProcessingStarted { .. } => Self::ActionReceiptProcessingStarted,
            near_indexer::near_primitives::views::StateChangeCauseView::ActionReceiptGasReward { .. } => Self::ActionReceiptGasReward,
            near_indexer::near_primitives::views::StateChangeCauseView::ReceiptProcessing { .. } => Self::ReceiptProcessing,
            near_indexer::near_primitives::views::StateChangeCauseView::PostponedReceipt { .. } => Self::PostponedReceipt,
            near_indexer::near_primitives::views::StateChangeCauseView::UpdatedDelayedReceipts { .. } => Self::UpdatedDelayedReceipts,
            near_indexer::near_primitives::views::StateChangeCauseView::ValidatorAccountsUpdate { .. } => Self::ValidatorAccountsUpdate,
            near_indexer::near_primitives::views::StateChangeCauseView::Migration { .. } => Self::Migration,
            near_indexer::near_primitives::views::StateChangeCauseView::Resharding { .. } => Self::Resharding,
            near_indexer::near_primitives::views::StateChangeCauseView::NotWritableToDisk | near_indexer::near_primitives::views::StateChangeCauseView::InitialState => panic!("Unexpected variant {:?} received", state_change_cause_view),
        }
    }
}

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Nft_event_kind"]
#[PgType = "nft_event_kind"]
pub enum NftEventKind {
    Mint,
    Transfer,
    Burn,
}

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Ft_event_kind"]
#[PgType = "ft_event_kind"]
pub enum FtEventKind {
    Mint,
    Transfer,
    Burn,
}
