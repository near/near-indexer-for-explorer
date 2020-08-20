-- Your SQL goes here
CREATE TABLE transactions (
	hash varchar(58) PRIMARY KEY NOT NULL,
	block_id numeric(45) NOT NULL,
	block_timestamp numeric(45, 0) NOT NULL,
    nonce numeric(58) NOT NULL,
    signer_id text NOT NULL,
    signer_public_key text NOT NULL,
	signature text NOT NULL,
    receiver_id text NOT NULL,
    receipt_conversion_gas_burnt numeric(58),
    receipt_conversion_tokens_burnt numeric(58),
    receipt_id varchar(58),
    -- status
	CONSTRAINT chunks_fk FOREIGN KEY (block_id) REFERENCES blocks(height) ON DELETE CASCADE
);
