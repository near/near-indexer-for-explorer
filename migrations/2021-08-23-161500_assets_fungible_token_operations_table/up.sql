CREATE TABLE assets__fungible_token_operations
(
    processed_in_receipt_id      text           NOT NULL,
    processed_in_block_timestamp numeric(20, 0) NOT NULL,
    called_method                text           NOT NULL,
    ft_contract_account_id       text           NOT NULL,
    ft_sender_account_id         text           NOT NULL,
    ft_receiver_account_id       text           NOT NULL,
    ft_amount                    numeric(45, 0) NOT NULL,
    args                         jsonb          NOT NULL
);

ALTER TABLE ONLY assets__fungible_token_operations
    ADD CONSTRAINT assets__fungible_token_operations_pkey PRIMARY KEY (processed_in_receipt_id);

CREATE INDEX assets__fungible_token_operations_timestamp_idx ON assets__fungible_token_operations USING btree (processed_in_block_timestamp);

ALTER TABLE ONLY assets__fungible_token_operations
    ADD CONSTRAINT assets__fungible_token_operations_processed_in_receipt_id_fk FOREIGN KEY (processed_in_receipt_id) REFERENCES receipts (receipt_id) ON DELETE CASCADE;

ALTER TABLE ONLY assets__fungible_token_operations
    ADD CONSTRAINT assets__fungible_token_operations_ft_contract_account_id_fk FOREIGN KEY (ft_contract_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

ALTER TABLE ONLY assets__fungible_token_operations
    ADD CONSTRAINT assets__fungible_token_operations_ft_sender_account_id_fk FOREIGN KEY (ft_sender_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;

ALTER TABLE ONLY assets__fungible_token_operations
    ADD CONSTRAINT assets__fungible_token_operations_ft_receiver_account_id_fk FOREIGN KEY (ft_receiver_account_id) REFERENCES accounts (account_id) ON DELETE CASCADE;
