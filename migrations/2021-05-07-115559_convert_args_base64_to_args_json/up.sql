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
WHERE action_kind = 'FUNCTION_CALL' AND receipt_receiver_account_id != 'client.bridge.near';
