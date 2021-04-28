ALTER TABLE action_receipt_actions
    DROP COLUMN predecessor_account_id,
    DROP COLUMN receiver_account_id,
    DROP COLUMN included_in_block_timestamp;
