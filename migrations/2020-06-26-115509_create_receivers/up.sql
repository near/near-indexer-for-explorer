-- Your SQL goes here
CREATE TABLE actions_output_data (
    id numeric(45) PRIMARY KEY NOT NULL,
    receipt_id varchar(58) NOT NULL,
    data_id varchar(58) NOT NULL,
    receiver_id text NOT NULL
);

CREATE TABLE actions_input_data (
    id numeric(45) PRIMARY KEY NOT NULL,
    receipt_id varchar(58) NOT NULL,
    data_id varchar(58) NOT NULL
);
