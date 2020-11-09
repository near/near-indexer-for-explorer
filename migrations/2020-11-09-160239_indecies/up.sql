-- TRANSACTIONS
CREATE INDEX tx_block_hash_idx ON transactions (block_hash);
CREATE INDEX tx_chunk_hash_idx ON transactions (chunk_hash);
CREATE INDEX tx_signer_account_id_idx ON transactions (signer_account_id);
CREATE INDEX tx_signer_public_key_idx ON transactions (signer_public_key);
CREATE INDEX tx_actions_tx_hash_idx ON transaction_actions (transaction_hash);

-- RECEIPTS
CREATE INDEX receipts_block_hash_idx ON receipts (block_hash);
CREATE INDEX receipts_chunk_hash_idx ON receipts (chunk_hash);
CREATE INDEX receipts_predecessor_account_id_idx ON receipts (predecessor_account_id);
CREATE INDEX receipts_receiver_account_id_idx ON receipts (receiver_account_id);

CREATE INDEX receipt_data_receipt_id_idx ON receipt_data (receipt_id);
CREATE INDEX receipt_actions_signer_account_id_idx ON receipt_actions (signer_account_id);
CREATE INDEX receipt_action_actions_receipt_id_idx ON receipt_action_actions (receipt_id);

CREATE INDEX receipt_action_output_data_output_from_receipt_id_idx ON receipt_action_output_data (output_from_receipt_id);
CREATE INDEX receipt_action_output_data_receiver_account_id_idx ON receipt_action_output_data (receiver_account_id);

CREATE INDEX receipt_action_input_data_input_to_receipt_id_idx ON receipt_action_input_data (input_to_receipt_id);
CREATE INDEX receipt_action_input_data_input_data_id_idx ON receipt_action_input_data (input_data_id);

-- EXECUTION OUTCOMES
CREATE INDEX execution_outcomes_receipt_id_idx ON execution_outcomes (receipt_id);
CREATE INDEX execution_outcomes_block_hash_idx ON execution_outcomes (block_hash);
