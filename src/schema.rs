table! {
    use diesel::sql_types::*;

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
    use diesel::sql_types::*;

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
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_action_actions (id) {
        id -> Int8,
        receipt_id -> Text,
        index -> Int4,
        #[sql_name = "type"]
        type_ -> Action_type,
        args -> Nullable<Jsonb>,
    }
}

table! {
    use diesel::sql_types::*;

    receipt_action_input_data (id) {
        id -> Int8,
        receipt_id -> Text,
        data_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;

    receipt_action_output_data (id) {
        id -> Int8,
        receipt_id -> Text,
        data_id -> Varchar,
        receiver_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;

    receipt_actions (id) {
        id -> Int8,
        receipt_id -> Text,
        signer_id -> Text,
        signer_public_key -> Text,
        gas_price -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;

    receipt_data (id) {
        id -> Int8,
        receipt_id -> Text,
        data_id -> Text,
        data -> Nullable<Bytea>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipts (receipt_id) {
        receipt_id -> Text,
        block_height -> Nullable<Numeric>,
        predecessor_id -> Text,
        receiver_id -> Text,
        #[sql_name = "type"]
        type_ -> Receipt_type,
    }
}
