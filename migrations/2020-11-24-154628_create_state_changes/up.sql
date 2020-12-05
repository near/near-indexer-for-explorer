CREATE TYPE state_change_reason_kind AS ENUM (
   'TRANSACTION_PROCESSING',
   'ACTION_RECEIPT_PROCESSING_STARTED',
   'ACTION_RECEIPT_GAS_REWARD',
   'RECEIPT_PROCESSING',
   'POSTPONED_RECEIPT',
   'UPDATED_DELAYED_RECEIPTS',
   'VALIDATOR_ACCOUNTS_UPDATE'
);

CREATE TABLE account_changes (
    id bigserial PRIMARY KEY,
    affected_account_id text NOT NULL,
    changed_in_block_timestamp numeric(20) NOT NULL,
    changed_in_block_hash text NOT NULL,
    caused_by_transaction_hash text,
    caused_by_receipt_id text,
    update_reason state_change_reason_kind NOT NULL,
    affected_account_nonstaked_balance numeric(45) NOT NULL,
    affected_account_staked_balance numeric(45) NOT NULL,
    affected_account_storage_usage numeric(20) NOT NULL,
    CONSTRAINT account_id_fk FOREIGN KEY (affected_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE,
    CONSTRAINT block_hash_fk FOREIGN KEY (changed_in_block_hash) REFERENCES blocks (block_hash) ON DELETE CASCADE,
    CONSTRAINT transaction_hash_fk FOREIGN KEY (caused_by_transaction_hash) REFERENCES transactions (transaction_hash) ON DELETE CASCADE,
    CONSTRAINT receipt_id_fk FOREIGN KEY (caused_by_receipt_id) REFERENCES receipts (receipt_id) ON DELETE CASCADE,
    UNIQUE (affected_account_id, changed_in_block_hash, caused_by_transaction_hash, caused_by_receipt_id)
);

CREATE INDEX account_changes_changed_in_block_timestamp_idx ON account_changes(changed_in_block_timestamp);
CREATE INDEX account_changes_changed_in_block_hash_idx ON account_changes(changed_in_block_hash);
CREATE INDEX account_changes_changed_in_caused_by_transaction_hash_idx ON account_changes(caused_by_transaction_hash);
CREATE INDEX account_changes_changed_in_caused_by_receipt_id_idx ON account_changes(caused_by_receipt_id);
CREATE INDEX account_changes_affected_account_id_idx ON account_changes (affected_account_id);
