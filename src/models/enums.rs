use diesel_derive_enum::DbEnum;

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
#[DieselType = "Receipt_type"]
#[PgType = "receipt_type"]
pub enum ReceiptType {
    Action,
    Data,
}

#[derive(Debug, DbEnum, Clone)]
#[DbValueStyle = "PascalCase"]
#[DieselType = "Action_type"]
#[PgType = "action_type"]
pub enum ActionType {
    CreateAccount,
    DeployContract,
    FunctionCall,
    Transfer,
    Stake,
    AddKey,
    DeleteKey,
    DeleteAccount,
}
