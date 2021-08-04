use std::ops::{Add, Div};
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use actix::Addr;
use actix_diesel::Database;
use actix_web::rt::time;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::PgConnection;
use tokio::time::Instant;
use tracing::{error, info};

use crate::aggregated::account_details;
use crate::aggregated::account_details::get_account_balance;
use crate::aggregated::circulating_supply::lockup;
use crate::aggregated::circulating_supply::lockup::TRANSFERS_ENABLED;
use crate::db_adapters::accounts;
use crate::db_adapters::aggregated::circulating_supply::{
    add_circulating_supply, get_precomputed_circulating_supply_for_timestamp,
};
use crate::db_adapters::blocks::get_latest_block_before_timestamp;
use crate::models::aggregated::circulating_supply::CirculatingSupply;
use crate::models::Block;

const DAY: std::time::Duration = std::time::Duration::from_secs(86400);

// Compute circulating supply on a daily basis, starting from 13 Oct 2020
// (Transfers enabled moment on the Mainnet), and put it to the Indexer DB.
// Circulating supply is calculated by the formula:
// total_supply - sum(locked_tokens_on_each_lockup) - sum(locked_foundation_account)
// The value is always computed for the last block in a day (UTC).
pub(crate) async fn compute_circulating_supply(
    view_client: Addr<near_client::ViewClientActor>,
    pool: Database<PgConnection>,
) {
    let retry_duration = DAY.div(12);

    // Adding 10 minutes to be sure that the picture is finalized and we can collect all needed data
    let ten_minutes: u64 = 10 * 60 * 1000_000_000;
    let mut day_to_compute =
        TRANSFERS_ENABLED - TRANSFERS_ENABLED % (DAY.as_nanos() as u64) + ten_minutes;

    loop {
        time::sleep_until(get_instant_for_timestamp(day_to_compute)).await;
        match check_and_collect_daily_circulating_supply(&view_client, &pool, day_to_compute).await
        {
            Ok(_) => {
                day_to_compute += DAY.as_nanos() as u64;
            }
            Err(_) => {
                error!(
                    target: crate::AGGREGATED,
                    "Failed to compute circulating supply for timestamp {}. Retry in {} hours",
                    day_to_compute,
                    retry_duration.as_secs() / 60 / 60,
                );
                time::sleep(retry_duration).await;
            }
        };
    }
}

async fn check_and_collect_daily_circulating_supply(
    view_client: &Addr<near_client::ViewClientActor>,
    pool: &Database<PgConnection>,
    request_timestamp: u64,
) -> Result<Option<CirculatingSupply>, String> {
    let start_of_day = request_timestamp - request_timestamp % (DAY.as_nanos() as u64);
    let block = get_latest_block_before_timestamp(&pool, start_of_day).await?;
    let block_timestamp = block
        .block_timestamp
        .to_u64()
        .ok_or("`block_timestamp` expected to be u64")?;

    match get_precomputed_circulating_supply_for_timestamp(&pool, block_timestamp).await {
        Ok(None) => {
            info!(
                target: crate::AGGREGATED,
                "Computing circulating supply for the timestamp {}", block_timestamp
            );
            let supply = compute_circulating_supply_for_block(&pool, view_client, &block).await?;
            add_circulating_supply(&pool, &supply).await;
            info!(
                target: crate::AGGREGATED,
                "Circulating supply for the timestamp {} is {}",
                block_timestamp,
                supply.circulating_tokens_supply
            );
            Ok(Some(supply))
        }
        Ok(Some(supply)) => {
            info!(
                target: crate::AGGREGATED,
                "Circulating supply for the timestamp {} was already computed: {}",
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
    block: &Block,
) -> Result<CirculatingSupply, String> {
    let block_timestamp = block
        .block_timestamp
        .to_u64()
        .ok_or("`block_timestamp` expected to be u64")?;
    let block_height = block
        .block_height
        .to_u64()
        .ok_or("`block_height` expected to be u64")?;
    let total_supply = u128::from_str_radix(&*block.total_supply.to_string(), 10)
        .map_err(|_| "`total_supply` expected to be u128")?;

    let lockup_ids = accounts::get_lockup_ids_at_block_height(&pool, block_height).await?;

    let mut lockups_locked_tokens: u128 = 0;
    let mut unfinished_lockups_count: u128 = 0;

    for lockup_id in lockup_ids.iter() {
        let state = lockup::get_account_state(&view_client, lockup_id, block_height).await?;
        let code_hash =
            account_details::get_contract_code_hash(&view_client, lockup_id, block_height).await?;
        let is_lockup_with_bug = lockup::is_bug_inside_contract(&code_hash, lockup_id)?;
        let locked_amount = state
            .get_locked_amount(block_timestamp, is_lockup_with_bug)
            .0;
        lockups_locked_tokens += locked_amount;
        if locked_amount > 0 {
            unfinished_lockups_count += 1;
        }
    }

    // The list is taken from the conversation with Yessin
    let foundation_locked_accs = ["lockup.near", "contributors.near"];
    let mut foundation_locked_tokens: u128 = 0;
    for acc in foundation_locked_accs.iter() {
        foundation_locked_tokens += get_account_balance(&view_client, acc, block_height).await?;
    }

    let circulating_supply: u128 = total_supply - foundation_locked_tokens - lockups_locked_tokens;

    Ok(CirculatingSupply {
        computed_at_block_timestamp: BigDecimal::from(block_timestamp),
        computed_at_block_hash: (&block.block_hash).to_string(),
        circulating_tokens_supply: BigDecimal::from_str(circulating_supply.to_string().as_str())
            .map_err(|_| "`circulating_tokens_supply` expected to be u128")?,
        total_tokens_supply: BigDecimal::from_str(total_supply.to_string().as_str())
            .map_err(|_| "`total_supply` expected to be u128")?,
        total_lockup_contracts_count: BigDecimal::from_str(lockup_ids.len().to_string().as_str())
            .map_err(|_| "`lockups_number` expected to be u128")?,
        unfinished_lockup_contracts_count: BigDecimal::from_str(
            unfinished_lockups_count.to_string().as_str(),
        )
        .map_err(|_| "`active_lockups_number` expected to be u128")?,
        foundation_locked_tokens: BigDecimal::from_str(
            foundation_locked_tokens.to_string().as_str(),
        )
        .map_err(|_| "`foundation_locked_supply` expected to be u128")?,
        lockups_locked_tokens: BigDecimal::from_str(lockups_locked_tokens.to_string().as_str())
            .map_err(|_| "`lockups_locked_supply` expected to be u128")?,
    })
}

fn get_instant_for_timestamp(timestamp: u64) -> Instant {
    let now_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64;
    if timestamp < now_timestamp {
        Instant::now()
            .checked_sub(Duration::from_nanos(now_timestamp - timestamp))
            .expect("Transfers were enabled before 1970!")
    } else {
        Instant::now().add(Duration::from_nanos(timestamp - now_timestamp))
    }
}
