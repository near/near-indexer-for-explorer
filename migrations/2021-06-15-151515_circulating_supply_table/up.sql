CREATE TABLE circulating_supply
(
    block_timestamp          numeric(20, 0) NOT NULL,
    block_hash               text           NOT NULL,
    value                    numeric(45, 0) NOT NULL,
    total_supply             numeric(45, 0) NOT NULL,
    lockups_number           numeric(45, 0) NOT NULL,
    active_lockups_number    numeric(45, 0) NOT NULL,
    foundation_locked_supply numeric(45, 0) NOT NULL,
    lockups_locked_supply    numeric(45, 0) NOT NULL
);

ALTER TABLE ONLY circulating_supply
    ADD CONSTRAINT circulating_supply_pkey PRIMARY KEY (block_hash);

CREATE INDEX circulating_supply_timestamp_idx ON circulating_supply USING btree (block_timestamp);

ALTER TABLE ONLY circulating_supply
    ADD CONSTRAINT circulating_supply_fk FOREIGN KEY (block_hash) REFERENCES blocks (block_hash) ON DELETE CASCADE;

CREATE VIEW lockups AS
(
SELECT accounts.account_id,
       blocks_start.block_height AS creation_block_height,
       blocks_end.block_height   AS deletion_block_height
FROM accounts
         LEFT JOIN receipts AS receipts_start ON accounts.created_by_receipt_id = receipts_start.receipt_id
         LEFT JOIN blocks AS blocks_start ON receipts_start.included_in_block_hash = blocks_start.block_hash
         LEFT JOIN receipts AS receipts_end ON accounts.deleted_by_receipt_id = receipts_end.receipt_id
         LEFT JOIN blocks AS blocks_end ON receipts_end.included_in_block_hash = blocks_end.block_hash
WHERE accounts.account_id like '%.lockup.near');
