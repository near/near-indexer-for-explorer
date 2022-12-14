ALTER TABLE action_receipt_actions
    DROP COLUMN receipt_predecessor_account_id,
    DROP COLUMN receipt_receiver_account_id,
    DROP COLUMN receipt_included_in_block_timestamp;
