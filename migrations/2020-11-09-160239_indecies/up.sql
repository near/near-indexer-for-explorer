-- TRANSACTIONS
CREATE INDEX tx_block_hash_idx ON transactions (included_in_block_hash);
CREATE INDEX tx_chunk_hash_idx ON transactions (included_in_chunk_hash);
CREATE INDEX tx_signer_account_id_idx ON transactions (signer_account_id);
CREATE INDEX tx_signer_public_key_idx ON transactions (signer_public_key);
CREATE INDEX tx_actions_tx_hash_idx ON transaction_actions (transaction_hash);

-- RECEIPTS
CREATE INDEX receipts_block_hash_idx ON receipts (included_in_block_hash);
CREATE INDEX receipts_chunk_hash_idx ON receipts (included_in_chunk_hash);
CREATE INDEX receipts_predecessor_account_id_idx ON receipts (predecessor_account_id);
CREATE INDEX receipts_receiver_account_id_idx ON receipts (receiver_account_id);

CREATE INDEX data_receipts_receipt_id_idx ON data_receipts (receipt_id);
CREATE INDEX action_receipt_signer_account_id_idx ON action_receipts (signer_account_id);
CREATE INDEX action_receipt_actions_receipt_id_idx ON action_receipt_actions (receipt_id);

CREATE INDEX action_receipt_output_data_output_from_receipt_id_idx ON action_receipt_output_data (output_from_receipt_id);
CREATE INDEX action_receipt_output_data_receiver_account_id_idx ON action_receipt_output_data (receiver_account_id);

CREATE INDEX action_receipt_input_data_input_to_receipt_id_idx ON action_receipt_input_data (input_to_receipt_id);
CREATE INDEX action_receipt_input_data_input_data_id_idx ON action_receipt_input_data (input_data_id);

-- EXECUTION OUTCOMES
CREATE INDEX execution_outcomes_receipt_id_idx ON execution_outcomes (receipt_id);
CREATE INDEX execution_outcomes_block_hash_idx ON execution_outcomes (executed_in_block_hash);
