use bigdecimal::BigDecimal;

use crate::models::enums::FtEventKind;
use crate::schema;
use schema::assets__fungible_token_events;

#[derive(Insertable, Queryable, Clone, Debug)]
#[table_name = "assets__fungible_token_events"]
pub struct FungibleTokenEvent {
    pub emitted_for_receipt_id: String,
    pub emitted_at_block_timestamp: BigDecimal,
    pub emitted_in_shard_id: BigDecimal,
    pub emitted_index_of_event_entry_in_shard: i32,
    pub emitted_by_contract_account_id: String,
    pub amount: String,
    pub event_kind: FtEventKind,
    pub token_old_owner_account_id: String,
    pub token_new_owner_account_id: String,
    pub event_memo: String,
}
