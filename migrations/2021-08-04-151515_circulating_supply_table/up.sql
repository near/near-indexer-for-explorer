CREATE TABLE aggregated__circulating_supply
(
    computed_at_block_timestamp       numeric(20, 0) NOT NULL,
    computed_at_block_hash            text           NOT NULL,
    circulating_tokens_supply         numeric(45, 0) NOT NULL,
    total_tokens_supply               numeric(45, 0) NOT NULL,
    total_lockup_contracts_count      integer        NOT NULL,
    unfinished_lockup_contracts_count integer        NOT NULL,
    foundation_locked_tokens          numeric(45, 0) NOT NULL,
    lockups_locked_tokens             numeric(45, 0) NOT NULL
);

ALTER TABLE ONLY aggregated__circulating_supply
    ADD CONSTRAINT aggregated__circulating_supply_pkey PRIMARY KEY (computed_at_block_hash);

CREATE INDEX aggregated__circulating_supply_timestamp_idx ON aggregated__circulating_supply USING btree (computed_at_block_timestamp);

ALTER TABLE ONLY aggregated__circulating_supply
    ADD CONSTRAINT aggregated__circulating_supply_fk FOREIGN KEY (computed_at_block_hash) REFERENCES blocks (block_hash) ON DELETE CASCADE;

CREATE VIEW aggregated__lockups AS
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
