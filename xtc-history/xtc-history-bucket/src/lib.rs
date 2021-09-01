use ic_cdk::export::candid::{CandidType, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::Deserialize;
use xtc_history_common::bucket::*;
use xtc_history_common::types::*;

pub struct Data {
    bucket: BucketData,
    controller: Option<Principal>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            bucket: BucketData::default(),
            controller: None,
        }
    }
}

#[init]
fn init() {
    let data = storage::get_mut::<Data>();
    data.controller = Some(caller());
    // For each transaction reserve 2.5kb for heap allocated data, the size of Transaction is
    // just the size needed to store the pointers. And is about 120 bytes.
    let size = std::mem::size_of::<Transaction>() + 2560;
    // 3.5 GB / ~2.5KB ~= 1.4M
    let num = (3584 * 1024 * 1024) / size;
    data.bucket.reserve(num);
}

#[derive(Deserialize, CandidType)]
pub struct BucketMetadata {
    pub version: u64,
    pub size: usize,
    pub offset: TransactionId,
    pub next: Option<Principal>,
}

#[query]
fn metadata() -> BucketMetadata {
    let data = storage::get::<Data>();
    BucketMetadata {
        version: 0,
        size: data.bucket.len(),
        offset: data.bucket.get_offset(),
        next: data.bucket.get_next().cloned(),
    }
}

#[update]
fn set_metadata(meta: SetBucketMetadataArgs) {
    let data = storage::get_mut::<Data>();

    if caller() != data.controller.unwrap() {
        trap("Only the controller is allowed to call set_metadata.");
    }

    data.bucket.set_metadata(meta);
}

#[update]
fn append(mut events: Vec<Transaction>) {
    let data = storage::get_mut::<Data>();
    if caller() != data.controller.unwrap() {
        trap("Only the controller is allowed to call push.");
    }
    data.bucket.append(&mut events);
}

#[query]
fn get_transaction(id: TransactionId) -> Option<&'static Transaction> {
    storage::get::<Data>().bucket.get_transaction(id)
}

#[query]
fn events(args: EventsArgs) -> EventsConnection<'static> {
    storage::get::<Data>()
        .bucket
        .events(args.offset, args.limit as usize, || id())
}
