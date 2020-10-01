-- Your SQL goes here
CREATE TABLE chunks (
	block_hash text NOT NULL,
	hash text PRIMARY KEY NOT NULL,
	shard_id numeric(20) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	signature text NOT NULL,
	gas_limit numeric(20) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	gas_used numeric(20) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	CONSTRAINT chunks_fk FOREIGN KEY (block_hash) REFERENCES blocks(hash) ON DELETE CASCADE
);
