use actix::Addr;
use near_client::{Query, ViewClientActor};
use near_indexer::near_primitives::types::{Balance, BlockId, BlockReference};
use near_indexer::near_primitives::views::{QueryRequest, QueryResponseKind};

pub async fn get_user_balance(
    view_client: Addr<ViewClientActor>,
    account_id: &str,
    block_height: u64,
) -> Balance {
    let block_reference = BlockReference::BlockId(BlockId::Height(block_height));
    let request = QueryRequest::ViewAccount {
        account_id: account_id.parse().unwrap(),
    };
    let query = Query::new(block_reference, request);

    let wrapped_response = view_client.send(query).await;
    let account_response = wrapped_response
        .expect(&format!(
            "Error while delivering account {}, block {}",
            account_id, block_height
        ))
        .expect(&format!(
            "Invalid query: account {}, block {}",
            account_id, block_height
        ));
    let account = match account_response.kind {
        QueryResponseKind::ViewAccount(x) => x,
        _ => {
            panic!(
                "ViewAccount result expected for {}, block {}",
                account_id, block_height
            )
        }
    };
    return account.amount;
}
