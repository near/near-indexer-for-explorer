-- ACCOUNTS
ALTER TABLE accounts
    ADD COLUMN last_update_block_height numeric(20, 0);

CREATE INDEX accounts_last_update_block_height_idx ON accounts(last_update_block_height);

-- FILL last_update_block_height for not deleted accounts
UPDATE accounts A
    SET last_update_block_height = B.block_height
FROM receipts R
    JOIN blocks B ON B.block_hash = R.included_in_block_hash
WHERE R.receipt_id = A.created_by_receipt_id AND A.deleted_by_receipt_id IS NULL;

-- FILL last_update_block_height for deleted accounts
UPDATE accounts A
    SET last_update_block_height = B.block_height
FROM receipts R
    JOIN blocks B ON B.block_hash = R.included_in_block_hash
WHERE R.receipt_id = A.created_by_receipt_id AND A.deleted_by_receipt_id IS NOT NULL;

-- Update accounts from genesis
UPDATE accounts SET last_update_block_height = 0 WHERE created_by_receipt_id IS NULL AND deleted_by_receipt_id IS NULL;

ALTER TABLE accounts
    ALTER COLUMN last_update_block_height SET NOT NULL;

-- ACCESS KEYS
ALTER TABLE access_keys
    ADD COLUMN last_update_block_height numeric(20, 0);

CREATE INDEX access_keys_last_update_block_height_idx ON access_keys(last_update_block_height);

-- FILL last_update_block_height for not deleted access_keys
UPDATE access_keys A
    SET last_update_block_height = B.block_height
FROM receipts R
    JOIN blocks B ON B.block_hash = R.included_in_block_hash
WHERE R.receipt_id = A.created_by_receipt_id AND A.deleted_by_receipt_id IS NULL;

-- FILL last_update_block_height for deleted access_keys
UPDATE access_keys A
    SET last_update_block_height = B.block_height
FROM receipts R
    JOIN blocks B ON B.block_hash = R.included_in_block_hash
WHERE R.receipt_id = A.deleted_by_receipt_id AND A.deleted_by_receipt_id IS NOT NULL;

-- Update access keys added from genesis
UPDATE access_keys SET last_update_block_height = 0 WHERE created_by_receipt_id IS NULL AND deleted_by_receipt_id IS NULL;

ALTER TABLE access_keys
    ALTER COLUMN last_update_block_height SET NOT NULL;
