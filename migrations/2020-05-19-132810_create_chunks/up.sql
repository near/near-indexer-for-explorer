-- Your SQL goes here
CREATE TABLE chunks (
	block_id numeric(45) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
	hash text PRIMARY KEY NOT NULL,
	shard_id numeric(20) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	signature text NOT NULL,
	gas_limit numeric(20) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	gas_used numeric(20) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	height_created numeric(45) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
	height_included numeric(45) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
	CONSTRAINT chunks_fk FOREIGN KEY (block_id) REFERENCES blocks(height) ON DELETE CASCADE
);
