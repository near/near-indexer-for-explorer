-- Your SQL goes here
CREATE TABLE chunks (
	block_id numeric(45) NOT NULL,
	hash varchar(58) PRIMARY KEY NOT NULL,
	shard_id numeric(20) NOT NULL,
	signature text NOT NULL,
	gas_limit numeric(45) NOT NULL,
	gas_used numeric(45) NOT NULL,
	height_created numeric(45) NOT NULL,
	height_included numeric(45) NOT NULL,
	CONSTRAINT chunks_fk FOREIGN KEY (block_id) REFERENCES blocks(height) ON DELETE CASCADE
);