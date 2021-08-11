use std::ops::{Add, Sub};
use std::time::{Duration, SystemTime};

use actix::Addr;
use actix_diesel::Database;
use chrono::NaiveDateTime;
use diesel::PgConnection;
use tokio::time;
use tokio::time::Instant;
use tracing::error;

pub(crate) mod circulating_supply;
pub(crate) mod lockup;
pub(crate) mod lockup_types;

// Compute circulating supply on a daily basis, starting from 13 Oct 2020
// (Transfers enabled moment on the Mainnet), and put it to the Indexer DB.
// Circulating supply is calculated by the formula:
// total_supply - sum(locked_tokens_on_each_lockup) - sum(locked_foundation_account)
// The value is always computed for the last block in a day (UTC).
pub(crate) async fn run_circulating_supply_computation(
    view_client: Addr<near_client::ViewClientActor>,
    pool: Database<PgConnection>,
) {
    // We perform actual computations 00:10 UTC each day to be sure that the data is finalized
    let mut day_to_compute = lockup::TRANSFERS_ENABLED
        .sub(Duration::from_secs(
            lockup::TRANSFERS_ENABLED.as_secs() % circulating_supply::DAY.as_secs(),
        ))
        .add(circulating_supply::DAY)
        .add(Duration::from_secs(10 * 60));

    loop {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");

        if now < day_to_compute {
            time::sleep_until(Instant::now().add(day_to_compute.sub(now))).await;
        }
        circulating_supply::wait_for_loading_needed_blocks(&view_client, &day_to_compute).await;

        match circulating_supply::check_and_collect_daily_circulating_supply(
            &view_client,
            &pool,
            &day_to_compute,
        )
        .await
        {
            Ok(_) => {
                day_to_compute = day_to_compute.add(circulating_supply::DAY);
            }
            Err(err) => {
                error!(
                    target: crate::AGGREGATED,
                    "Failed to compute circulating supply for {}: {}. Retry in {} hours",
                    NaiveDateTime::from_timestamp(day_to_compute.as_secs() as i64, 0).date(),
                    err,
                    circulating_supply::RETRY_DURATION.as_secs() / 60 / 60,
                );
                time::sleep(circulating_supply::RETRY_DURATION).await;
            }
        };
    }
}
