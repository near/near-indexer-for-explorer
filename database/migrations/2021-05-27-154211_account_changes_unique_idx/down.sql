-- See comment in corresponding up.sql
-- We decided to add it anyway for consistency
ALTER TABLE ONLY account_changes
    ADD CONSTRAINT account_changes_affected_account_id_changed_in_block_hash_c_key UNIQUE (
    affected_account_id,
    changed_in_block_hash,
    caused_by_transaction_hash,
    caused_by_receipt_id);

DROP INDEX IF EXISTS account_changes_transaction_uni_idx;
DROP INDEX IF EXISTS account_changes_receipt_uni_idx;
DROP INDEX IF EXISTS account_changes_null_uni_idx;
