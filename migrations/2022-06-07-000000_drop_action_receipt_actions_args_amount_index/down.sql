CREATE INDEX action_receipt_actions_args_amount_idx ON action_receipt_actions((args->'args_json'->>'amount'))
    WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL' AND (action_receipt_actions.args->>'args_json') IS NOT NULL;
