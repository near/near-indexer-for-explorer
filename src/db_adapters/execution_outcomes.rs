use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use tokio_diesel::AsyncRunQueryDsl;
use tracing::{debug, error};

use crate::models;
use crate::schema;
use diesel::pg::expression::array_comparison::any;

/// Saves ExecutionOutcome to database and then saves ExecutionOutcomesReceipts
pub(crate) async fn store_execution_outcomes(
    pool: &Pool<ConnectionManager<PgConnection>>,
    execution_outcomes: &near_indexer::ExecutionOutcomesWithReceipts,
) {
    let known_receipt_ids: std::collections::HashSet<String> = loop {
        match schema::receipts::table
            .filter(
                schema::receipts::dsl::receipt_id.eq(any(execution_outcomes
                    .keys()
                    .map(|key| key.to_string())
                    .collect::<Vec<_>>())),
            )
            .select(schema::receipts::dsl::receipt_id)
            .load_async(&pool)
            .await
        {
            Ok(res) => {
                break res.into_iter().collect();
            }
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while fetching the parent receipt for ExecutionOutcome. Retrying in {} milliseconds... \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    };

    let mut outcome_models: Vec<models::execution_outcomes::ExecutionOutcome> = vec![];
    let mut outcome_receipt_models: Vec<models::execution_outcomes::ExecutionOutcomeReceipt> =
        vec![];
    for outcome in execution_outcomes
        .values()
        .filter(|outcome| known_receipt_ids.contains(&(outcome.execution_outcome.id).to_string()))
    {
        let model = models::execution_outcomes::ExecutionOutcome::from(&outcome.execution_outcome);
        outcome_models.push(model);

        outcome_receipt_models.extend(
            outcome
                .execution_outcome
                .outcome
                .receipt_ids
                .iter()
                .enumerate()
                .map(
                    |(index, receipt_id)| models::execution_outcomes::ExecutionOutcomeReceipt {
                        execution_outcome_receipt_id: outcome.execution_outcome.id.to_string(),
                        index: index as i32,
                        receipt_id: receipt_id.to_string(),
                    },
                ),
        );
    }

    loop {
        match diesel::insert_into(schema::execution_outcomes::table)
            .values(outcome_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(affected_rows) => {
                debug!(target: crate::INDEXER_FOR_EXPLORER, "outcomes added {}", affected_rows);
                break;
            },
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ExecutionOutcome were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &outcome_models,
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }

    loop {
        match diesel::insert_into(schema::execution_outcome_receipts::table)
            .values(outcome_receipt_models.clone())
            .on_conflict_do_nothing()
            .execute_async(&pool)
            .await
        {
            Ok(affected_rows) => {
                debug!(target: crate::INDEXER_FOR_EXPLORER, "outcome related receipts added {}", affected_rows);
                break;
            },
            Err(async_error) => {
                error!(
                    target: crate::INDEXER_FOR_EXPLORER,
                    "Error occurred while ExecutionOutcomeReceipt were adding to database. Retrying in {} milliseconds... \n {:#?} \n {:#?}",
                    crate::INTERVAL.as_millis(),
                    async_error,
                    &outcome_receipt_models
                );
                tokio::time::delay_for(crate::INTERVAL).await;
            }
        }
    }
}
