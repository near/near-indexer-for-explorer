-- BLOCKS
ALTER TABLE blocks
    RENAME COLUMN block_height TO height;

ALTER TABLE blocks
    RENAME COLUMN block_hash TO hash;

ALTER TABLE blocks
    RENAME COLUMN prev_block_hash TO prev_hash;

ALTER TABLE blocks
    RENAME COLUMN block_timestamp TO "timestamp";

-- CHUNKS
ALTER TABLE chunks
    RENAME COLUMN chunk_hash TO hash;

-- TRANSACTIONS
ALTER TABLE transactions
    RENAME COLUMN signer_account_id TO signer_id;

ALTER TABLE transactions
    RENAME COLUMN signer_public_key TO public_key;

ALTER TABLE transactions
    RENAME COLUMN receiver_account_id TO receiver_id;

ALTER TABLE transaction_actions
    RENAME COLUMN index_in_transaction TO index;

-- RECEIPTS
ALTER TABLE receipt_actions
    RENAME COLUMN signer_account_id TO signer_id;
