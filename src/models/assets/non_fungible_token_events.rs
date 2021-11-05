use bigdecimal::BigDecimal;

use crate::models::enums::NftEventKind;
use crate::schema;
use schema::assets__non_fungible_token_events;

#[derive(Insertable, Queryable, Clone, Debug)]
#[table_name = "assets__non_fungible_token_events"]
pub struct NonFungibleTokenEvent {
    pub emitted_for_receipt_id: String,
    pub emitted_at_block_timestamp: BigDecimal,
    pub emitted_in_shard_id: BigDecimal,
    pub emitted_index_of_event_entry_in_shard: i32,
    pub emitted_by_contract_account_id: String,
    pub token_id: String,
    pub event_kind: NftEventKind,
    pub token_old_owner_account_id: String,
    pub token_new_owner_account_id: String,
    pub token_authorized_account_id: String,
    pub event_memo: String,
}
