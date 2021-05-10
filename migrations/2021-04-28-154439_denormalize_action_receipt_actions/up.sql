ALTER TABLE action_receipt_actions
    ADD COLUMN receipt_predecessor_account_id text NOT NULL DEFAULT '',
    ADD COLUMN receipt_receiver_account_id text NOT NULL DEFAULT '',
    ADD COLUMN receipt_included_in_block_timestamp numeric(20, 0) NOT NULL DEFAULT 0;

UPDATE action_receipt_actions
SET receipt_predecessor_account_id = receipts.predecessor_account_id,
    receipt_receiver_account_id = receipts.receiver_account_id,
    receipt_included_in_block_timestamp = receipts.included_in_block_timestamp
    FROM receipts
WHERE action_receipt_actions.receipt_id = receipts.receipt_id;

ALTER TABLE action_receipt_actions
    ALTER COLUMN receipt_predecessor_account_id DROP DEFAULT,
    ALTER COLUMN receipt_receiver_account_id DROP DEFAULT,
    ALTER COLUMN receipt_included_in_block_timestamp DROP DEFAULT;
