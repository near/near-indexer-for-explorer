use actix::Addr;

use near_client::{Query, ViewClientActor};
use near_indexer::near_primitives::hash::CryptoHash;
use near_indexer::near_primitives::types::{Balance, BlockId, BlockReference};
use near_indexer::near_primitives::views::{AccountView, QueryRequest, QueryResponseKind};

pub(crate) async fn get_account_balance(
    view_client: &Addr<ViewClientActor>,
    account_id: &str,
    block_height: u64,
) -> Result<Balance, String> {
    get_account_view_for_block_height(view_client, account_id, block_height)
        .await
        .map(|acc| acc.amount)
}

pub(crate) async fn get_contract_code_hash(
    view_client: &Addr<ViewClientActor>,
    account_id: &str,
    block_height: u64,
) -> Result<CryptoHash, String> {
    get_account_view_for_block_height(view_client, account_id, block_height)
        .await
        .map(|acc| acc.code_hash)
}

async fn get_account_view_for_block_height(
    view_client: &Addr<ViewClientActor>,
    account_id: &str,
    block_height: u64,
) -> Result<AccountView, String> {
    let block_reference = BlockReference::BlockId(BlockId::Height(block_height));
    let request = QueryRequest::ViewAccount {
        account_id: account_id
            .parse()
            .map_err(|_| "Failed to parse `account_id`")?,
    };
    let query = Query::new(block_reference, request);

    let account_response = view_client
        .send(query)
        .await
        .map_err(|err| {
            format!(
                "Error while delivering ViewAccount for account {}, block {}: {}",
                account_id, block_height, err
            )
        })?
        .map_err(|_| {
            format!(
                "Invalid ViewAccount query for account {}, block {}",
                account_id, block_height
            )
        })?;

    match account_response.kind {
        QueryResponseKind::ViewAccount(account) => Ok(account),
        _ => Err(format!(
            "ViewAccount result expected for {}, block {}",
            account_id, block_height
        )),
    }
}
