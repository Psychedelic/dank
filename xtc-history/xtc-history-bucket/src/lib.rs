use ic_cdk::export::candid::{CandidType, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::Deserialize;
use xtc_history_types::*;

pub struct BucketData {
    /// The events in this bucket, smaller index means older data.
    events: Vec<Transaction>,
    /// The controller of this bucket canister, which is the `XTC` canister id.
    controller: Option<Principal>,
    /// Actual ID of the first event in the history.
    offset: Option<TransactionId>,
    /// The next bucket canister.
    next: Option<Principal>,
}

impl Default for BucketData {
    fn default() -> Self {
        BucketData {
            events: Vec::with_capacity(100000),
            controller: None,
            offset: None,
            next: None,
        }
    }
}

impl BucketData {
    #[inline]
    pub fn push(&mut self, mut events: Vec<Transaction>) {
        self.events.append(&mut events);
    }

    #[inline]
    fn get_index(&self, id: TransactionId) -> usize {
        let from = self.offset.unwrap();

        if id < from {
            trap("Transaction is in older buckets.");
        }

        let index = (id - from) as usize;

        if index >= self.events.len() {
            trap("Transaction is in newer buckets.");
        }

        index
    }

    #[inline]
    pub fn get_transaction(&self, id: TransactionId) -> Option<&Transaction> {
        let index = self.get_index(id);
        Some(&self.events[index])
    }

    #[inline]
    pub fn events(&self, offset: Option<u64>, limit: u16) -> EventsConnection {
        let end_offset = (self.offset.unwrap() + self.events.len() as u64)
            .checked_sub(1)
            .unwrap_or(0);
        let offset = offset.unwrap_or(end_offset);
        let take = limit as usize + 1;
        let e = self.get_index(offset);
        let s = e.checked_sub(take).unwrap_or(0);

        let mut data = &self.events[s..e];
        let next_canister_id = if data.len() > limit as usize {
            data = &data[1..];
            Some(id())
        } else {
            self.next.clone()
        };

        EventsConnection {
            data: data.into_iter().rev().collect(),
            next_offset: offset - data.len() as u64,
            next_canister_id,
        }
    }
}

#[init]
fn init() {
    let data = storage::get_mut::<BucketData>();
    data.controller = Some(caller());
}

#[derive(Deserialize, CandidType)]
pub struct BucketMetadata {
    pub version: u64,
    pub size: usize,
    pub from: TransactionId,
    pub next: Option<Principal>,
}

#[query]
fn metadata() -> BucketMetadata {
    let data = storage::get::<BucketData>();
    BucketMetadata {
        version: 0,
        size: data.events.len(),
        from: data.offset.unwrap(),
        next: data.next,
    }
}

#[update]
fn set_metadata(meta: SetBucketMetadataArgs) {
    let data = storage::get_mut::<BucketData>();

    if caller() != data.controller.unwrap() {
        trap("Only the controller is allowed to call set_metadata.");
    }

    data.offset = Some(meta.from);
    data.next = meta.next;
}

#[update]
fn push(events: Vec<Transaction>) {
    let data = storage::get_mut::<BucketData>();
    if caller() != data.controller.unwrap() {
        trap("Only the controller is allowed to call set_metadata.");
    }
    data.push(events);
}

#[query]
fn get_transaction(id: TransactionId) -> Option<&'static Transaction> {
    storage::get::<BucketData>().get_transaction(id)
}

#[query]
fn events(args: EventsArgs) -> EventsConnection<'static> {
    let data = storage::get::<BucketData>();
    data.events(args.offset, args.limit)
}
