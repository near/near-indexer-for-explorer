-- BLOCKS
ALTER TABLE blocks
    RENAME COLUMN height TO block_height;

ALTER TABLE blocks
    RENAME COLUMN hash TO block_hash;

ALTER TABLE blocks
    RENAME COLUMN prev_hash TO prev_block_hash;

ALTER TABLE blocks
    RENAME COLUMN "timestamp" TO block_timestamp;

-- CHUNKS
ALTER TABLE chunks
    RENAME COLUMN hash TO chunk_hash;

-- TRANSACTIONS
ALTER TABLE transactions
    RENAME COLUMN signer_id TO signer_account_id;

ALTER TABLE transactions
    RENAME COLUMN public_key TO signer_public_key;

ALTER TABLE transactions
    RENAME COLUMN receiver_id TO receiver_account_id;

ALTER TABLE transaction_actions
    RENAME COLUMN index TO index_in_transaction;

-- RECEIPTS
ALTER TABLE receipt_actions
    RENAME COLUMN signer_id TO signer_account_id;
