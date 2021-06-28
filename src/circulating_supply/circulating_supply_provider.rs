use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::PgConnection;

use crate::circulating_supply::lockup::{
    get_account_state, get_code_version, is_bug_inside, TRANSFERS_ENABLED,
};
use crate::circulating_supply::user_balance::get_user_balance;
use crate::db_adapters::accounts::collect_lockups_for_block;
use crate::db_adapters::blocks::closest_block_for;
use crate::db_adapters::circulating_supply::{
    add_circulating_supply, get_precomputed_circulating_supply,
};
use crate::models;
use crate::models::circulating_supply::CirculatingSupply;
use crate::models::Block;
use actix::Addr;
use actix_web::rt::time;
use std::ops::Div;
use std::str::FromStr;
use std::time::SystemTime;
use tracing::info;

const DAY: std::time::Duration = std::time::Duration::from_secs(86400);

pub(crate) async fn compute_circulating_supply(view_client: Addr<near_client::ViewClientActor>) {
    let mut current_day = TRANSFERS_ENABLED;
    while current_day < get_now_timestamp() {
        compute_circulating_supply_for(&view_client, current_day).await;
        current_day += DAY.as_nanos() as u64;
    }

    // We ping the number twice a day to be sure that we computed it, and to deliver it earlier.
    // If the value is already in the DB, we will just return almost immediately
    let mut interval = time::interval(DAY.div(2));
    loop {
        compute_circulating_supply_for(&view_client, get_now_timestamp()).await;
        interval.tick().await;
    }
}

fn get_now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

async fn compute_circulating_supply_for(
    view_client: &Addr<near_client::ViewClientActor>,
    request_timestamp: u64,
) {
    let pool = models::establish_connection();

    let block = find_last_yesterday_block_for(request_timestamp, &pool).await;
    let block_timestamp = block
        .block_timestamp
        .to_u64()
        .expect("`block_timestamp` expected to be u64");
    let block_height = block
        .block_height
        .to_u64()
        .expect("`block_height` expected to be u64");
    let total_supply = u128::from_str_radix(&*block.total_supply.to_string(), 10)
        .expect("`total_supply` expected to be u128");

    match get_precomputed_circulating_supply(block_timestamp, &pool).await {
        Ok(None) => {
            info!(
                target: crate::INDEXER_FOR_EXPLORER,
                "Computing circulating supply for the timestamp {}", block_timestamp
            );
        }
        Ok(Some(supply)) => {
            info!(
                target: crate::INDEXER_FOR_EXPLORER,
                "Circulating supply for the timestamp {} was already computed: {}",
                block_timestamp,
                supply
            );
            return;
        }
        Err(err) => {
            panic!("Error {}", err);
        }
    }

    let lockups = collect_lockups_for_block(block_height, &pool).await;

    let mut total_locked_amount: u128 = 0;
    let mut active_locked_accounts: u128 = 0;

    for lockup in lockups.iter() {
        let state = get_account_state(&view_client, lockup, block_height).await;
        let is_lockup_with_bug = is_bug_inside(
            &get_code_version(&view_client, lockup, block_height).await,
            lockup,
        );
        let locked_amount = state
            .get_locked_amount(block_timestamp, is_lockup_with_bug)
            .0;
        total_locked_amount += locked_amount;
        if locked_amount > 0 {
            active_locked_accounts += 1;
        }
    }

    let foundation_locked_accs = ["lockup.near", "contributors.near"];
    let mut foundation_locked: u128 = 0;
    for acc in foundation_locked_accs.iter() {
        foundation_locked += get_user_balance(view_client.clone(), acc, block_height).await;
    }

    let circulating_supply: u128 = total_supply - foundation_locked - total_locked_amount;

    let stats = CirculatingSupply {
        block_timestamp: block.block_timestamp,
        block_hash: block.block_hash,
        value: BigDecimal::from_str(circulating_supply.to_string().as_str())
            .expect("`value` expected to be u128"),
        total_supply: BigDecimal::from_str(total_supply.to_string().as_str())
            .expect("`total_supply` expected to be u128"),
        lockups_number: BigDecimal::from_str(lockups.len().to_string().as_str())
            .expect("`lockups_number` expected to be u128"),
        active_lockups_number: BigDecimal::from_str(active_locked_accounts.to_string().as_str())
            .expect("`active_lockups_number` expected to be u128"),
        foundation_locked_supply: BigDecimal::from_str(foundation_locked.to_string().as_str())
            .expect("`foundation_locked_supply` expected to be u128"),
        lockups_locked_supply: BigDecimal::from_str(total_locked_amount.to_string().as_str())
            .expect("`lockups_locked_supply` expected to be u128"),
    };

    add_circulating_supply(&pool, &stats).await;
    info!(
        target: crate::INDEXER_FOR_EXPLORER,
        "Circulating supply for the timestamp {} is {}", block_timestamp, circulating_supply
    );
}

async fn find_last_yesterday_block_for(
    current_timestamp: u64,
    pool: &actix_diesel::Database<PgConnection>,
) -> Block {
    let day_length = 86400000000000;
    let start_of_day = current_timestamp - (current_timestamp % day_length);
    let block = closest_block_for(pool, start_of_day).await;
    return block.expect(&format!(
        "Unable to find the block before {}",
        current_timestamp
    ));
}
