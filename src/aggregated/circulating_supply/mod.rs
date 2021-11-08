use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use actix::Addr;
use actix_diesel::Database;
use anyhow::Context;
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::NaiveDateTime;
use diesel::PgConnection;
use tracing::{error, info, warn};

use near_indexer::near_primitives;

use crate::aggregated::{account_details, circulating_supply};
use crate::db_adapters::accounts;
use crate::db_adapters::aggregated::circulating_supply::{
    add_circulating_supply, get_precomputed_circulating_supply_for_timestamp,
};
use crate::db_adapters::blocks;
use crate::models;
use crate::models::aggregated::circulating_supply::CirculatingSupply;

mod lockup;
mod lockup_types;

const DAY: Duration = Duration::from_secs(60 * 60 * 24);
const RETRY_DURATION: Duration = Duration::from_secs(60 * 60 * 2);

// Compute circulating supply on a daily basis, starting from 13 Oct 2020
// (Transfers enabled moment on the Mainnet), and put it to the Indexer DB.
// Circulating supply is calculated by the formula:
// total_supply - sum(locked_tokens_on_each_lockup) - sum(locked_foundation_account)
// The value is always computed for the last block in a day (UTC).
pub(super) async fn run_circulating_supply_computation(
    view_client: Addr<near_client::ViewClientActor>,
    pool: Database<PgConnection>,
) {
    // We perform actual computations 00:10 UTC each day to be sure that the data is finalized
    let mut day_to_compute = lockup::TRANSFERS_ENABLED
        .sub(Duration::from_secs(
            lockup::TRANSFERS_ENABLED.as_secs() % DAY.as_secs(),
        ))
        .add(DAY)
        .add(Duration::from_secs(10 * 60));

    loop {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");

        if now < day_to_compute {
            tokio::time::sleep_until(tokio::time::Instant::now().add(day_to_compute.sub(now)))
                .await;
        }
        wait_for_loading_needed_blocks(&view_client, &day_to_compute).await;

        match check_and_collect_daily_circulating_supply(&view_client, &pool, &day_to_compute).await
        {
            Ok(_) => {
                day_to_compute = day_to_compute.add(DAY);
            }
            Err(err) => {
                error!(
                    target: crate::AGGREGATED,
                    "Failed to compute circulating supply for {}: {}. Retry in {} hours",
                    NaiveDateTime::from_timestamp(day_to_compute.as_secs() as i64, 0).date(),
                    err,
                    RETRY_DURATION.as_secs() / 60 / 60,
                );
                tokio::time::sleep(RETRY_DURATION).await;
            }
        };
    }
}

async fn check_and_collect_daily_circulating_supply(
    view_client: &Addr<near_client::ViewClientActor>,
    pool: &Database<PgConnection>,
    request_datetime: &Duration,
) -> anyhow::Result<Option<CirculatingSupply>> {
    let start_of_day = request_datetime.as_nanos()
        - request_datetime.as_nanos() % circulating_supply::DAY.as_nanos();
    let printable_date = NaiveDateTime::from_timestamp(request_datetime.as_secs() as i64, 0).date();
    let block = blocks::get_latest_block_before_timestamp(pool, start_of_day as u64).await?;
    let block_timestamp = block
        .block_timestamp
        .to_u64()
        .context("`block_timestamp` expected to be u64")?;

    match get_precomputed_circulating_supply_for_timestamp(pool, block_timestamp).await {
        Ok(None) => {
            info!(
                target: crate::AGGREGATED,
                "Computing circulating supply for {} (timestamp {})",
                printable_date,
                block_timestamp
            );
            let supply = compute_circulating_supply_for_block(pool, view_client, &block).await?;
            add_circulating_supply(pool, &supply).await;
            info!(
                target: crate::AGGREGATED,
                "Circulating supply for {} (timestamp {}) is {}",
                printable_date,
                block_timestamp,
                supply.circulating_tokens_supply
            );
            Ok(Some(supply))
        }
        Ok(Some(supply)) => {
            info!(
                target: crate::AGGREGATED,
                "Circulating supply for {} (timestamp {}) was already computed: {}",
                printable_date,
                block_timestamp,
                supply
            );
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

async fn compute_circulating_supply_for_block(
    pool: &Database<PgConnection>,
    view_client: &Addr<near_client::ViewClientActor>,
    block: &models::Block,
) -> anyhow::Result<CirculatingSupply> {
    let block_timestamp = block
        .block_timestamp
        .to_u64()
        .context("`block_timestamp` expected to be u64")?;
    let block_height = block
        .block_height
        .to_u64()
        .context("`block_height` expected to be u64")?;
    let total_supply = block
        .total_supply
        .to_string()
        .parse::<u128>()
        .context("`total_supply` expected to be u128")?;

    let lockup_account_ids =
        accounts::get_lockup_account_ids_at_block_height(pool, &block_height).await?;

    let mut lockups_locked_tokens: u128 = 0;
    let mut unfinished_lockup_contracts_count: i32 = 0;

    for lockup_account_id in &lockup_account_ids {
        let state =
            lockup::get_lockup_contract_state(view_client, lockup_account_id, &block_height)
                .await
                .with_context(|| {
                    format!(
                        "Failed to get lockup contract details for {}",
                        lockup_account_id
                    )
                })?;
        let code_hash =
            account_details::get_contract_code_hash(view_client, lockup_account_id, &block_height)
                .await?;
        let is_lockup_with_bug = lockup::is_bug_inside_contract(&code_hash, lockup_account_id)?;
        let locked_amount = state
            .get_locked_amount(block_timestamp, is_lockup_with_bug)
            .0;
        lockups_locked_tokens += locked_amount;
        if locked_amount > 0 {
            unfinished_lockup_contracts_count += 1;
        }
    }

    // The list is taken from the conversation with Yessin
    let foundation_locked_account_ids: [near_primitives::types::AccountId; 2] = [
        near_primitives::types::AccountId::from_str("lockup.near")
            .expect("lockup.near expected to be a valid AccountId"),
        near_primitives::types::AccountId::from_str("contributors.near")
            .expect("contributors.near expected to be a valid AccountId"),
    ];
    let mut foundation_locked_tokens: u128 = 0;
    for account_id in &foundation_locked_account_ids {
        foundation_locked_tokens +=
            account_details::get_account_balance(view_client, account_id, &block_height).await?;
    }

    let circulating_supply: u128 = total_supply - foundation_locked_tokens - lockups_locked_tokens;

    Ok(CirculatingSupply {
        computed_at_block_timestamp: BigDecimal::from(block_timestamp),
        computed_at_block_hash: (&block.block_hash).to_string(),
        circulating_tokens_supply: BigDecimal::from_str(&circulating_supply.to_string())
            .context("`circulating_tokens_supply` expected to be u128")?,
        total_tokens_supply: BigDecimal::from_str(&total_supply.to_string())
            .context("`total_supply` expected to be u128")?,
        total_lockup_contracts_count: lockup_account_ids.len() as i32,
        unfinished_lockup_contracts_count,
        foundation_locked_tokens: BigDecimal::from_str(&foundation_locked_tokens.to_string())
            .context("`foundation_locked_supply` expected to be u128")?,
        lockups_locked_tokens: BigDecimal::from_str(&lockups_locked_tokens.to_string())
            .context("`lockups_locked_supply` expected to be u128")?,
    })
}

async fn wait_for_loading_needed_blocks(
    view_client: &Addr<near_client::ViewClientActor>,
    day_to_compute: &Duration,
) {
    loop {
        match get_final_block_timestamp(view_client).await {
            Ok(timestamp) => {
                if timestamp > *day_to_compute {
                    return;
                }
                warn!(
                        target: crate::AGGREGATED,
                        "Blocks are not loaded to calculate circulating supply for {}. Wait for {} hours",
                        NaiveDateTime::from_timestamp(day_to_compute.as_secs() as i64, 0).date(),
                        circulating_supply::RETRY_DURATION.as_secs() / 60 / 60,
                    );
            }
            Err(err) => {
                error!(
                    target: crate::AGGREGATED,
                    "Failed to get latest block timestamp: {}. Retry in {} hours",
                    err,
                    circulating_supply::RETRY_DURATION.as_secs() / 60 / 60,
                );
            }
        }
        tokio::time::sleep(circulating_supply::RETRY_DURATION).await;
    }
}

async fn get_final_block_timestamp(
    view_client: &Addr<near_client::ViewClientActor>,
) -> anyhow::Result<Duration> {
    let block_reference =
        near_primitives::types::BlockReference::Finality(near_primitives::types::Finality::Final);
    let query = near_client::GetBlock(block_reference);

    let block_response = view_client
        .send(query)
        .await
        .context("Failed to deliver response")?
        .context("Invalid request")?;

    Ok(Duration::from_nanos(block_response.header.timestamp))
}
