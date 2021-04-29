ALTER TABLE action_receipt_actions
    DROP COLUMN predecessor_account_id,
    DROP COLUMN receiver_account_id,
    DROP COLUMN included_in_block_timestamp;

DROP INDEX action_receipt_actions_args_function_call_idx;
DROP INDEX action_receipt_actions_args_amount_idx;
DROP INDEX action_receipt_actions_args_receiver_id_idx;
