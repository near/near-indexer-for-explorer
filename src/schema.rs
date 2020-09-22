table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    blocks (height) {
        height -> Numeric,
        hash -> Bytea,
        prev_hash -> Bytea,
        timestamp -> Numeric,
        total_supply -> Numeric,
        gas_price -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    chunks (hash) {
        block_id -> Numeric,
        hash -> Bytea,
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
        receipt_id -> Bytea,
        index -> Int4,
        action_kind -> Action_type,
        args -> Jsonb,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_action_input_data (data_id) {
        data_id -> Bytea,
        receipt_id -> Bytea,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_action_output_data (data_id) {
        data_id -> Bytea,
        receipt_id -> Bytea,
        receiver_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_actions (receipt_id) {
        receipt_id -> Bytea,
        signer_id -> Text,
        signer_public_key -> Text,
        gas_price -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_data (data_id) {
        data_id -> Bytea,
        receipt_id -> Bytea,
        data -> Nullable<Bytea>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipts (receipt_id) {
        receipt_id -> Bytea,
        block_height -> Nullable<Numeric>,
        predecessor_id -> Text,
        receiver_id -> Text,
        receipt_kind -> Receipt_type,
    }
}

joinable!(chunks -> blocks (block_id));

allow_tables_to_appear_in_same_query!(
    blocks,
    chunks,
    receipt_action_actions,
    receipt_action_input_data,
    receipt_action_output_data,
    receipt_actions,
    receipt_data,
    receipts,
);
