CREATE TYPE receipt_type AS ENUM ('ACTION', 'DATA');
CREATE TYPE action_type AS ENUM ('CreateAccount', 'DeployContract', 'FunctionCall', 'Transfer', 'Stake', 'AddKey', 'DeleteKey', 'DeleteAccount');

CREATE TABLE receipts (
    receipt_id text PRIMARY KEY NOT NULL,
    block_height numeric(45, 0),
    predecessor_id text NOT NULL,
    receiver_id text NOT NULL,
    type receipt_type NOT NULL
);
CREATE TABLE receipt_data (
    id bigserial PRIMARY KEY,
    receipt_id text NOT NULL,
    data_id text NOT NULL,
    data bytea
);

CREATE TABLE receipt_actions (
    id bigserial PRIMARY KEY,
    receipt_id text NOT NULL,
    signer_id text NOT NULL,
    signer_public_key text NOT NULL,
    gas_price numeric(45, 0) NOT NULL
);

CREATE TABLE receipt_action_actions (
    id bigserial PRIMARY KEY,
    receipt_id text NOT NULL,
    index integer NOT NULL,
    type action_type NOT NULL,
    args jsonb
);

CREATE TABLE receipt_action_output_data (
    id bigserial PRIMARY KEY NOT NULL,
    receipt_id text NOT NULL,
    data_id varchar(58) NOT NULL,
    receiver_id text NOT NULL
);

CREATE TABLE receipt_action_input_data (
    id bigserial PRIMARY KEY NOT NULL,
    receipt_id text NOT NULL,
    data_id text NOT NULL
);
