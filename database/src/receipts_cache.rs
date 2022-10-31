use cached::SizedCache;
use tokio::sync::Mutex;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum ReceiptOrDataId {
    ReceiptId(near_lake_framework::near_indexer_primitives::CryptoHash),
    DataId(near_lake_framework::near_indexer_primitives::CryptoHash),
}
// Creating type aliases to make HashMap types for cache more explicit
pub type ParentTransactionHashString = String;
// Introducing a simple cache for Receipts to find their parent Transactions without
// touching the database
// The key is ReceiptID
// The value is TransactionHash (the very parent of the Receipt)
pub type ReceiptsCache =
    std::sync::Arc<Mutex<SizedCache<ReceiptOrDataId, ParentTransactionHashString>>>;
