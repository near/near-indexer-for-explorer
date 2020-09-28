table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    blocks (height) {
        height -> Numeric,
        hash -> Text,
        prev_hash -> Text,
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
        hash -> Text,
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

    execution_outcome_receipts (execution_outcome_receipt_id, index, receipt_id) {
        execution_outcome_receipt_id -> Text,
        index -> Int4,
        receipt_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    execution_outcomes (receipt_id) {
        receipt_id -> Text,
        block_hash -> Text,
        gas_burnt -> Numeric,
        tokens_burnt -> Numeric,
        executor_id -> Text,
        status -> Execution_outcome_status,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_action_actions (id) {
        id -> Int8,
        receipt_id -> Text,
        index -> Int4,
        action_kind -> Action_type,
        args -> Jsonb,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_action_input_data (id) {
        id -> Int8,
        data_id -> Text,
        receipt_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_action_output_data (id) {
        id -> Int8,
        data_id -> Text,
        receipt_id -> Text,
        receiver_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_actions (receipt_id) {
        receipt_id -> Text,
        signer_id -> Text,
        signer_public_key -> Text,
        gas_price -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipt_data (data_id) {
        data_id -> Text,
        receipt_id -> Text,
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
        receipt_kind -> Receipt_type,
        transaction_hash -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    transaction_actions (transaction_hash, index) {
        transaction_hash -> Text,
        index -> Int4,
        action_kind -> Action_type,
        args -> Jsonb,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    transactions (transaction_hash) {
        transaction_hash -> Text,
        block_height -> Numeric,
        chunk_hash -> Text,
        signer_id -> Text,
        public_key -> Text,
        nonce -> Numeric,
        receiver_id -> Text,
        signature -> Text,
        status -> Execution_outcome_status,
        receipt_id -> Text,
        receipt_conversion_gas_burnt -> Nullable<Numeric>,
        receipt_conversion_tokens_burnt -> Nullable<Numeric>,
    }
}

joinable!(chunks -> blocks (block_id));
joinable!(execution_outcome_receipts -> execution_outcomes (execution_outcome_receipt_id));
joinable!(execution_outcome_receipts -> receipts (execution_outcome_receipt_id));
joinable!(execution_outcomes -> receipts (receipt_id));
joinable!(receipts -> blocks (block_height));
joinable!(receipts -> transactions (transaction_hash));
joinable!(transaction_actions -> transactions (transaction_hash));
joinable!(transactions -> blocks (block_height));
joinable!(transactions -> chunks (chunk_hash));

allow_tables_to_appear_in_same_query!(
    blocks,
    chunks,
    execution_outcome_receipts,
    execution_outcomes,
    receipt_action_actions,
    receipt_action_input_data,
    receipt_action_output_data,
    receipt_actions,
    receipt_data,
    receipts,
    transaction_actions,
    transactions,
);
