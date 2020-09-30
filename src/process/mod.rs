pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
pub(crate) mod receipts;
pub(crate) mod transactions;

pub(crate) fn break_on_foreignkey_violation(async_error: &tokio_diesel::AsyncError) -> bool {
    matches!(async_error,
        tokio_diesel::AsyncError::Error(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::ForeignKeyViolation, _)))
}
