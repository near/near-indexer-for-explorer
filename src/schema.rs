table! {
    access_keys (account_id) {
        account_id -> Text,
        public_key -> Text,
        access_key_type -> Text,
    }
}

table! {
    accounts (account_id) {
        account_id -> Text,
        index -> Int4,
        created_by_receipt_id -> Varchar,
        created_at_timestamp -> Nullable<Numeric>,
    }
}

table! {
    actions (id) {
        id -> Int8,
        receipt_id -> Varchar,
        index -> Int4,
        #[sql_name = "type"]
        type_ -> Varchar,
        args -> Nullable<Json>,
    }
}

table! {
    actions_input_data (id) {
        id -> Numeric,
        receipt_id -> Varchar,
        data_id -> Varchar,
    }
}

table! {
    actions_output_data (id) {
        id -> Numeric,
        receipt_id -> Varchar,
        data_id -> Varchar,
        receiver_id -> Text,
    }
}

table! {
    blocks (height) {
        height -> Numeric,
        hash -> Varchar,
        prev_hash -> Varchar,
        timestamp -> Numeric,
        total_supply -> Numeric,
        gas_limit -> Numeric,
        gas_used -> Numeric,
        gas_price -> Numeric,
    }
}

table! {
    chunks (hash) {
        block_id -> Numeric,
        hash -> Varchar,
        shard_id -> Numeric,
        signature -> Text,
        gas_limit -> Numeric,
        gas_used -> Numeric,
        height_created -> Numeric,
        height_included -> Numeric,
    }
}

table! {
    receipt_action (id) {
        id -> Int8,
        receipt_id -> Varchar,
        signer_id -> Varchar,
        signer_public_key -> Text,
        gas_price -> Nullable<Numeric>,
    }
}

table! {
    receipt_data (id) {
        id -> Int8,
        receipt_id -> Varchar,
        data_id -> Varchar,
        data -> Nullable<Text>,
    }
}

table! {
    receipts (receipt_id) {
        receipt_id -> Varchar,
        predecessor_id -> Nullable<Text>,
        receiver_id -> Nullable<Text>,
        status -> Nullable<Varchar>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
    }
}

table! {
    transactions (hash) {
        hash -> Varchar,
        block_id -> Numeric,
        block_timestamp -> Numeric,
        nonce -> Numeric,
        signer_id -> Text,
        signer_public_key -> Text,
        signature -> Text,
        receiver_id -> Text,
        receipt_conversion_gas_burnt -> Nullable<Numeric>,
        receipt_conversion_tokens_burnt -> Nullable<Numeric>,
        receipt_id -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    access_keys,
    accounts,
    actions,
    actions_input_data,
    actions_output_data,
    blocks,
    chunks,
    receipt_action,
    receipt_data,
    receipts,
    transactions,
);
