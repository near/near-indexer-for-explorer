use std::collections::HashMap;
use std::convert::TryFrom;

use actix_diesel::dsl::AsyncRunQueryDsl;
use actix_diesel::Database;
use bigdecimal::BigDecimal;
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use futures::try_join;
use tracing::info;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

pub(crate) async fn handle_access_keys(
    pool: &actix_diesel::Database<PgConnection>,
    outcomes: &[near_indexer::IndexerExecutionOutcomeWithReceipt],
    block_height: near_primitives::types::BlockHeight,
) -> anyhow::Result<()> {
    if outcomes.is_empty() {
        return Ok(());
    }
    let successful_receipts = outcomes
        .iter()
        .filter(|outcome_with_receipt| {
            matches!(
                outcome_with_receipt.execution_outcome.outcome.status,
                near_primitives::views::ExecutionStatusView::SuccessValue(_)
                    | near_primitives::views::ExecutionStatusView::SuccessReceiptId(_)
            )
        })
        .map(|outcome_with_receipt| &outcome_with_receipt.receipt);

    let mut access_keys = HashMap::<(String, String), models::access_keys::AccessKey>::new();
    let mut deleted_accounts = HashMap::<String, String>::new();

    for receipt in successful_receipts {
        if let near_primitives::views::ReceiptEnumView::Action { actions, .. } = &receipt.receipt {
            for action in actions {
                match action {
                    near_primitives::views::ActionView::DeleteAccount { .. } => {
                        deleted_accounts.insert(
                            receipt.receiver_id.to_string(),
                            receipt.receipt_id.to_string(),
                        );
                        access_keys
                            .iter_mut()
                            .filter(|((_, receiver_id), _)| {
                                receiver_id == receipt.receiver_id.as_ref()
                            })
                            .for_each(|(_, access_key)| {
                                access_key.deleted_by_receipt_id =
                                    Some(receipt.receipt_id.to_string());
                            });
                    }
                    near_primitives::views::ActionView::AddKey {
                        public_key,
                        access_key,
                    } => {
                        access_keys.insert(
                            (public_key.to_string(), receipt.receiver_id.to_string()),
                            models::access_keys::AccessKey::from_action_view(
                                public_key,
                                &receipt.receiver_id,
                                access_key,
                                &receipt.receipt_id,
                                block_height,
                            ),
                        );
                    }
                    near_primitives::views::ActionView::DeleteKey { public_key } => {
                        access_keys
                            .entry((public_key.to_string(), receipt.receiver_id.to_string()))
                            .and_modify(|existing_access_key| {
                                existing_access_key.deleted_by_receipt_id =
                                    Some(receipt.receipt_id.to_string());
                            })
                            .or_insert_with(|| models::access_keys::AccessKey {
                                public_key: public_key.to_string(),
                                account_id: receipt.receiver_id.to_string(),
                                created_by_receipt_id: None,
                                deleted_by_receipt_id: Some(receipt.receipt_id.to_string()),
                                // this is a workaround to avoid additional struct with optional field
                                // permission_kind is not supposed to change on delete action
                                permission_kind: models::enums::AccessKeyPermission::FullAccess,
                                last_update_block_height: block_height.into(),
                            });
                    }
                    near_indexer::near_primitives::views::ActionView::Transfer { .. } => {
                        if receipt.receiver_id.len() != 64usize {
                            continue;
                        }
                        if let Ok(public_key_bytes) = hex::decode(receipt.receiver_id.as_ref()) {
                            if let Ok(public_key) =
                                near_crypto::ED25519PublicKey::try_from(&public_key_bytes[..])
                            {
                                access_keys.insert(
                                    (near_crypto::PublicKey::from(public_key.clone()).to_string(), receipt.receiver_id.to_string()),
                                    models::access_keys::AccessKey::from_action_view(
                                        &near_crypto::PublicKey::from(public_key.clone()),
                                        &receipt.receiver_id,
                                        &near_primitives::views::AccessKeyView {
                                            nonce: 0,
                                            permission: near_primitives::views::AccessKeyPermissionView::FullAccess
                                        },
                                        &receipt.receipt_id,
                                        block_height,
                                    ),
                                );
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    let (access_keys_to_insert, access_keys_to_update): (
        Vec<models::access_keys::AccessKey>,
        Vec<models::access_keys::AccessKey>,
    ) = access_keys
        .values()
        .cloned()
        .partition(|model| model.created_by_receipt_id.is_some());

    let delete_access_keys_for_deleted_accounts = async {
        let last_update_block_height: BigDecimal = block_height.into();
        for (account_id, deleted_by_receipt_id) in deleted_accounts {
            let target = schema::access_keys::table
                .filter(schema::access_keys::dsl::deleted_by_receipt_id.is_null())
                .filter(
                    schema::access_keys::dsl::last_update_block_height
                        .lt(last_update_block_height.clone()),
                )
                .filter(schema::access_keys::dsl::account_id.eq(account_id));

            crate::await_retry_or_panic!(
                diesel::update(target.clone())
                    .set((
                        schema::access_keys::dsl::deleted_by_receipt_id
                            .eq(deleted_by_receipt_id.clone()),
                        schema::access_keys::dsl::last_update_block_height
                            .eq(last_update_block_height.clone()),
                    ))
                    .execute_async(pool),
                10,
                "AccessKeys were deleting".to_string(),
                &deleted_by_receipt_id
            );
        }
        Ok(())
    };

    let update_access_keys_future = async {
        for value in access_keys_to_update {
            let target = schema::access_keys::table
                .filter(schema::access_keys::dsl::public_key.eq(value.public_key.clone()))
                .filter(
                    schema::access_keys::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                )
                .filter(schema::access_keys::dsl::account_id.eq(value.account_id));

            crate::await_retry_or_panic!(
                diesel::update(target.clone())
                    .set((
                        schema::access_keys::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::access_keys::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(pool),
                10,
                "AccessKeys were updating".to_string(),
                &value.public_key
            );
        }
        Ok(())
    };

    let add_access_keys_future = async {
        crate::await_retry_or_panic!(
            diesel::insert_into(schema::access_keys::table)
                .values(access_keys_to_insert.clone())
                .on_conflict_do_nothing()
                .execute_async(pool),
            10,
            "AccessKeys were stored in database".to_string(),
            &access_keys_to_insert
        );

        for value in access_keys_to_insert {
            let target = schema::access_keys::table
                .filter(schema::access_keys::dsl::public_key.eq(value.public_key.clone()))
                .filter(
                    schema::access_keys::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                )
                .filter(schema::access_keys::dsl::account_id.eq(value.account_id));

            crate::await_retry_or_panic!(
                diesel::update(target.clone())
                    .set((
                        schema::access_keys::dsl::created_by_receipt_id
                            .eq(value.created_by_receipt_id.clone()),
                        schema::access_keys::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::access_keys::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(pool),
                10,
                "AccessKeys were created".to_string(),
                &value.public_key
            );
        }
        Ok(())
    };

    try_join!(
        delete_access_keys_for_deleted_accounts,
        update_access_keys_future,
        add_access_keys_future
    )?;

    Ok(())
}

pub(crate) async fn store_access_keys_from_genesis(
    pool: Database<PgConnection>,
    access_keys_models: Vec<models::access_keys::AccessKey>,
) -> anyhow::Result<()> {
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Adding/updating access keys from genesis..."
    );

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::access_keys::table)
            .values(access_keys_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool),
        10,
        "AccessKeys were stored from genesis".to_string(),
        &access_keys_models
    );
    Ok(())
}
