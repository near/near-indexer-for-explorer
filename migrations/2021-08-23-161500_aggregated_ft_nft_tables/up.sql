CREATE TABLE aggregated__fungible_token_operations
(
    processed_in_block_timestamp  numeric(20, 0) NOT NULL,
    processed_in_transaction_hash text           NOT NULL,
    processing_index_in_chunk     integer        NOT NULL,
    ft_contract_account_id        text           NOT NULL,
    ft_affected_account_id        text           NOT NULL,
    called_method                 text           NOT NULL,
    ft_affected_account_balance   numeric(45, 0) NOT NULL,
    args                          jsonb          NOT NULL
);

ALTER TABLE ONLY aggregated__fungible_token_operations
    ADD CONSTRAINT aggregated__fungible_token_operations_pkey PRIMARY KEY (processed_in_transaction_hash,
                                                                           ft_contract_account_id,
                                                                           ft_affected_account_id);

-- I will edit it after we finalise the naming
-- CREATE INDEX aggregated__fungible_tokens_timestamp_idx ON aggregated__fungible_tokens USING btree (included_in_transaction_timestamp);
--
-- ALTER TABLE ONLY aggregated__fungible_tokens
--     ADD CONSTRAINT aggregated__fungible_tokens_fk FOREIGN KEY (included_in_transaction_hash) REFERENCES transactions (transaction_hash) ON DELETE CASCADE;
--
-- ALTER TABLE ONLY aggregated__fungible_tokens
--     ADD CONSTRAINT aggregated__ft_issued_contract_id_fk FOREIGN KEY (issued_contract_id) REFERENCES accounts (account_id) ON DELETE CASCADE;
--
-- ALTER TABLE ONLY aggregated__fungible_tokens
--     ADD CONSTRAINT aggregated__ft_predecessor_account_id_fk FOREIGN KEY (predecessor_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;
--
-- ALTER TABLE ONLY aggregated__fungible_tokens
--     ADD CONSTRAINT aggregated__ft_receiver_account_id_fk FOREIGN KEY (receiver_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

CREATE TABLE aggregated__non_fungible_token_operations
(
    processed_in_block_timestamp  numeric(20, 0) NOT NULL,
    processed_in_transaction_hash text           NOT NULL,
    processing_index_in_chunk     integer        NOT NULL,
    nft_contract_account_id       text           NOT NULL,
    nft_affected_account_id       text           NOT NULL,
    called_method                 text           NOT NULL,
    nft_id                        text           NOT NULL,
    args                          jsonb          NOT NULL
);

ALTER TABLE ONLY aggregated__non_fungible_token_operations
    ADD CONSTRAINT aggregated__non_fungible_token_operations_pkey PRIMARY KEY (processed_in_transaction_hash,
                                                                               nft_contract_account_id,
                                                                               nft_affected_account_id,
                                                                               nft_id);

-- I will edit it after we finalise the naming
-- CREATE INDEX aggregated__non_fungible_tokens_timestamp_idx ON aggregated__non_fungible_tokens USING btree (included_in_transaction_timestamp);
--
-- ALTER TABLE ONLY aggregated__non_fungible_tokens
--     ADD CONSTRAINT aggregated__non_fungible_tokens_fk FOREIGN KEY (included_in_transaction_hash) REFERENCES transactions (transaction_hash) ON DELETE CASCADE;
--
-- ALTER TABLE ONLY aggregated__non_fungible_tokens
--     ADD CONSTRAINT aggregated__nft_issued_contract_id_fk FOREIGN KEY (issued_contract_id) REFERENCES accounts (account_id) ON DELETE CASCADE;
--
-- ALTER TABLE ONLY aggregated__non_fungible_tokens
--     ADD CONSTRAINT aggregated__nft_predecessor_account_id_fk FOREIGN KEY (predecessor_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;
--
-- ALTER TABLE ONLY aggregated__non_fungible_tokens
--     ADD CONSTRAINT aggregated__nft_receiver_account_id_fk FOREIGN KEY (receiver_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;
