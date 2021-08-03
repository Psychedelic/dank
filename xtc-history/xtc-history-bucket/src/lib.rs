use ic_cdk::export::Principal;
use ic_cdk::*;
use ic_cdk_macros::*;
use xtc_history_types::*;

struct BucketData {
    /// The events in this bucket, smaller index means older data.
    events: Vec<Transaction>,
    /// The controller of this bucket canister, which is the `XTC` canister id.
    controller: Option<Principal>,
    /// Actual ID of the first event in the history.
    from: Option<TransactionId>,
    /// The next bucket canister.
    next: Option<Principal>,
}

impl Default for BucketData {
    fn default() -> Self {
        BucketData {
            events: Vec::with_capacity(100000),
            controller: None,
            from: None,
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
        let from = self.from.unwrap();

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
    pub fn events(&self, from: u64, limit: u16) -> EventsConnection {
        let index = self.get_index(from);
        let end = (index + limit as usize).min(self.events.len());

        let next_canister_id = if end == self.events.len() {
            self.next
        } else {
            Some(id())
        };

        EventsConnection {
            data: &self.events[index..end],
            next_canister_id,
        }
    }
}

#[init]
fn init() {
    let data = storage::get_mut::<BucketData>();
    data.controller = Some(caller());
}

#[query]
fn metadata() -> BucketMetadata {
    let data = storage::get::<BucketData>();
    BucketMetadata {
        version: 0,
        size: data.events.len(),
        from: data.from.unwrap(),
        next: data.next,
    }
}

#[update]
fn set_metadata(meta: SetBucketMetadataArgs) {
    let data = storage::get_mut::<BucketData>();

    if caller() != data.controller.unwrap() {
        trap("Only the controller is allowed to call set_metadata.");
    }

    data.from = Some(meta.from);
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
    let from = data.from.unwrap();
    data.events(args.from.unwrap_or(from), args.limit)
}
