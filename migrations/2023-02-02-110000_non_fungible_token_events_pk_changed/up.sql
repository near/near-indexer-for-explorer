-- This command is heavy for the full DB, consider adding CONCURRENTLY
CREATE UNIQUE INDEX assets__non_fungible_idx_tmp
    ON assets__non_fungible_token_events (emitted_for_receipt_id, emitted_index_of_event_entry_in_shard);

-- Next block runs ~1 sec even on the full DB
-- If you apply this manually, uncomment BEGIN TRANSACTION and COMMIT

-- BEGIN TRANSACTION;
SAVEPOINT change_nft_pks;
ALTER TABLE assets__non_fungible_token_events
    DROP CONSTRAINT assets__non_fungible_token_events_pkey;
ALTER TABLE assets__non_fungible_token_events
    DROP CONSTRAINT assets__non_fungible_token_events_unique;
-- This command will automatically rename assets__non_fungible_idx_tmp to assets__non_fungible_token_events_pkey
ALTER TABLE assets__non_fungible_token_events
    ADD CONSTRAINT assets__non_fungible_token_events_pkey PRIMARY KEY USING INDEX assets__non_fungible_idx_tmp;
RELEASE SAVEPOINT change_nft_pks;
-- COMMIT;