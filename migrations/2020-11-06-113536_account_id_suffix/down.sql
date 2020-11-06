ALTER TABLE receipts
    RENAME COLUMN predecessor_account_id TO predecessor_id;

ALTER TABLE receipts
    RENAME COLUMN receiver_account_id TO receiver_id;

ALTER TABLE receipt_action_output_data
    RENAME COLUMN receiver_account_id TO receiver_id;

ALTER TABLE execution_outcomes
    RENAME COLUMN executor_account_id TO executor_id;
