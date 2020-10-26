ALTER TABLE access_keys
    ADD CONSTRAINT account_fk FOREIGN KEY (account_id) REFERENCES accounts(account_id) ON DELETE CASCADE;
