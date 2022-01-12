table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    access_keys (public_key, account_id) {
        public_key -> Text,
        account_id -> Text,
        created_by_receipt_id -> Nullable<Text>,
        deleted_by_receipt_id -> Nullable<Text>,
        permission_kind -> Access_key_permission_kind,
        last_update_block_height -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    account_changes (id) {
        id -> Int8,
        affected_account_id -> Text,
        changed_in_block_timestamp -> Numeric,
        changed_in_block_hash -> Text,
        caused_by_transaction_hash -> Nullable<Text>,
        caused_by_receipt_id -> Nullable<Text>,
        update_reason -> State_change_reason_kind,
        affected_account_nonstaked_balance -> Numeric,
        affected_account_staked_balance -> Numeric,
        affected_account_storage_usage -> Numeric,
        index_in_block -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    accounts (id) {
        id -> Int8,
        account_id -> Text,
        created_by_receipt_id -> Nullable<Text>,
        deleted_by_receipt_id -> Nullable<Text>,
        last_update_block_height -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    action_receipt_actions (receipt_id, index_in_action_receipt) {
        receipt_id -> Text,
        index_in_action_receipt -> Int4,
        action_kind -> Action_kind,
        args -> Jsonb,
        receipt_predecessor_account_id -> Text,
        receipt_receiver_account_id -> Text,
        receipt_included_in_block_timestamp -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    action_receipt_input_data (input_data_id, input_to_receipt_id) {
        input_data_id -> Text,
        input_to_receipt_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    action_receipt_output_data (output_data_id, output_from_receipt_id) {
        output_data_id -> Text,
        output_from_receipt_id -> Text,
        receiver_account_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    action_receipts (receipt_id) {
        receipt_id -> Text,
        signer_account_id -> Text,
        signer_public_key -> Text,
        gas_price -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    aggregated__circulating_supply (computed_at_block_hash) {
        computed_at_block_timestamp -> Numeric,
        computed_at_block_hash -> Text,
        circulating_tokens_supply -> Numeric,
        total_tokens_supply -> Numeric,
        total_lockup_contracts_count -> Int4,
        unfinished_lockup_contracts_count -> Int4,
        foundation_locked_tokens -> Numeric,
        lockups_locked_tokens -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    assets__fungible_token_events (emitted_for_receipt_id, emitted_at_block_timestamp, emitted_in_shard_id, emitted_index_of_event_entry_in_shard, emitted_by_contract_account_id, amount, event_kind, token_old_owner_account_id, token_new_owner_account_id, event_memo) {
        emitted_for_receipt_id -> Text,
        emitted_at_block_timestamp -> Numeric,
        emitted_in_shard_id -> Numeric,
        emitted_index_of_event_entry_in_shard -> Int4,
        emitted_by_contract_account_id -> Text,
        amount -> Text,
        event_kind -> Ft_event_kind,
        token_old_owner_account_id -> Text,
        token_new_owner_account_id -> Text,
        event_memo -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    assets__non_fungible_token_events (emitted_for_receipt_id, emitted_at_block_timestamp, emitted_in_shard_id, emitted_index_of_event_entry_in_shard, emitted_by_contract_account_id, token_id, event_kind, token_old_owner_account_id, token_new_owner_account_id, token_authorized_account_id, event_memo) {
        emitted_for_receipt_id -> Text,
        emitted_at_block_timestamp -> Numeric,
        emitted_in_shard_id -> Numeric,
        emitted_index_of_event_entry_in_shard -> Int4,
        emitted_by_contract_account_id -> Text,
        token_id -> Text,
        event_kind -> Nft_event_kind,
        token_old_owner_account_id -> Text,
        token_new_owner_account_id -> Text,
        token_authorized_account_id -> Text,
        event_memo -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    blocks (block_hash) {
        block_height -> Numeric,
        block_hash -> Text,
        prev_block_hash -> Text,
        block_timestamp -> Numeric,
        total_supply -> Numeric,
        gas_price -> Numeric,
        author_account_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    chunks (chunk_hash) {
        included_in_block_hash -> Text,
        chunk_hash -> Text,
        shard_id -> Numeric,
        signature -> Text,
        gas_limit -> Numeric,
        gas_used -> Numeric,
        author_account_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    data_receipts (data_id) {
        data_id -> Text,
        receipt_id -> Text,
        data -> Nullable<Bytea>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    execution_outcome_receipts (executed_receipt_id, index_in_execution_outcome, produced_receipt_id) {
        executed_receipt_id -> Text,
        index_in_execution_outcome -> Int4,
        produced_receipt_id -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    execution_outcomes (receipt_id) {
        receipt_id -> Text,
        executed_in_block_hash -> Text,
        executed_in_block_timestamp -> Numeric,
        index_in_chunk -> Int4,
        gas_burnt -> Numeric,
        tokens_burnt -> Numeric,
        executor_account_id -> Text,
        status -> Execution_outcome_status,
        shard_id -> Numeric,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    receipts (receipt_id) {
        receipt_id -> Text,
        included_in_block_hash -> Text,
        included_in_chunk_hash -> Text,
        index_in_chunk -> Int4,
        included_in_block_timestamp -> Numeric,
        predecessor_account_id -> Text,
        receiver_account_id -> Text,
        receipt_kind -> Receipt_kind,
        originated_from_transaction_hash -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    transaction_actions (transaction_hash, index_in_transaction) {
        transaction_hash -> Text,
        index_in_transaction -> Int4,
        action_kind -> Action_kind,
        args -> Jsonb,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::enums::*;

    transactions (transaction_hash) {
        transaction_hash -> Text,
        included_in_block_hash -> Text,
        included_in_chunk_hash -> Text,
        index_in_chunk -> Int4,
        block_timestamp -> Numeric,
        signer_account_id -> Text,
        signer_public_key -> Text,
        nonce -> Numeric,
        receiver_account_id -> Text,
        signature -> Text,
        status -> Execution_outcome_status,
        converted_into_receipt_id -> Text,
        receipt_conversion_gas_burnt -> Nullable<Numeric>,
        receipt_conversion_tokens_burnt -> Nullable<Numeric>,
    }
}

joinable!(account_changes -> blocks (changed_in_block_hash));
joinable!(account_changes -> receipts (caused_by_receipt_id));
joinable!(account_changes -> transactions (caused_by_transaction_hash));
joinable!(action_receipt_actions -> receipts (receipt_id));
joinable!(aggregated__circulating_supply -> blocks (computed_at_block_hash));
joinable!(assets__fungible_token_events -> receipts (emitted_for_receipt_id));
joinable!(assets__non_fungible_token_events -> receipts (emitted_for_receipt_id));
joinable!(chunks -> blocks (included_in_block_hash));
joinable!(execution_outcome_receipts -> execution_outcomes (executed_receipt_id));
joinable!(execution_outcome_receipts -> receipts (executed_receipt_id));
joinable!(execution_outcomes -> blocks (executed_in_block_hash));
joinable!(execution_outcomes -> receipts (receipt_id));
joinable!(receipts -> blocks (included_in_block_hash));
joinable!(receipts -> chunks (included_in_chunk_hash));
joinable!(receipts -> transactions (originated_from_transaction_hash));
joinable!(transaction_actions -> transactions (transaction_hash));
joinable!(transactions -> blocks (included_in_block_hash));
joinable!(transactions -> chunks (included_in_chunk_hash));

allow_tables_to_appear_in_same_query!(
    access_keys,
    account_changes,
    accounts,
    action_receipt_actions,
    action_receipt_input_data,
    action_receipt_output_data,
    action_receipts,
    aggregated__circulating_supply,
    assets__fungible_token_events,
    assets__non_fungible_token_events,
    blocks,
    chunks,
    data_receipts,
    execution_outcome_receipts,
    execution_outcomes,
    receipts,
    transaction_actions,
    transactions,
);
