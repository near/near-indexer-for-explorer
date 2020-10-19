use std::collections::HashMap;

use diesel::pg::upsert::excluded;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use futures::join;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use near_indexer::near_primitives;

use crate::models;
use crate::schema;

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
                            });
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

    let update_access_keys_future = async {
        for value in access_keys_to_update {
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
        }
    };

    let add_access_keys_future = async {
        loop {
            match diesel::insert_into(schema::access_keys::table)
                .values(access_keys_to_insert.clone())
                .on_conflict((
                    schema::access_keys::dsl::public_key,
                    schema::access_keys::dsl::account_id,
                ))
                .do_update()
                .set((
                    schema::access_keys::dsl::created_by_receipt_id
                        .eq(excluded(schema::access_keys::dsl::created_by_receipt_id)),
                    schema::access_keys::dsl::deleted_by_receipt_id
                        .eq(excluded(schema::access_keys::dsl::deleted_by_receipt_id)),
                    schema::access_keys::dsl::permission_kind
                        .eq(excluded(schema::access_keys::dsl::permission_kind)),
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
    };

    join!(update_access_keys_future, add_access_keys_future);
}
