-- If your DB is not empty, you need to turn off the Indexer, then apply the migration, then switch to 0.12.0

ALTER TYPE action_kind ADD VALUE IF NOT EXISTS 'DELEGATE_ACTION';

-- For all happy users of Postgres 11+, this should run fast
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