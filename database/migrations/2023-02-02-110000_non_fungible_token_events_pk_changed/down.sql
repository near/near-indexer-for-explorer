-- These commands are heavy for the full DB, consider adding CONCURRENTLY
CREATE UNIQUE INDEX assets__non_fungible_idx_tmp
    ON assets__non_fungible_token_events (emitted_for_receipt_id,
                                          emitted_at_block_timestamp,
                                          emitted_in_shard_id,
                                          emitted_index_of_event_entry_in_shard,
                                          emitted_by_contract_account_id,
                                          token_id,
                                          event_kind,
                                          token_old_owner_account_id,
                                          token_new_owner_account_id,
                                          token_authorized_account_id,
                                          event_memo);
CREATE UNIQUE INDEX assets__non_fungible_token_events_unique
    ON assets__non_fungible_token_events (emitted_for_receipt_id, emitted_index_of_event_entry_in_shard);

-- Next block runs ~1 sec even on the full DB
-- If you apply this manually, uncomment BEGIN TRANSACTION and COMMIT

-- BEGIN TRANSACTION;
SAVEPOINT change_nft_pks_back;
ALTER TABLE assets__non_fungible_token_events
    DROP CONSTRAINT assets__non_fungible_token_events_pkey;
-- This command will automatically rename assets__non_fungible_idx_tmp to assets__non_fungible_token_events_pkey
ALTER TABLE assets__non_fungible_token_events
    ADD CONSTRAINT assets__non_fungible_token_events_pkey PRIMARY KEY USING INDEX assets__non_fungible_idx_tmp;
RELEASE SAVEPOINT change_nft_pks_back;
-- COMMIT;
