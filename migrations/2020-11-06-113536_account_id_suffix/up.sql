ALTER TABLE receipts
    RENAME COLUMN predecessor_id TO predecessor_account_id;
ALTER TABLE receipts
    RENAME COLUMN receiver_id TO receiver_account_id;

ALTER TABLE receipt_action_output_data
    RENAME COLUMN receiver_id TO receiver_account_id;

ALTER TABLE execution_outcomes
    RENAME COLUMN executor_id TO executor_account_id;
