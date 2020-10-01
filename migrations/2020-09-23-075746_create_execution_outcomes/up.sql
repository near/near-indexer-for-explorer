CREATE TYPE execution_outcome_status AS ENUM ('UNKNOWN', 'FAILURE', 'SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID');

CREATE TABLE execution_outcomes (
    receipt_id text PRIMARY KEY,
    block_hash text NOT NULL,
    gas_burnt numeric(20, 0) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
    tokens_burnt numeric(45,0) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
    executor_id text NOT NULL,
    status execution_outcome_status NOT NULL,
    CONSTRAINT receipt_execution_outcome_fk FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT block_hash_execution_outcome_fk FOREIGN KEY (block_hash) REFERENCES blocks(hash) ON DELETE CASCADE
);

CREATE TABLE execution_outcome_receipts (
    execution_outcome_receipt_id text NOT NULL,
    index int NOT NULL,
    receipt_id text NOT NULL,
    CONSTRAINT execution_outcome_fk FOREIGN KEY (execution_outcome_receipt_id) REFERENCES execution_outcomes(receipt_id) ON DELETE CASCADE,
    CONSTRAINT receipts_fk FOREIGN KEY (execution_outcome_receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT execution_outcome_receipt_pk PRIMARY KEY (execution_outcome_receipt_id, index, receipt_id)
);
