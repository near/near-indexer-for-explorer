ALTER TABLE action_receipt_actions
    ADD COLUMN predecessor_account_id text NOT NULL DEFAULT '',
    ADD COLUMN receiver_account_id text NOT NULL DEFAULT '',
    ADD COLUMN included_in_block_timestamp numeric(20, 0) NOT NULL DEFAULT 0;


UPDATE action_receipt_actions
    SET predecessor_account_id = receipts.predecessor_account_id,
        receiver_account_id = receipts.receiver_account_id,
        included_in_block_timestamp = receipts.included_in_block_timestamp
    FROM receipts
    WHERE action_receipt_actions.receipt_id = receipts.receipt_id;


ALTER TABLE action_receipt_actions
    ALTER COLUMN predecessor_account_id DROP DEFAULT,
    ALTER COLUMN receiver_account_id DROP DEFAULT,
    ALTER COLUMN included_in_block_timestamp DROP DEFAULT;
