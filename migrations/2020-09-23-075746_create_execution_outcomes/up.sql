CREATE TYPE execution_outcome_status AS ENUM ('UNKNOWN', 'FAILURE', 'SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID');

CREATE TABLE execution_outcomes (
    receipt_id text PRIMARY KEY,
    block_hash text NOT NULL,
    executed_in_block_timestamp numeric(20, 0) NOT NULL,
    executed_in_chunk_hash text NOT NULL,
    index_in_chunk INT NOT NULL,
    gas_burnt numeric(20, 0) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
    tokens_burnt numeric(45,0) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
    executor_id text NOT NULL,
    status execution_outcome_status NOT NULL,
    CONSTRAINT receipt_execution_outcome_fk FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT chunk_hash_outcome_fk FOREIGN KEY (executed_in_chunk_hash) REFERENCES chunks(hash) ON DELETE CASCADE,
    CONSTRAINT block_hash_execution_outcome_fk FOREIGN KEY (block_hash) REFERENCES blocks(hash) ON DELETE CASCADE
);

CREATE INDEX execution_outcome_executed_in_chunk_hash_idx ON execution_outcomes (executed_in_chunk_hash);
CREATE INDEX execution_outcome_executed_in_block_timestamp ON execution_outcomes (executed_in_block_timestamp);

CREATE TABLE execution_outcome_receipts (
    execution_outcome_receipt_id text NOT NULL,
    index int NOT NULL,
    receipt_id text NOT NULL,
    CONSTRAINT execution_outcome_fk FOREIGN KEY (execution_outcome_receipt_id) REFERENCES execution_outcomes(receipt_id) ON DELETE CASCADE,
    CONSTRAINT receipts_fk FOREIGN KEY (execution_outcome_receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT execution_outcome_receipt_pk PRIMARY KEY (execution_outcome_receipt_id, index, receipt_id)
);
