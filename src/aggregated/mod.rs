use actix_diesel::Database;
use diesel::PgConnection;
use near_indexer::Indexer;

mod account_details;
mod circulating_supply;

pub(crate) fn spawn_aggregated_computations(pool: Database<PgConnection>, indexer: &Indexer) {
    let view_client = indexer.client_actors().0;
    if indexer.near_config().genesis.config.chain_id == "mainnet" {
        actix::spawn(circulating_supply::run_circulating_supply_computation(
            view_client,
            pool,
        ));
    }
}
