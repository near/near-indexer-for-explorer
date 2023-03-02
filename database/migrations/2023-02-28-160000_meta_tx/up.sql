ALTER TYPE action_kind ADD VALUE IF NOT EXISTS 'DELEGATE_ACTION';

ALTER TABLE transaction_actions
    ADD COLUMN is_delegate_action BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN delegate_parameters JSONB,
    ADD COLUMN delegate_parent_index_in_transaction INTEGER;

ALTER TABLE action_receipt_actions
    ADD COLUMN is_delegate_action BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN delegate_parameters JSONB,
    ADD COLUMN delegate_parent_index_in_action_receipt INTEGER;

ALTER TABLE transaction_actions ALTER COLUMN is_delegate_action DROP DEFAULT;
ALTER TABLE action_receipt_actions ALTER COLUMN is_delegate_action DROP DEFAULT;
