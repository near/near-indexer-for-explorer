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

CREATE OR REPLACE FUNCTION decode_or_null(bytea) RETURNS jsonb
   LANGUAGE plpgsql AS
$$BEGIN
   RETURN convert_from($1, 'UTF8')::jsonb;
EXCEPTION
   WHEN others THEN
      RAISE WARNING '%', SQLERRM;
RETURN '{}'::jsonb;

END;$$;

UPDATE action_receipt_actions
    SET args = jsonb_set(args, '{args_json}', decode_or_null(decode(args->>'args_base64', 'base64')), true)
WHERE action_kind = 'FUNCTION_CALL' AND receiver_account_id != 'client.bridge.near';

ALTER TABLE action_receipt_actions
    ALTER COLUMN predecessor_account_id DROP DEFAULT,
    ALTER COLUMN receiver_account_id DROP DEFAULT,
    ALTER COLUMN included_in_block_timestamp DROP DEFAULT;

CREATE INDEX action_receipt_actions_predecessor_account_id_idx ON action_receipt_actions(predecessor_account_id);
CREATE INDEX action_receipt_actions_receiver_account_id_idx ON action_receipt_actions(receiver_account_id);
CREATE INDEX action_receipt_actions_included_in_block_timestamp_idx ON action_receipt_actions(included_in_block_timestamp);

CREATE INDEX action_receipt_actions_args_function_call_idx ON action_receipt_actions((args->>'method_name'))
    WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL';

CREATE INDEX action_receipt_actions_args_amount_idx ON action_receipt_actions((args->'args_json'->>'amount'))
    WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL' AND (action_receipt_actions.args->>'args_json') IS NOT NULL;

CREATE INDEX action_receipt_actions_args_receiver_id_idx ON action_receipt_actions((args->'args_json'->>'receiver_id'))
    WHERE action_receipt_actions.action_kind = 'FUNCTION_CALL' AND (action_receipt_actions.args->>'args_json') IS NOT NULL;
