CREATE TYPE execution_outcome_status AS ENUM ('UNKNOWN', 'FAILURE', 'SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID');

CREATE TABLE execution_outcomes (
    receipt_id bytea PRIMARY KEY,
    block_hash bytea NOT NULL,
    gas_burnt numeric(45, 0) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
    tokens_burnt numeric(45,0) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
    executor_id text NOT NULL,
    status execution_outcome_status NOT NULL
);

CREATE TABLE execution_outcome_receipts (
    execution_outcome_receipt_id bytea NOT NULL,
    index int NOT NULL,
    receipt_id bytea NOT NULL,
    CONSTRAINT execution_outcome_fk FOREIGN KEY (execution_outcome_receipt_id) REFERENCES execution_outcomes(receipt_id) ON DELETE CASCADE,
    CONSTRAINT execution_outcome_receipt_pk PRIMARY KEY (execution_outcome_receipt_id, index, receipt_id)
);
