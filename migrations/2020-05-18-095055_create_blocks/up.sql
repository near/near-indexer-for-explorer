-- Your SQL goes here
CREATE TABLE blocks (
	height numeric(45, 0) PRIMARY KEY  NOT NULL,
	hash varchar(58) NOT NULL,
	prev_hash varchar(58) NOT NULL,
	timestamp numeric(45, 0) NOT NULL,
	total_supply numeric(45, 0) NOT NULL,
	gas_limit numeric(45, 0) NOT NULL,
	gas_used numeric(45, 0) NOT NULL,
	gas_price numeric(45, 0) NOT NULL
);
CREATE INDEX blocks_height_idx ON blocks (height, timestamp);
