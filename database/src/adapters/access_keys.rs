use std::collections::HashMap;

use actix_diesel::dsl::AsyncRunQueryDsl;

use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use futures::try_join;
use tracing::info;

use crate::models;
use crate::schema;

pub async fn handle_access_keys(
    pool: &actix_diesel::Database<PgConnection>,
    state_changes: &[near_indexer_primitives::views::StateChangeWithCauseView],
    block_height: near_indexer_primitives::types::BlockHeight,
) -> anyhow::Result<()> {
    if state_changes.is_empty() {
        return Ok(());
    }

    let mut access_keys = HashMap::<(String, String), models::access_keys::AccessKey>::new();

    for state_change in state_changes {
        if let near_indexer_primitives::views::StateChangeCauseView::ReceiptProcessing {
            receipt_hash,
        } = state_change.cause
        {
            match &state_change.value {
                near_indexer_primitives::views::StateChangeValueView::AccessKeyUpdate {
                    account_id,
                    public_key,
                    access_key,
                } => {
                    access_keys.insert(
                        (public_key.to_string(), account_id.to_string()),
                        models::access_keys::AccessKey::from_action_view(
                            public_key,
                            account_id,
                            access_key,
                            &receipt_hash,
                            block_height,
                        ),
                    );
                }
                near_indexer_primitives::views::StateChangeValueView::AccessKeyDeletion {
                    account_id,
                    public_key,
                } => {
                    access_keys
                        .entry((public_key.to_string(), account_id.to_string()))
                        .and_modify(|existing_access_key| {
                            existing_access_key.deleted_by_receipt_id =
                                Some(receipt_hash.to_string());
                        })
                        .or_insert_with(|| models::access_keys::AccessKey {
                            public_key: public_key.to_string(),
                            account_id: account_id.to_string(),
                            created_by_receipt_id: None,
                            deleted_by_receipt_id: Some(receipt_hash.to_string()),
                            // this is a workaround to avoid additional struct with optional field
                            // permission_kind is not supposed to change on delete action
                            permission_kind: models::enums::AccessKeyPermission::FullAccess,
                            last_update_block_height: block_height.into(),
                        });
                }
                _ => continue,
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

    try_join!(update_access_keys_future, add_access_keys_future)?;

    Ok(())
}

pub(crate) async fn store_access_keys_from_genesis(
    pool: &actix_diesel::Database<PgConnection>,
    access_keys_models: Vec<models::access_keys::AccessKey>,
) -> anyhow::Result<()> {
    info!(
        target: crate::EXPLORER_DATABASE,
        "Adding/updating {} access keys from genesis...",
        access_keys_models.len(),
    );

    crate::await_retry_or_panic!(
        diesel::insert_into(schema::access_keys::table)
            .values(access_keys_models.clone())
            .on_conflict_do_nothing()
            .execute_async(pool),
        10,
        "Failed to store AccessKeys from genesis".to_string(),
        &access_keys_models
    );

    Ok(())
}
