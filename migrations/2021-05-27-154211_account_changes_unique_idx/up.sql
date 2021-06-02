-- This constraint is bad because of 2 reasons:
-- 1. It does not contain update_reason, so it can remove needed lines
-- 2. Fortunately, it does not work at all because we have 2 nullable columns, they could not be compared by equality
--    (all nulls are considered unique)
ALTER TABLE account_changes DROP CONSTRAINT IF EXISTS account_changes_affected_account_id_changed_in_block_hash_c_key;

CREATE UNIQUE INDEX account_changes_transaction_uni_idx
ON account_changes (
    affected_account_id,
    changed_in_block_hash,
    caused_by_transaction_hash,
    update_reason,
    affected_account_nonstaked_balance,
    affected_account_staked_balance,
    affected_account_storage_usage
)
WHERE caused_by_transaction_hash IS NOT NULL AND caused_by_receipt_id IS NULL;

CREATE UNIQUE INDEX account_changes_receipt_uni_idx
ON account_changes (
    affected_account_id,
    changed_in_block_hash,
    caused_by_receipt_id,
    update_reason,
    affected_account_nonstaked_balance,
    affected_account_staked_balance,
    affected_account_storage_usage
)
WHERE caused_by_transaction_hash IS NULL AND caused_by_receipt_id IS NOT NULL;

CREATE UNIQUE INDEX account_changes_null_uni_idx
ON account_changes (
    affected_account_id,
    changed_in_block_hash,
    update_reason,
    affected_account_nonstaked_balance,
    affected_account_staked_balance,
    affected_account_storage_usage
)
WHERE caused_by_transaction_hash IS NULL AND caused_by_receipt_id IS NULL;
