-- TRANSACTIONS
CREATE INDEX transactions_signer_account_id_idx ON transactions (signer_account_id);
CREATE INDEX transactions_signer_public_key_idx ON transactions (signer_public_key);

-- RECEIPTS
CREATE INDEX receipts_predecessor_account_id_idx ON receipts (predecessor_account_id);
CREATE INDEX receipts_receiver_account_id_idx ON receipts (receiver_account_id);

CREATE INDEX data_receipts_receipt_id_idx ON data_receipts (receipt_id);
CREATE INDEX action_receipt_signer_account_id_idx ON action_receipts (signer_account_id);

CREATE INDEX action_receipt_output_data_output_from_receipt_id_idx ON action_receipt_output_data (output_from_receipt_id);
CREATE INDEX action_receipt_output_data_receiver_account_id_idx ON action_receipt_output_data (receiver_account_id);

CREATE INDEX action_receipt_input_data_input_to_receipt_id_idx ON action_receipt_input_data (input_to_receipt_id);
CREATE INDEX action_receipt_input_data_input_data_id_idx ON action_receipt_input_data (input_data_id);

ALTER INDEX action_output_data_id_idx RENAME TO action_receipt_output_data_output_data_id_idx;
ALTER INDEX tx_timestamp_idx RENAME TO transactions_included_in_block_timestamp_idx;
ALTER INDEX chunks_block_hash_idx RENAME TO chunks_included_in_block_hash_idx;
DROP INDEX blocks_hash_idx;
