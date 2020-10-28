CREATE TABLE transactions (
    transaction_hash text PRIMARY KEY,
    block_hash text NOT NULL,
    chunk_hash text NOT NULL,
    index_in_chunk INT NOT NULL,
    block_timestamp numeric(20, 0) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
    signer_id text NOT NULL,
    public_key text NOT NULL,
    nonce numeric(20, 0) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
    receiver_id text NOT NULL,
    signature text NOT NULL,
    status execution_outcome_status NOT NULL,
    receipt_id text NOT NULL,
    receipt_conversion_gas_burnt numeric(20, 0), -- numeric(precision) 20 digits should be enough to store u64::MAX
    receipt_conversion_tokens_burnt numeric(45, 0), -- numeric(precision) 45 digits should be enough to store u128::MAX
    CONSTRAINT block_tx_fk FOREIGN KEY (block_hash) REFERENCES blocks(hash) ON DELETE CASCADE,
    CONSTRAINT chunk_tx_fk FOREIGN KEY (chunk_hash) REFERENCES chunks(hash) ON DELETE CASCADE
);
CREATE INDEX tx_timestamp_idx ON transactions (block_timestamp);
CREATE INDEX tx_index_in_chunk_idx ON transactions (index_in_chunk);

CREATE TABLE transaction_actions (
    transaction_hash text NOT NULL,
    index integer NOT NULL,
    action_kind action_type NOT NULL,
    args jsonb NOT NULL,
    CONSTRAINT tx_action_fk FOREIGN KEY (transaction_hash) REFERENCES transactions(transaction_hash) ON DELETE CASCADE,
    CONSTRAINT transaction_action_pk PRIMARY KEY (transaction_hash, index)
);

ALTER TABLE receipts ADD COLUMN transaction_hash text NOT NULL DEFAULT '',
    ADD CONSTRAINT tx_receipt_fk FOREIGN KEY (transaction_hash) REFERENCES transactions(transaction_hash) ON DELETE CASCADE;
