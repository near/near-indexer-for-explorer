CREATE TYPE nft_event_kind AS ENUM (
    'MINT',
    'TRANSFER',
    'BURN'
    );

CREATE TABLE assets__non_fungible_token_events
(
    emitted_for_receipt_id                text           NOT NULL,

    -- Next three columns (emitted_at_block_timestamp, emitted_in_shard_id, emitted_index_of_event_entry_in_shard)
    -- should be used for sorting purposes, at the order that we just named.
    emitted_at_block_timestamp            numeric(20, 0) NOT NULL,
    emitted_in_shard_id                   numeric(20, 0) NOT NULL,
    -- `emitted_index_of_event_entry_in_shard` has non-trivial implementation. It combines the order from:
    -- 1. execution_outcomes::index_in_chunk
    -- 2. Index of current action_receipt
    -- 3. Index of event entry that we are currently working on. Note, one receipt can have multiple events
    --    (read: log with multiple statements), each of them can have multiple account_ids and token_ids.
    --    We use continuous numbering for all these items.
    emitted_index_of_event_entry_in_shard integer        NOT NULL,

    -- account_id of the contract itself. In a simple words, it's the owner/creator of NFT contract
    emitted_by_contract_account_id        text           NOT NULL,
    -- Unique ID of the token
    token_id                              text           NOT NULL,
    event_kind                            nft_event_kind NOT NULL,

    -- We use `NOT NULL DEFAULT ''` in all the lines below to simplify further issue with nulls + constraints
    -- Previous owner of the token. Empty if we have nft_event_kind 'MINT'.
    token_old_owner_account_id            text           NOT NULL DEFAULT '',
    -- New owner of the token. Empty if we have nft_event_kind 'BURN'.
    token_new_owner_account_id            text           NOT NULL DEFAULT '',
    -- The account that initialized the event.
    -- It differs from token_old_owner_account_id, but it is approved to manipulate with current token.
    -- More information here https://nomicon.io/Standards/NonFungibleToken/ApprovalManagement.html
    -- Optional field: filled only if the event is done NOT by token_old_owner_account_id.
    -- Empty if we have nft_event_kind 'MINT'.
    token_authorized_account_id           text           NOT NULL DEFAULT '',
    -- Optional message associated with token movement.
    event_memo                            text           NOT NULL DEFAULT ''
);

-- We have to add everything to PK because of some reasons:
-- 1. We need to ignore the same lines, they could come from different indexers, that is fully legal context.
-- 2. We need to catch the situation when we passed PK constraint, but failed UNIQUE constraint below.
ALTER TABLE ONLY assets__non_fungible_token_events
    ADD CONSTRAINT assets__non_fungible_token_events_pkey PRIMARY KEY (emitted_for_receipt_id,
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

-- This set of columns is enough to identify the record
-- We use UNIQUE constraint here to catch the errors if the incoming data looks inconsistent
ALTER TABLE ONLY assets__non_fungible_token_events
    ADD CONSTRAINT assets__non_fungible_token_events_unique UNIQUE (emitted_for_receipt_id,
                                                                    emitted_index_of_event_entry_in_shard);

-- To sum up, let's consider all possible situations:
-- PK passed, UNIQUE passed: everything is OK, let's insert the line
-- PK passed, UNIQUE failed: we have UNIQUE constraint error, let's log it somewhere, there should be a bug somewhere.
-- PK failed, UNIQUE passed: unreachable
-- PK failed, UNIQUE failed: we have PK constraint error
--                           (we have both, but PK constraint is more severe and DB will complain only about it).
--                           It's the correct line from other indexer, simply ignore it

CREATE INDEX assets__non_fungible_token_events_sorting_idx ON assets__non_fungible_token_events
    USING btree (emitted_at_block_timestamp,
                 emitted_in_shard_id,
                 emitted_index_of_event_entry_in_shard);

CREATE INDEX assets__non_fungible_token_events_block_timestamp_idx ON assets__non_fungible_token_events
    USING btree (emitted_at_block_timestamp);

CREATE INDEX assets__non_fungible_token_events_old_owner_account_id_idx ON assets__non_fungible_token_events
    USING btree (token_old_owner_account_id);

CREATE INDEX assets__non_fungible_token_events_new_owner_account_id_idx ON assets__non_fungible_token_events
    USING btree (token_new_owner_account_id);

ALTER TABLE ONLY assets__non_fungible_token_events
    ADD CONSTRAINT assets__non_fungible_token_events_fk
        FOREIGN KEY (emitted_for_receipt_id) REFERENCES receipts (receipt_id) ON DELETE CASCADE;
