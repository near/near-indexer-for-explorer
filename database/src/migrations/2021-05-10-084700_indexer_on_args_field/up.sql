CREATE INDEX action_receipt_actions_args_function_call_idx ON action_receipt_actions((args->>'method_name'))
    WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL';

CREATE INDEX action_receipt_actions_args_amount_idx ON action_receipt_actions((args->'args_json'->>'amount'))
    WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL' AND (action_receipt_actions.args->>'args_json') IS NOT NULL;

CREATE INDEX action_receipt_actions_args_receiver_id_idx ON action_receipt_actions((args->'args_json'->>'receiver_id'))
    WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL' AND (action_receipt_actions.args->>'args_json') IS NOT NULL;
