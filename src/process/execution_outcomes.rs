use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use tokio_diesel::AsyncRunQueryDsl;
use tracing::error;

use crate::models;
use crate::schema;

/// Saves ExecutionOutcome to database and then saves ExecutionOutcomesReceipts
pub(crate) async fn process_execution_outcomes(
    pool: &Pool<ConnectionManager<PgConnection>>,
    execution_outcomes: Vec<&near_indexer::near_primitives::views::ExecutionOutcomeWithIdView>,
) {
    for outcome in execution_outcomes {
        let model = models::execution_outcomes::ExecutionOutcome::from(outcome);
        loop {
            match diesel::insert_into(schema::execution_outcomes::table)
                .values(model.clone())
                .on_conflict_do_nothing()
                .execute_async(&pool)
                .await
            {
                Ok(_) => break,
                Err(async_error) => {
                    error!(
                        target: crate::INDEXER_FOR_EXPLORER,
                        "Error occurred while ExecutionOutcome were adding to database. Retrying in {} milliseconds... \n {:#?}",
                        crate::INTERVAL.as_millis(),
                        async_error
                    );
                    tokio::time::delay_for(crate::INTERVAL).await;
                }
            }
        }

        let child_receipt_models: Vec<models::execution_outcomes::ExecutionOutcomeReceipt> =
            outcome
                .outcome
                .receipt_ids
                .iter()
                .enumerate()
                .map(
                    |(index, receipt_id)| models::execution_outcomes::ExecutionOutcomeReceipt {
                        execution_outcome_receipt_id: outcome.id.as_ref().to_vec(),
                        index: index as i32,
                        receipt_id: receipt_id.as_ref().to_vec(),
                    },
                )
                .collect();

        loop {
            match diesel::insert_into(schema::execution_outcome_receipts::table)
                .values(child_receipt_models.clone())
                .on_conflict_do_nothing()
                .execute_async(&pool)
                .await
            {
                Ok(_) => break,
                Err(async_error) => {
                    error!(
                        target: crate::INDEXER_FOR_EXPLORER,
                        "Error occurred while ExecutionOutcomeReceipt were adding to database. Retrying in {} milliseconds... \n {:#?}",
                        crate::INTERVAL.as_millis(),
                        async_error
                    );
                    tokio::time::delay_for(crate::INTERVAL).await;
                }
            }
        }
    }
}
