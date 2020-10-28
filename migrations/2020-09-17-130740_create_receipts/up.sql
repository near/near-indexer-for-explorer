CREATE TYPE receipt_type AS ENUM ('ACTION', 'DATA');
CREATE TYPE action_type AS ENUM (
    'CREATE_ACCOUNT',
    'DEPLOY_CONTRACT',
    'FUNCTION_CALL',
    'TRANSFER',
    'STAKE',
    'ADD_KEY',
    'DELETE_KEY',
    'DELETE_ACCOUNT'
);

CREATE TABLE receipts (
    receipt_id text PRIMARY KEY,
    block_hash text NOT NULL,
    chunk_hash text NOT NULL,
    index_in_chunk INT NOT NULL,
    block_timestamp numeric(20, 0) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
    predecessor_id text NOT NULL,
    receiver_id text NOT NULL,
    receipt_kind receipt_type NOT NULL,
    CONSTRAINT block_receipts_fk FOREIGN KEY (block_hash) REFERENCES blocks(hash) ON DELETE CASCADE,
    CONSTRAINT chunk_receipts_fk FOREIGN KEY (chunk_hash) REFERENCES chunks(hash) ON DELETE CASCADE
);
CREATE INDEX receipts_timestamp_idx ON receipts (block_timestamp);
CREATE INDEX receipts_index_in_chunk_idx ON receipts (index_in_chunk);

CREATE TABLE receipt_data (
    data_id text PRIMARY KEY,
    receipt_id text NOT NULL,
    data bytea
);

CREATE TABLE receipt_actions (
    receipt_id text PRIMARY KEY,
    signer_id text NOT NULL,
    signer_public_key text NOT NULL,
    gas_price numeric(45, 0) NOT NULL -- numeric(precision) 45 digits should be enough to store u128::MAX
);

CREATE TABLE receipt_action_actions (
    receipt_id text NOT NULL,
    index integer NOT NULL,
    action_kind action_type NOT NULL,
    args jsonb NOT NULL,
    CONSTRAINT action_receipt_fk FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE,
    CONSTRAINT receipt_action_action_pk PRIMARY KEY (receipt_id, index)
);

CREATE TABLE receipt_action_output_data (
    data_id text NOT NULL,
    receipt_id text NOT NULL,
    receiver_id text NOT NULL,
    CONSTRAINT action_output_pk PRIMARY KEY (data_id, receipt_id)
);

CREATE TABLE receipt_action_input_data (
    data_id text NOT NULL,
    receipt_id text NOT NULL,
    CONSTRAINT action_input_pk PRIMARY KEY (data_id, receipt_id)
);
