CREATE TYPE access_key_permission_type AS ENUM ('NOT_APPLICABLE', 'FULL_ACCESS', 'FUNCTION_CALL');

CREATE TABLE access_keys (
    public_key text NOT NULL,
    account_id text NOT NULL,
    created_by_receipt_id text,
    deleted_by_receipt_id text,
    "permission" access_key_permission_type NOT NULL,
    CONSTRAINT access_keys_pk PRIMARY KEY (public_key, account_id),
    CONSTRAINT account_fk FOREIGN KEY (account_id) REFERENCES accounts(account_id) ON DELETE CASCADE,
    CONSTRAINT created_by_receipt_fk FOREIGN KEY (created_by_receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT deleted_by_receipt_fk FOREIGN KEY (deleted_by_receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT access_key_unique UNIQUE (public_key, account_id)
);
CREATE INDEX access_keys_public_key_idx ON access_keys (public_key);
CREATE INDEX access_keys_account_id_idx ON access_keys (account_id);
