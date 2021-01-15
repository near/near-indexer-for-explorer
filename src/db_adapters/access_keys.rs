use std::collections::HashMap;
use std::convert::TryFrom;

use bigdecimal::BigDecimal;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use futures::{join, StreamExt};
use itertools::Itertools;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::{error, info};

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

pub(crate) async fn handle_access_keys(
    pool: &Pool<ConnectionManager<PgConnection>>,
    outcomes: &[near_indexer::IndexerExecutionOutcomeWithReceipt],
    block_height: near_primitives::types::BlockHeight,
) {
    if outcomes.is_empty() {
        return;
    }
    let successful_receipts = outcomes
        .iter()
        .filter(|outcome_with_receipt| {
            match outcome_with_receipt.execution_outcome.outcome.status {
                near_primitives::views::ExecutionStatusView::SuccessValue(_)
                | near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => true,
                _ => false,
            }
        })
        .filter_map(|outcome_with_receipt| outcome_with_receipt.receipt.as_ref());

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
                            .filter(|((_, receiver_id), _)| receiver_id == &receipt.receiver_id)
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
                        if let Ok(public_key_bytes) = hex::decode(&receipt.receiver_id) {
                            if let Ok(public_key) =
                                near_crypto::ED25519PublicKey::try_from(&public_key_bytes[..])
                            {
                                access_keys.insert(
                                    (near_crypto::PublicKey::from(public_key).to_string(), receipt.receiver_id.to_string()),
                                    models::access_keys::AccessKey::from_action_view(
                                        &near_crypto::PublicKey::from(public_key),
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

            let mut interval = crate::INTERVAL;
            loop {
                match diesel::update(target.clone())
                    .set((
                        schema::access_keys::dsl::deleted_by_receipt_id
                            .eq(deleted_by_receipt_id.clone()),
                        schema::access_keys::dsl::last_update_block_height
                            .eq(last_update_block_height.clone()),
                    ))
                    .execute_async(&pool)
                    .await
                {
                    Ok(_) => break,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while updating AccessKey. Retrying in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::delay_for(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        }
    };

    let update_access_keys_future = async {
        for value in access_keys_to_update {
            let target = schema::access_keys::table
                .filter(schema::access_keys::dsl::public_key.eq(value.public_key))
                .filter(
                    schema::access_keys::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                )
                .filter(schema::access_keys::dsl::account_id.eq(value.account_id));

            let mut interval = crate::INTERVAL;
            loop {
                match diesel::update(target.clone())
                    .set((
                        schema::access_keys::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::access_keys::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(&pool)
                    .await
                {
                    Ok(_) => break,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while updating AccessKey. Retrying in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::delay_for(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        }
    };

    let add_access_keys_future = async {
        let mut interval = crate::INTERVAL;
        loop {
            match diesel::insert_into(schema::access_keys::table)
                .values(access_keys_to_insert.clone())
                .on_conflict_do_nothing()
                .execute_async(&pool)
                .await
            {
                Ok(_) => break,
                Err(async_error) => {
                    error!(
                        target: crate::INDEXER_FOR_EXPLORER,
                        "Error occurred while AccessKeys were adding to database. Retrying in {} milliseconds... \n {:#?}",
                        interval.as_millis(),
                        async_error,
                    );
                    tokio::time::delay_for(interval).await;
                    if interval < crate::MAX_DELAY_TIME {
                        interval *= 2;
                    }
                }
            }
        }

        for value in access_keys_to_insert {
            let target = schema::access_keys::table
                .filter(schema::access_keys::dsl::public_key.eq(value.public_key))
                .filter(
                    schema::access_keys::dsl::last_update_block_height
                        .lt(value.last_update_block_height.clone()),
                )
                .filter(schema::access_keys::dsl::account_id.eq(value.account_id));

            let mut interval = crate::INTERVAL;
            loop {
                match diesel::update(target.clone())
                    .set((
                        schema::access_keys::dsl::created_by_receipt_id
                            .eq(value.created_by_receipt_id.clone()),
                        schema::access_keys::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                        schema::access_keys::dsl::last_update_block_height
                            .eq(value.last_update_block_height.clone()),
                    ))
                    .execute_async(&pool)
                    .await
                {
                    Ok(_) => break,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while updating AccessKey. Retrying in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::delay_for(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        }
    };

    join!(
        delete_access_keys_for_deleted_accounts,
        update_access_keys_future,
        add_access_keys_future
    );
}

pub(crate) async fn store_access_keys_from_genesis(near_config: near_indexer::NearConfig) {
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Adding/updating access keys from genesis..."
    );
    let pool = crate::models::establish_connection();

    let genesis_height = near_config.genesis.config.genesis_height;

    let access_keys_models = near_config
        .genesis
        .records
        .as_ref()
        .iter()
        .filter_map(|record| {
            if let near_indexer::near_primitives::state_record::StateRecord::AccessKey {
                account_id,
                public_key,
                access_key,
            } = record
            {
                Some(models::access_keys::AccessKey::from_genesis(
                    &public_key,
                    &account_id,
                    &access_key,
                    genesis_height,
                ))
            } else {
                None
            }
        });

    let portion_size = 5000;
    let total_access_keys_chunks = access_keys_models.clone().count() / portion_size + 1;
    let access_keys_portion = access_keys_models.chunks(portion_size);

    let insert_genesis_access_keys: futures::stream::FuturesUnordered<_> = access_keys_portion
        .into_iter()
        .map(|access_keys| async {
            let collected_access_keys = access_keys.collect::<Vec<models::access_keys::AccessKey>>();
            let mut interval = crate::INTERVAL;
            loop {
                match diesel::insert_into(schema::access_keys::table)
                    .values(collected_access_keys.clone())
                    .on_conflict_do_nothing()
                    .execute_async(&pool)
                    .await
                {
                    Ok(result) => break result,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while AccessKeys from genesis were being added to database. Retrying in {} milliseconds... \n {:#?}",
                            interval.as_millis(),
                            async_error,
                        );
                        tokio::time::delay_for(interval).await;
                        if interval < crate::MAX_DELAY_TIME {
                            interval *= 2;
                        }
                    }
                }
            }
        })
        .collect();

    let mut insert_genesis_access_keys = insert_genesis_access_keys.enumerate();

    while let Some((index, _result)) = insert_genesis_access_keys.next().await {
        info!(
            target: crate::INDEXER_FOR_EXPLORER,
            "AccessKeys from genesis adding {}%",
            index * 100 / total_access_keys_chunks
        );
    }

    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "AccessKeys from genesis were added/updated successful."
    );
}
