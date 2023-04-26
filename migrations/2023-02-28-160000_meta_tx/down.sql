-- We do not drop `DELEGATE_ACTION` here because we can't remove items from enum in Postgres.
-- To be honest, it does not sound as a big problem. `IF_NOT_EXISTS` will prevent re-adding it.

ALTER TABLE transaction_actions
    DROP COLUMN is_delegate_action,
    DROP COLUMN delegate_parameters,
    DROP COLUMN delegate_parent_index_in_transaction;

ALTER TABLE action_receipt_actions
    DROP COLUMN is_delegate_action,
    DROP COLUMN delegate_parameters,
    DROP COLUMN delegate_parent_index_in_action_receipt;