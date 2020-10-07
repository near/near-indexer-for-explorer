CREATE TABLE accounts (
    id bigserial PRIMARY KEY,
    account_id text UNIQUE NOT NULL,
    created_by_receipt_id text NOT NULL,
    deleted_by_receipt_id text,
    CONSTRAINT created_receipt_fk FOREIGN KEY (created_by_receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT deleted_receipt_fk FOREIGN KEY (deleted_by_receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE
);
