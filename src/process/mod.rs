pub(crate) mod blocks;
pub(crate) mod chunks;
pub(crate) mod execution_outcomes;
pub(crate) mod receipts;
pub(crate) mod transactions;

pub(crate) fn break_on_foreignkey_violation(async_error: &tokio_diesel::AsyncError) -> bool {
    if let tokio_diesel::AsyncError::Error(error) = async_error {
        if let diesel::result::Error::DatabaseError(kind, _) = error {
            if matches!(kind, diesel::result::DatabaseErrorKind::ForeignKeyViolation) {
                return true;
            }
        }
    }
    false
}
