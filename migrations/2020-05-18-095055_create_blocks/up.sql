-- Your SQL goes here
CREATE TABLE blocks (
	height numeric(20, 0) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	hash text PRIMARY KEY ,
	prev_hash text NOT NULL,
	timestamp numeric(20, 0) NOT NULL, -- numeric(precision) 20 digits should be enough to store u64::MAX
	total_supply numeric(45, 0) NOT NULL, -- numeric(precision) 45 digits should be enough to store u128::MAX
	gas_price numeric(45, 0) NOT NULL -- numeric(precision) 45 digits should be enough to store u128::MAX
);
CREATE INDEX blocks_height_idx ON blocks (height);
CREATE INDEX blocks_hash_idx ON blocks (hash);
CREATE INDEX blocks_prev_hash_idx ON blocks (prev_hash);
CREATE INDEX blocks_timestamp_idx ON blocks (timestamp);
