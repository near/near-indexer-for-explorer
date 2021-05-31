CREATE UNIQUE INDEX account_changes_transaction_uni_idx
ON account_changes (affected_account_id, changed_in_block_hash, caused_by_transaction_hash)
WHERE caused_by_transaction_hash IS NOT NULL AND caused_by_receipt_id IS NULL;

CREATE UNIQUE INDEX account_changes_receipt_uni_idx
ON account_changes (affected_account_id, changed_in_block_hash, caused_by_receipt_id)
WHERE caused_by_transaction_hash IS NULL AND caused_by_receipt_id IS NOT NULL;

CREATE UNIQUE INDEX account_changes_null_uni_idx
ON account_changes (affected_account_id, changed_in_block_hash)
WHERE caused_by_transaction_hash IS NULL AND caused_by_receipt_id IS NULL;
