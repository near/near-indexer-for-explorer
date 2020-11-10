-- TRANSACTIONS
DROP INDEX tx_block_hash_idx;
DROP INDEX tx_chunk_hash_idx;
DROP INDEX tx_signer_account_id_idx;
DROP INDEX tx_signer_public_key_idx;
DROP INDEX tx_actions_tx_hash_idx;

-- RECEIPTS
DROP INDEX receipts_block_hash_idx;
DROP INDEX receipts_chunk_hash_idx;
DROP INDEX receipts_predecessor_account_id_idx;
DROP INDEX receipts_receiver_account_id_idx;

DROP INDEX data_receipts_receipt_id_idx;
DROP INDEX action_receipt_signer_account_id_idx;
DROP INDEX action_receipt_actions_receipt_id_idx;

DROP INDEX action_receipt_output_data_output_from_receipt_id_idx;
DROP INDEX action_receipt_output_data_receiver_account_id_idx;

DROP INDEX action_receipt_input_data_input_to_receipt_id_idx;
DROP INDEX action_receipt_input_data_input_data_id_idx;

-- EXECUTION OUTCOMES
DROP INDEX execution_outcomes_receipt_id_idx;
DROP INDEX execution_outcomes_block_hash_idx;
