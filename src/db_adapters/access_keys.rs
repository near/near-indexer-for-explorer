use std::collections::HashMap;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;
use diesel::pg::upsert::excluded;

pub(crate) async fn handle_access_keys(
    pool: &Pool<ConnectionManager<PgConnection>>,
    outcomes: &near_indexer::ExecutionOutcomesWithReceipts,
) {
    let successful_receipts = outcomes
        .values()
        .filter(|outcome_with_receipt| {
            match outcome_with_receipt.execution_outcome.outcome.status {
                near_primitives::views::ExecutionStatusView::SuccessValue(_)
                | near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => true,
                _ => false,
            }
        })
        .filter_map(|outcome_with_receipt| outcome_with_receipt.receipt.as_ref());

    let mut access_keys = HashMap::<(String, String), models::access_keys::AccessKey>::new();

    for receipt in successful_receipts {
        if let near_primitives::views::ReceiptEnumView::Action { actions, .. } = &receipt.receipt {
            for action in actions {
                match action {
                    near_primitives::views::ActionView::AddKey {
                        public_key,
                        access_key,
                    } => {
                        if let Some(existing_record) = access_keys
                            .get_mut(&(public_key.to_string(), receipt.receiver_id.to_string()))
                        {
                            existing_record.created_by_receipt_id =
                                Some(receipt.receipt_id.to_string());
                            existing_record.deleted_by_receipt_id = None;
                        } else {
                            access_keys.insert(
                                (public_key.to_string(), receipt.receiver_id.to_string()),
                                models::access_keys::AccessKey::from_action_view(
                                    public_key,
                                    &receipt.receiver_id,
                                    access_key,
                                    &receipt.receipt_id,
                                ),
                            );
                        }
                    }
                    near_primitives::views::ActionView::DeleteKey { public_key } => {
                        if let Some(existing_record) = access_keys
                            .get_mut(&(public_key.to_string(), receipt.receiver_id.to_string()))
                        {
                            existing_record.deleted_by_receipt_id =
                                Some(receipt.receipt_id.to_string());
                        } else {
                            access_keys.insert(
                                (public_key.to_string(), receipt.receiver_id.to_string()),
                                models::access_keys::AccessKey {
                                    public_key: public_key.to_string(),
                                    account_id: receipt.receiver_id.to_string(),
                                    created_by_receipt_id: None,
                                    deleted_by_receipt_id: Some(receipt.receipt_id.to_string()),
                                    permission: models::enums::AccessKeyPermission::NotApplicable,
                                },
                            );
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    let mut access_keys_to_insert: Vec<models::access_keys::AccessKey> = vec![];

    for (_, value) in access_keys {
        if value.created_by_receipt_id.is_none() {
            let target = schema::access_keys::table
                .filter(schema::access_keys::dsl::public_key.eq(value.public_key))
                .filter(schema::access_keys::dsl::account_id.eq(value.account_id));
            loop {
                match diesel::update(target.clone())
                    .set(
                        schema::access_keys::dsl::deleted_by_receipt_id
                            .eq(value.deleted_by_receipt_id.clone()),
                    )
                    .execute_async(&pool)
                    .await
                {
                    Ok(_) => break,
                    Err(async_error) => {
                        error!(
                            target: crate::INDEXER_FOR_EXPLORER,
                            "Error occurred while updating AccessKey. Retrying in {} milliseconds... \n {:#?}",
                            crate::INTERVAL.as_millis(),
                            async_error,
                        );
                        tokio::time::delay_for(crate::INTERVAL).await;
                    }
                }
            }
        } else {
            access_keys_to_insert.push(value);
        }
    }

    loop {
        match diesel::insert_into(schema::access_keys::table)
            .values(access_keys_to_insert.clone())
            .on_conflict((schema::access_keys::dsl::public_key, schema::access_keys::dsl::account_id))
            .do_update()
            .set((
                schema::access_keys::dsl::created_by_receipt_id
                    .eq(excluded(schema::access_keys::dsl::created_by_receipt_id)),
                schema::access_keys::dsl::deleted_by_receipt_id
                    .eq(excluded(schema::access_keys::dsl::deleted_by_receipt_id)),
            ))
            .execute_async(&pool)
            .await
        {
            Ok(_) => break,
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while AccessKeys were adding to database. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }
}
