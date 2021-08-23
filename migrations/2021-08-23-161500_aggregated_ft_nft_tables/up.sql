CREATE TABLE aggregated__fungible_tokens
(
    id                                bigserial                NOT NULL,
    included_in_transaction_hash      text                     NOT NULL,
    included_in_transaction_timestamp numeric(20, 0)           NOT NULL,
    transaction_status                execution_outcome_status NOT NULL,
    issued_contract_id                text                     NOT NULL,
    called_method                     text                     NOT NULL,
    predecessor_account_id            text                     NOT NULL,
    receiver_account_id               text                     NOT NULL,
    amount                            numeric(45, 0)           NOT NULL,
    args                              jsonb                    NOT NULL
);

ALTER TABLE ONLY aggregated__fungible_tokens
    ADD CONSTRAINT aggregated__fungible_tokens_pkey PRIMARY KEY (id);

CREATE INDEX aggregated__fungible_tokens_timestamp_idx ON aggregated__fungible_tokens USING btree (included_in_transaction_timestamp);

ALTER TABLE ONLY aggregated__fungible_tokens
    ADD CONSTRAINT aggregated__fungible_tokens_fk FOREIGN KEY (included_in_transaction_hash) REFERENCES transactions (transaction_hash) ON DELETE CASCADE;

ALTER TABLE ONLY aggregated__fungible_tokens
    ADD CONSTRAINT aggregated__ft_issued_contract_id_fk FOREIGN KEY (issued_contract_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

ALTER TABLE ONLY aggregated__fungible_tokens
    ADD CONSTRAINT aggregated__ft_predecessor_account_id_fk FOREIGN KEY (predecessor_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

ALTER TABLE ONLY aggregated__fungible_tokens
    ADD CONSTRAINT aggregated__ft_receiver_account_id_fk FOREIGN KEY (receiver_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

CREATE TABLE aggregated__non_fungible_tokens
(
    id                                bigserial                NOT NULL,
    included_in_transaction_hash      text                     NOT NULL,
    included_in_transaction_timestamp numeric(20, 0)           NOT NULL,
    transaction_status                execution_outcome_status NOT NULL,
    issued_contract_id                text                     NOT NULL,
    called_method                     text                     NOT NULL,
    non_fungible_token_id             text                     NOT NULL,
    predecessor_account_id            text                     NOT NULL,
    receiver_account_id               text                     NOT NULL,
    amount                            numeric(45, 0)           NOT NULL,
    args                              jsonb                    NOT NULL
);

ALTER TABLE ONLY aggregated__non_fungible_tokens
    ADD CONSTRAINT aggregated__non_fungible_tokens_pkey PRIMARY KEY (id);

CREATE INDEX aggregated__non_fungible_tokens_timestamp_idx ON aggregated__non_fungible_tokens USING btree (included_in_transaction_timestamp);

ALTER TABLE ONLY aggregated__non_fungible_tokens
    ADD CONSTRAINT aggregated__non_fungible_tokens_fk FOREIGN KEY (included_in_transaction_hash) REFERENCES transactions (transaction_hash) ON DELETE CASCADE;

ALTER TABLE ONLY aggregated__non_fungible_tokens
    ADD CONSTRAINT aggregated__nft_issued_contract_id_fk FOREIGN KEY (issued_contract_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

ALTER TABLE ONLY aggregated__non_fungible_tokens
    ADD CONSTRAINT aggregated__nft_predecessor_account_id_fk FOREIGN KEY (predecessor_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

ALTER TABLE ONLY aggregated__non_fungible_tokens
    ADD CONSTRAINT aggregated__nft_receiver_account_id_fk FOREIGN KEY (receiver_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;
