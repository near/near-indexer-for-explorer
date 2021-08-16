use actix::Addr;

use near_client::{Query, ViewClientActor};
use near_indexer::near_primitives;

pub(crate) async fn get_account_balance(
    view_client: &Addr<ViewClientActor>,
    account_id: &near_primitives::types::AccountId,
    block_height: &near_primitives::types::BlockHeight,
) -> Result<near_primitives::types::Balance, String> {
    get_account_view_for_block_height(view_client, account_id, block_height)
        .await
        .map(|account| account.amount)
        .map_err(|err| format!("Unable to get account balance: {}", err))
}

pub(crate) async fn get_contract_code_hash(
    view_client: &Addr<ViewClientActor>,
    account_id: &near_primitives::types::AccountId,
    block_height: &near_primitives::types::BlockHeight,
) -> Result<near_primitives::hash::CryptoHash, String> {
    get_account_view_for_block_height(view_client, account_id, block_height)
        .await
        .map(|account| account.code_hash)
        .map_err(|err| format!("Unable to get contract code hash: {}", err))
}

async fn get_account_view_for_block_height(
    view_client: &Addr<ViewClientActor>,
    account_id: &near_primitives::types::AccountId,
    block_height: &near_primitives::types::BlockHeight,
) -> Result<near_primitives::views::AccountView, String> {
    let block_reference = near_primitives::types::BlockReference::BlockId(
        near_primitives::types::BlockId::Height(*block_height),
    );
    let request = near_primitives::views::QueryRequest::ViewAccount {
        account_id: account_id.clone(),
    };
    let query = Query::new(block_reference, request);

    let account_response = view_client
        .send(query)
        .await
        .map_err(|err| {
            format!(
                "Failed to deliver ViewAccount for account {}, block {}: {}",
                account_id, block_height, err
            )
        })?
        .map_err(|err| {
            format!(
                "Invalid ViewAccount query for account {}, block {}: {:?}",
                account_id, block_height, err
            )
        })?;

    match account_response.kind {
        near_primitives::views::QueryResponseKind::ViewAccount(account) => Ok(account),
        _ => Err(format!(
            "Failed to extract ViewAccount response for account {}, block {}",
            account_id, block_height
        )),
    }
}
