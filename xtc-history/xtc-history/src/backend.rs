use std::future::Future;
use std::pin::Pin;
use xtc_history_common::types::*;

/// Type alias for the data types returned from the async methods.
pub type Res<O> = Pin<Box<dyn Future<Output = Result<O, String>>>>;

pub trait Backend<Address> {
    /// Create a canister to be later used an archive.
    fn create_canister() -> Res<Address>;

    /// Install the WASM binary for the given canister id.
    fn install_code(canister_id: &Address) -> Res<()>;

    /// Perform the set_metadata call on the canister to update the metadata for this canister,
    /// the metadata includes the previous archive canister and the start offset for this canister.
    fn write_metadata(canister_id: &Address, data: SetBucketMetadataArgs<Address>) -> Res<()>;

    /// Write a batch of transactions into the given canister, the data should be sorted from
    /// older to newer.
    fn append_transactions(canister_id: &Address, data: &[Transaction]) -> Res<()>;

    /// Try to retrieve the given transaction id from the given bucket canister, the bucket
    /// should contain the transaction id, otherwise it returns an Err.
    fn lookup_transaction(canister_id: &Address, id: TransactionId) -> Res<Option<Transaction>>;

    /// Return the id of the current canister.
    fn id() -> Address;
}
