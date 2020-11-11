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

ALTER TABLE chunks
    RENAME COLUMN included_in_block_hash TO block_hash;

-- TRANSACTIONS
ALTER TABLE transactions
    RENAME COLUMN signer_account_id TO signer_id;

ALTER TABLE transactions
    RENAME COLUMN signer_public_key TO public_key;

ALTER TABLE transactions
    RENAME COLUMN receiver_account_id TO receiver_id;

ALTER TABLE transactions
    RENAME COLUMN converted_into_receipt_id TO receipt_id;

ALTER TABLE transactions
    RENAME COLUMN included_in_block_hash TO block_hash;

ALTER TABLE transactions
    RENAME COLUMN included_in_chunk_hash TO chunk_hash;

ALTER TABLE transaction_actions
    RENAME COLUMN index_in_transaction TO index;

-- RECEIPTS
ALTER TABLE receipts
    RENAME COLUMN included_in_block_hash TO block_hash;

ALTER TABLE receipts
    RENAME COLUMN included_in_block_timestamp TO block_timestamp;

ALTER TABLE receipts
    RENAME COLUMN included_in_chunk_hash TO chunk_hash;

ALTER TABLE receipts
    RENAME COLUMN originated_from_transaction_hash TO transaction_hash;

ALTER TABLE action_receipts
    RENAME COLUMN signer_account_id TO signer_id;

ALTER TABLE action_receipt_actions
    RENAME COLUMN index_in_action_receipt TO index;

ALTER TABLE action_receipt_input_data
    RENAME COLUMN input_data_id TO data_id;

ALTER TABLE action_receipt_input_data
    RENAME COLUMN input_to_receipt_id TO receipt_id;

ALTER TABLE action_receipt_output_data
    RENAME COLUMN output_data_id TO data_id;

ALTER TABLE action_receipt_output_data
    RENAME COLUMN output_from_receipt_id TO receipt_id;

ALTER TABLE action_receipts RENAME TO receipt_actions;

ALTER TABLE action_receipt_actions RENAME TO receipt_action_actions;

ALTER TABLE action_receipt_input_data RENAME TO receipt_action_input_data;

ALTER TABLE action_receipt_output_data RENAME TO receipt_action_output_data;

ALTER TABLE data_receipts RENAME TO receipt_data;

-- EXECUTION OUTCOMES
ALTER TABLE execution_outcomes
    RENAME COLUMN executed_in_block_hash TO block_hash;

ALTER TABLE execution_outcome_receipts
    RENAME COLUMN executed_receipt_id TO execution_outcome_receipt_id;

ALTER TABLE execution_outcome_receipts
    RENAME COLUMN produced_receipt_id TO receipt_id;

ALTER TABLE execution_outcome_receipts
    RENAME COLUMN index_in_execution_outcome TO index;

-- TYPES
ALTER TYPE receipt_kind RENAME TO receipt_type;
ALTER TYPE action_kind RENAME TO action_type;
