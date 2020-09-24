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
    block_height numeric(45, 0), -- numeric(precision) 45 digits should be enough to store u128::MAX
--     chunk_hash bytea NOT NULL,
    predecessor_id text NOT NULL,
    receiver_id text NOT NULL,
    receipt_kind receipt_type NOT NULL,
    CONSTRAINT block_receipts_fk FOREIGN KEY (block_height) REFERENCES blocks(height) ON DELETE CASCADE
--     CONSTRAINT chunk_receipts_fk FOREIGN KEY (chunk_hash) REFERENCES chunks(hash) ON DELETE CASCADE
);
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
    id bigserial PRIMARY KEY,
    receipt_id text NOT NULL,
    index integer NOT NULL,
    action_kind action_type NOT NULL,
    args jsonb NOT NULL
);

CREATE TABLE receipt_action_output_data (
    data_id text PRIMARY KEY,
    receipt_id text NOT NULL,
    receiver_id text NOT NULL
);

CREATE TABLE receipt_action_input_data (
    data_id text PRIMARY KEY,
    receipt_id text NOT NULL
);
