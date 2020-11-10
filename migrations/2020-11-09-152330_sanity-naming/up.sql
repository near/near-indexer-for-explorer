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

ALTER TABLE transactions
    RENAME COLUMN receipt_id TO converted_into_receipt_id;

ALTER TABLE transactions
    RENAME COLUMN block_hash TO included_in_block_hash;

ALTER TABLE transactions
    RENAME COLUMN chunk_hash TO included_in_chunk_hash;

ALTER TABLE transaction_actions
    RENAME COLUMN index TO index_in_transaction;

-- RECEIPTS
ALTER TABLE receipts
    RENAME COLUMN block_hash TO included_in_block_hash;

ALTER TABLE receipts
    RENAME COLUMN block_timestamp TO included_in_block_timestamp;

ALTER TABLE receipts
    RENAME COLUMN chunk_hash TO included_in_chunk_hash;

ALTER TABLE receipts
    RENAME COLUMN transaction_hash TO included_in_transaction_hash;

ALTER TABLE receipt_actions RENAME TO action_receipts;

ALTER TABLE receipt_action_actions RENAME TO action_receipt_actions;

ALTER TABLE receipt_action_input_data RENAME TO action_receipt_input_data;

ALTER TABLE receipt_action_output_data RENAME TO action_receipt_output_data;

ALTER TABLE receipt_data RENAME TO data_receipts;

ALTER TABLE action_receipts
    RENAME COLUMN signer_id TO signer_account_id;

ALTER TABLE action_receipt_actions
    RENAME COLUMN index TO index_in_action_receipt;

ALTER TABLE action_receipt_input_data
    RENAME COLUMN data_id TO input_data_id;

ALTER TABLE action_receipt_input_data
    RENAME COLUMN receipt_id TO input_to_receipt_id;

ALTER TABLE action_receipt_output_data
    RENAME COLUMN data_id TO output_data_id;

ALTER TABLE action_receipt_output_data
    RENAME COLUMN receipt_id TO output_from_receipt_id;

ALTER TABLE action_receipt_output_data
    RENAME COLUMN receiver_id TO receiver_account_id;

-- EXECUTION OUTCOMES
ALTER TABLE execution_outcomes
    RENAME COLUMN block_hash TO executed_in_block_hash;

ALTER TABLE execution_outcome_receipts
    RENAME COLUMN execution_outcome_receipt_id TO executed_receipt_id;

ALTER TABLE execution_outcome_receipts
    RENAME COLUMN receipt_id TO produced_receipt_id;

ALTER TABLE execution_outcome_receipts
    RENAME COLUMN index TO index_in_execution_outcome;
