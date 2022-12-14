--
-- PostgreSQL database dump
--
--
-- Name: access_key_permission_kind; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.access_key_permission_kind AS ENUM (
    'FULL_ACCESS',
    'FUNCTION_CALL'
);


--
-- Name: action_kind; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.action_kind AS ENUM (
    'CREATE_ACCOUNT',
    'DEPLOY_CONTRACT',
    'FUNCTION_CALL',
    'TRANSFER',
    'STAKE',
    'ADD_KEY',
    'DELETE_KEY',
    'DELETE_ACCOUNT'
);


--
-- Name: execution_outcome_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.execution_outcome_status AS ENUM (
    'UNKNOWN',
    'FAILURE',
    'SUCCESS_VALUE',
    'SUCCESS_RECEIPT_ID'
);


--
-- Name: receipt_kind; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.receipt_kind AS ENUM (
    'ACTION',
    'DATA'
);


--
-- Name: state_change_reason_kind; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.state_change_reason_kind AS ENUM (
    'TRANSACTION_PROCESSING',
    'ACTION_RECEIPT_PROCESSING_STARTED',
    'ACTION_RECEIPT_GAS_REWARD',
    'RECEIPT_PROCESSING',
    'POSTPONED_RECEIPT',
    'UPDATED_DELAYED_RECEIPTS',
    'VALIDATOR_ACCOUNTS_UPDATE'
);

--
-- Name: access_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.access_keys (
    public_key text NOT NULL,
    account_id text NOT NULL,
    created_by_receipt_id text,
    deleted_by_receipt_id text,
    permission_kind public.access_key_permission_kind NOT NULL,
    last_update_block_height numeric(20,0) NOT NULL
);


--
-- Name: account_changes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.account_changes (
    id bigserial NOT NULL,
    affected_account_id text NOT NULL,
    changed_in_block_timestamp numeric(20,0) NOT NULL,
    changed_in_block_hash text NOT NULL,
    caused_by_transaction_hash text,
    caused_by_receipt_id text,
    update_reason public.state_change_reason_kind NOT NULL,
    affected_account_nonstaked_balance numeric(45,0) NOT NULL,
    affected_account_staked_balance numeric(45,0) NOT NULL,
    affected_account_storage_usage numeric(20,0) NOT NULL
);

--
-- Name: accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.accounts (
    id bigserial NOT NULL,
    account_id text NOT NULL,
    created_by_receipt_id text,
    deleted_by_receipt_id text,
    last_update_block_height numeric(20,0) NOT NULL
);

--
-- Name: action_receipt_actions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.action_receipt_actions (
    receipt_id text NOT NULL,
    index_in_action_receipt integer NOT NULL,
    action_kind public.action_kind NOT NULL,
    args jsonb NOT NULL
);


--
-- Name: action_receipt_input_data; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.action_receipt_input_data (
    input_data_id text NOT NULL,
    input_to_receipt_id text NOT NULL
);


--
-- Name: action_receipt_output_data; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.action_receipt_output_data (
    output_data_id text NOT NULL,
    output_from_receipt_id text NOT NULL,
    receiver_account_id text NOT NULL
);


--
-- Name: action_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.action_receipts (
    receipt_id text NOT NULL,
    signer_account_id text NOT NULL,
    signer_public_key text NOT NULL,
    gas_price numeric(45,0) NOT NULL
);


--
-- Name: blocks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.blocks (
    block_height numeric(20,0) NOT NULL,
    block_hash text NOT NULL,
    prev_block_hash text NOT NULL,
    block_timestamp numeric(20,0) NOT NULL,
    total_supply numeric(45,0) NOT NULL,
    gas_price numeric(45,0) NOT NULL,
    author_account_id text NOT NULL
);


--
-- Name: chunks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.chunks (
    included_in_block_hash text NOT NULL,
    chunk_hash text NOT NULL,
    shard_id numeric(20,0) NOT NULL,
    signature text NOT NULL,
    gas_limit numeric(20,0) NOT NULL,
    gas_used numeric(20,0) NOT NULL,
    author_account_id text NOT NULL
);


--
-- Name: data_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.data_receipts (
    data_id text NOT NULL,
    receipt_id text NOT NULL,
    data bytea
);


--
-- Name: execution_outcome_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.execution_outcome_receipts (
    executed_receipt_id text NOT NULL,
    index_in_execution_outcome integer NOT NULL,
    produced_receipt_id text NOT NULL
);


--
-- Name: execution_outcomes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.execution_outcomes (
    receipt_id text NOT NULL,
    executed_in_block_hash text NOT NULL,
    executed_in_block_timestamp numeric(20,0) NOT NULL,
    executed_in_chunk_hash text NOT NULL,
    index_in_chunk integer NOT NULL,
    gas_burnt numeric(20,0) NOT NULL,
    tokens_burnt numeric(45,0) NOT NULL,
    executor_account_id text NOT NULL,
    status public.execution_outcome_status NOT NULL
);


--
-- Name: receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.receipts (
    receipt_id text NOT NULL,
    included_in_block_hash text NOT NULL,
    included_in_chunk_hash text NOT NULL,
    index_in_chunk integer NOT NULL,
    included_in_block_timestamp numeric(20,0) NOT NULL,
    predecessor_account_id text NOT NULL,
    receiver_account_id text NOT NULL,
    receipt_kind public.receipt_kind NOT NULL,
    originated_from_transaction_hash text NOT NULL
);


--
-- Name: transaction_actions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.transaction_actions (
    transaction_hash text NOT NULL,
    index_in_transaction integer NOT NULL,
    action_kind public.action_kind NOT NULL,
    args jsonb NOT NULL
);


--
-- Name: transactions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.transactions (
    transaction_hash text NOT NULL,
    included_in_block_hash text NOT NULL,
    included_in_chunk_hash text NOT NULL,
    index_in_chunk integer NOT NULL,
    block_timestamp numeric(20,0) NOT NULL,
    signer_account_id text NOT NULL,
    signer_public_key text NOT NULL,
    nonce numeric(20,0) NOT NULL,
    receiver_account_id text NOT NULL,
    signature text NOT NULL,
    status public.execution_outcome_status NOT NULL,
    converted_into_receipt_id text NOT NULL,
    receipt_conversion_gas_burnt numeric(20,0),
    receipt_conversion_tokens_burnt numeric(45,0)
);

--
-- Name: access_keys access_keys_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_keys
    ADD CONSTRAINT access_keys_pk PRIMARY KEY (public_key, account_id);


--
-- Name: account_changes account_changes_affected_account_id_changed_in_block_hash_c_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.account_changes
    ADD CONSTRAINT account_changes_affected_account_id_changed_in_block_hash_c_key UNIQUE (affected_account_id, changed_in_block_hash, caused_by_transaction_hash, caused_by_receipt_id);


--
-- Name: account_changes account_changes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.account_changes
    ADD CONSTRAINT account_changes_pkey PRIMARY KEY (id);


--
-- Name: accounts accounts_account_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_account_id_key UNIQUE (account_id);


--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_pkey PRIMARY KEY (id);


--
-- Name: action_receipt_input_data action_input_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipt_input_data
    ADD CONSTRAINT action_input_pk PRIMARY KEY (input_data_id, input_to_receipt_id);


--
-- Name: action_receipt_output_data action_output_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipt_output_data
    ADD CONSTRAINT action_output_pk PRIMARY KEY (output_data_id, output_from_receipt_id);


--
-- Name: blocks blocks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_pkey PRIMARY KEY (block_hash);


--
-- Name: chunks chunks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chunks
    ADD CONSTRAINT chunks_pkey PRIMARY KEY (chunk_hash);


--
-- Name: execution_outcome_receipts execution_outcome_receipt_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.execution_outcome_receipts
    ADD CONSTRAINT execution_outcome_receipt_pk PRIMARY KEY (executed_receipt_id, index_in_execution_outcome, produced_receipt_id);


--
-- Name: execution_outcomes execution_outcomes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.execution_outcomes
    ADD CONSTRAINT execution_outcomes_pkey PRIMARY KEY (receipt_id);


--
-- Name: action_receipt_actions receipt_action_action_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipt_actions
    ADD CONSTRAINT receipt_action_action_pk PRIMARY KEY (receipt_id, index_in_action_receipt);


--
-- Name: action_receipts receipt_actions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipts
    ADD CONSTRAINT receipt_actions_pkey PRIMARY KEY (receipt_id);


--
-- Name: data_receipts receipt_data_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.data_receipts
    ADD CONSTRAINT receipt_data_pkey PRIMARY KEY (data_id);


--
-- Name: receipts receipts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.receipts
    ADD CONSTRAINT receipts_pkey PRIMARY KEY (receipt_id);


--
-- Name: transaction_actions transaction_action_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transaction_actions
    ADD CONSTRAINT transaction_action_pk PRIMARY KEY (transaction_hash, index_in_transaction);


--
-- Name: transactions transactions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT transactions_pkey PRIMARY KEY (transaction_hash);


--
-- Name: access_keys_account_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX access_keys_account_id_idx ON public.access_keys USING btree (account_id);


--
-- Name: access_keys_last_update_block_height_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX access_keys_last_update_block_height_idx ON public.access_keys USING btree (last_update_block_height);


--
-- Name: access_keys_public_key_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX access_keys_public_key_idx ON public.access_keys USING btree (public_key);


--
-- Name: account_changes_affected_account_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX account_changes_affected_account_id_idx ON public.account_changes USING btree (affected_account_id);


--
-- Name: account_changes_changed_in_block_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX account_changes_changed_in_block_hash_idx ON public.account_changes USING btree (changed_in_block_hash);


--
-- Name: account_changes_changed_in_block_timestamp_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX account_changes_changed_in_block_timestamp_idx ON public.account_changes USING btree (changed_in_block_timestamp);


--
-- Name: account_changes_changed_in_caused_by_receipt_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX account_changes_changed_in_caused_by_receipt_id_idx ON public.account_changes USING btree (caused_by_receipt_id);


--
-- Name: account_changes_changed_in_caused_by_transaction_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX account_changes_changed_in_caused_by_transaction_hash_idx ON public.account_changes USING btree (caused_by_transaction_hash);


--
-- Name: accounts_last_update_block_height_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX accounts_last_update_block_height_idx ON public.accounts USING btree (last_update_block_height);


--
-- Name: action_receipt_input_data_input_data_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX action_receipt_input_data_input_data_id_idx ON public.action_receipt_input_data USING btree (input_data_id);


--
-- Name: action_receipt_input_data_input_to_receipt_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX action_receipt_input_data_input_to_receipt_id_idx ON public.action_receipt_input_data USING btree (input_to_receipt_id);


--
-- Name: action_receipt_output_data_output_data_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX action_receipt_output_data_output_data_id_idx ON public.action_receipt_output_data USING btree (output_data_id);


--
-- Name: action_receipt_output_data_output_from_receipt_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX action_receipt_output_data_output_from_receipt_id_idx ON public.action_receipt_output_data USING btree (output_from_receipt_id);


--
-- Name: action_receipt_output_data_receiver_account_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX action_receipt_output_data_receiver_account_id_idx ON public.action_receipt_output_data USING btree (receiver_account_id);


--
-- Name: action_receipt_signer_account_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX action_receipt_signer_account_id_idx ON public.action_receipts USING btree (signer_account_id);


--
-- Name: blocks_height_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX blocks_height_idx ON public.blocks USING btree (block_height);


--
-- Name: blocks_prev_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX blocks_prev_hash_idx ON public.blocks USING btree (prev_block_hash);


--
-- Name: blocks_timestamp_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX blocks_timestamp_idx ON public.blocks USING btree (block_timestamp);


--
-- Name: chunks_included_in_block_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX chunks_included_in_block_hash_idx ON public.chunks USING btree (included_in_block_hash);


--
-- Name: data_receipts_receipt_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX data_receipts_receipt_id_idx ON public.data_receipts USING btree (receipt_id);


--
-- Name: execution_outcome_executed_in_block_timestamp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX execution_outcome_executed_in_block_timestamp ON public.execution_outcomes USING btree (executed_in_block_timestamp);


--
-- Name: execution_outcome_executed_in_chunk_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX execution_outcome_executed_in_chunk_hash_idx ON public.execution_outcomes USING btree (executed_in_chunk_hash);


--
-- Name: execution_outcome_receipts_produced_receipt_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX execution_outcome_receipts_produced_receipt_id ON public.execution_outcome_receipts USING btree (produced_receipt_id);


--
-- Name: execution_outcomes_block_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX execution_outcomes_block_hash_idx ON public.execution_outcomes USING btree (executed_in_block_hash);


--
-- Name: execution_outcomes_receipt_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX execution_outcomes_receipt_id_idx ON public.execution_outcomes USING btree (receipt_id);


--
-- Name: receipts_included_in_block_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX receipts_included_in_block_hash_idx ON public.receipts USING btree (included_in_block_hash);


--
-- Name: receipts_included_in_chunk_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX receipts_included_in_chunk_hash_idx ON public.receipts USING btree (included_in_chunk_hash);


--
-- Name: receipts_predecessor_account_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX receipts_predecessor_account_id_idx ON public.receipts USING btree (predecessor_account_id);


--
-- Name: receipts_receiver_account_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX receipts_receiver_account_id_idx ON public.receipts USING btree (receiver_account_id);


--
-- Name: receipts_timestamp_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX receipts_timestamp_idx ON public.receipts USING btree (included_in_block_timestamp);


--
-- Name: transactions_converted_into_receipt_id_dx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX transactions_converted_into_receipt_id_dx ON public.transactions USING btree (converted_into_receipt_id);


--
-- Name: transactions_included_in_block_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX transactions_included_in_block_hash_idx ON public.transactions USING btree (included_in_block_hash);


--
-- Name: transactions_included_in_block_timestamp_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX transactions_included_in_block_timestamp_idx ON public.transactions USING btree (block_timestamp);


--
-- Name: transactions_included_in_chunk_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX transactions_included_in_chunk_hash_idx ON public.transactions USING btree (included_in_chunk_hash);


--
-- Name: transactions_signer_account_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX transactions_signer_account_id_idx ON public.transactions USING btree (signer_account_id);


--
-- Name: transactions_signer_public_key_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX transactions_signer_public_key_idx ON public.transactions USING btree (signer_public_key);


--
-- Name: account_changes account_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.account_changes
    ADD CONSTRAINT account_id_fk FOREIGN KEY (affected_account_id) REFERENCES public.accounts(account_id) ON DELETE CASCADE;


--
-- Name: action_receipt_actions action_receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipt_actions
    ADD CONSTRAINT action_receipt_fk FOREIGN KEY (receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: execution_outcomes block_hash_execution_outcome_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.execution_outcomes
    ADD CONSTRAINT block_hash_execution_outcome_fk FOREIGN KEY (executed_in_block_hash) REFERENCES public.blocks(block_hash) ON DELETE CASCADE;


--
-- Name: account_changes block_hash_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.account_changes
    ADD CONSTRAINT block_hash_fk FOREIGN KEY (changed_in_block_hash) REFERENCES public.blocks(block_hash) ON DELETE CASCADE;


--
-- Name: receipts block_receipts_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.receipts
    ADD CONSTRAINT block_receipts_fk FOREIGN KEY (included_in_block_hash) REFERENCES public.blocks(block_hash) ON DELETE CASCADE;


--
-- Name: transactions block_tx_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT block_tx_fk FOREIGN KEY (included_in_block_hash) REFERENCES public.blocks(block_hash) ON DELETE CASCADE;


--
-- Name: execution_outcomes chunk_hash_outcome_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.execution_outcomes
    ADD CONSTRAINT chunk_hash_outcome_fk FOREIGN KEY (executed_in_chunk_hash) REFERENCES public.chunks(chunk_hash) ON DELETE CASCADE;


--
-- Name: receipts chunk_receipts_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.receipts
    ADD CONSTRAINT chunk_receipts_fk FOREIGN KEY (included_in_chunk_hash) REFERENCES public.chunks(chunk_hash) ON DELETE CASCADE;


--
-- Name: transactions chunk_tx_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transactions
    ADD CONSTRAINT chunk_tx_fk FOREIGN KEY (included_in_chunk_hash) REFERENCES public.chunks(chunk_hash) ON DELETE CASCADE;


--
-- Name: chunks chunks_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chunks
    ADD CONSTRAINT chunks_fk FOREIGN KEY (included_in_block_hash) REFERENCES public.blocks(block_hash) ON DELETE CASCADE;


--
-- Name: access_keys created_by_receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_keys
    ADD CONSTRAINT created_by_receipt_fk FOREIGN KEY (created_by_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: accounts created_receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT created_receipt_fk FOREIGN KEY (created_by_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: access_keys deleted_by_receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.access_keys
    ADD CONSTRAINT deleted_by_receipt_fk FOREIGN KEY (deleted_by_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: accounts deleted_receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT deleted_receipt_fk FOREIGN KEY (deleted_by_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: execution_outcome_receipts execution_outcome_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.execution_outcome_receipts
    ADD CONSTRAINT execution_outcome_fk FOREIGN KEY (executed_receipt_id) REFERENCES public.execution_outcomes(receipt_id) ON DELETE CASCADE;


--
-- Name: execution_outcomes receipt_execution_outcome_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.execution_outcomes
    ADD CONSTRAINT receipt_execution_outcome_fk FOREIGN KEY (receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: data_receipts receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.data_receipts
    ADD CONSTRAINT receipt_fk FOREIGN KEY (receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: action_receipts receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipts
    ADD CONSTRAINT receipt_fk FOREIGN KEY (receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: action_receipt_output_data receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipt_output_data
    ADD CONSTRAINT receipt_fk FOREIGN KEY (output_from_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: action_receipt_input_data receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.action_receipt_input_data
    ADD CONSTRAINT receipt_fk FOREIGN KEY (input_to_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: account_changes receipt_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.account_changes
    ADD CONSTRAINT receipt_id_fk FOREIGN KEY (caused_by_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: execution_outcome_receipts receipts_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.execution_outcome_receipts
    ADD CONSTRAINT receipts_fk FOREIGN KEY (executed_receipt_id) REFERENCES public.receipts(receipt_id) ON DELETE CASCADE;


--
-- Name: account_changes transaction_hash_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.account_changes
    ADD CONSTRAINT transaction_hash_fk FOREIGN KEY (caused_by_transaction_hash) REFERENCES public.transactions(transaction_hash) ON DELETE CASCADE;


--
-- Name: transaction_actions tx_action_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transaction_actions
    ADD CONSTRAINT tx_action_fk FOREIGN KEY (transaction_hash) REFERENCES public.transactions(transaction_hash) ON DELETE CASCADE;


--
-- Name: receipts tx_receipt_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.receipts
    ADD CONSTRAINT tx_receipt_fk FOREIGN KEY (originated_from_transaction_hash) REFERENCES public.transactions(transaction_hash) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

