use crate::types::*;
use ic_cdk::export::candid::Principal;

/// A single knot in the history chain. This structure is responsible for storing a list of
/// events that start from a constant index called the bucket's offset, and provide API to
/// get views over the bucket's data using global indexes.
/// It also stores the link to the next bucket on the chain as part of its metadata, and
/// uses it to direct callers to the next bucket when paginating through the events and hit
/// the end of the buffer.
/// It is used to both store the events on the main canister as well as the archive canisters.
pub struct BucketData<Address = Principal, Event = Transaction> {
    /// The events in this bucket, smaller index means older data.
    events: Vec<Event>,
    /// The metadata for this bucket.
    metadata: Option<BucketMetadata<Address>>,
}

pub struct BucketMetadata<Address> {
    /// Global offset of this bucket. In other terms this is the transaction id of the first
    /// event in this bucket.
    pub offset: TransactionId,
    /// The previous bucket on the chain.
    pub next: Option<Address>,
}

impl<Address, Event> Default for BucketData<Address, Event> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            metadata: None,
        }
    }
}

impl<Address, Event> BucketData<Address, Event> {
    /// Create a new bucket with the given data.
    pub fn new(offset: TransactionId, events: Vec<Event>) -> Self {
        BucketData {
            events,
            metadata: Some(BucketMetadata { offset, next: None }),
        }
    }

    /// Pre-reserve space for the given number of transactions.
    #[inline]
    pub fn reserve(&mut self, capacity: usize) {
        self.events.reserve(capacity);
    }

    /// Set the metadata for this bucket.
    /// # Panics
    /// If the metadata is already set.
    #[inline]
    pub fn set_metadata(&mut self, data: SetBucketMetadataArgs<Address>) {
        assert!(self.metadata.is_none(), "The metadata is already set.");
        self.metadata = Some(BucketMetadata {
            offset: data.from,
            next: data.next,
        });
    }

    /// Get the offset of this bucket, it is the transaction id of the oldest event in this
    /// buffer.
    #[inline]
    pub fn get_offset(&self) -> TransactionId {
        self.metadata
            .as_ref()
            .expect("Metadata is not set yet.")
            .offset
    }

    /// Get the offset of this bucket, it is the transaction id of the oldest event in this
    /// buffer.
    #[inline]
    pub fn get_next(&self) -> Option<&Address> {
        self.metadata
            .as_ref()
            .expect("Metadata is not set yet.")
            .next
            .as_ref()
    }

    /// Return the given transaction from this bucket, None is returned when the transaction
    /// is not found in this bucket.
    #[inline]
    pub fn get_transaction(&self, id: TransactionId) -> Option<&Event> {
        let index = match id.checked_sub(self.get_offset()) {
            Some(index) => index,
            None => return None,
        } as usize;

        self.events.get(index)
    }

    /// Read a page of data from this bucket, the returned data is sorted from newest to oldest.
    /// This method starts collecting events from the given offset until oldest data until there
    /// is no more data or the limit is exceeded.
    ///
    /// The data returned does not include the transaction id of the given offset, but rather
    /// anything older than that.
    ///
    /// If the offset is not provided the newest data is returned.
    ///
    /// # Panics
    ///
    /// If the provided offset is smaller than the bucket's offset.
    pub fn events<F: FnOnce() -> Address>(
        &self,
        offset: Option<TransactionId>,
        limit: usize,
        get_id: F,
    ) -> EventsConnection<Address, Event>
    where
        Address: Clone,
    {
        let bucket_offset = self.get_offset();
        let max = bucket_offset + self.events.len() as u64;
        let offset = offset.unwrap_or(max);

        let (offset, limit) = if offset > max {
            let d = (offset - max) as usize;
            (max, limit.checked_sub(d).unwrap_or(0))
        } else {
            (offset, limit)
        };

        // 0 1 2 3 4 5 6 7 8 9
        // events(6, 3) -> {5, 4, 3}
        // end   = 6 - 0  = 6
        // start = end - limit = 6 - 3 = 3
        // next offset = 3

        let take = limit + 1;
        let end = (offset - bucket_offset) as usize;
        let start = end.checked_sub(take).unwrap_or(0);
        let mut data: &[Event] = &self.events[start..end];

        let has_more = if data.len() > limit {
            data = &data[1..];
            true
        } else {
            false
        };

        let (next_canister_id, next_offset) = if has_more {
            let next_offset = bucket_offset + start as u64 + 1;
            (Some(get_id()), next_offset)
        } else if let Some(address) = &self.metadata.as_ref().unwrap().next {
            (Some(address.clone()), bucket_offset)
        } else {
            (None, 0)
        };

        EventsConnection {
            data: data.into_iter().rev().collect(),
            next_offset,
            next_canister_id,
        }
    }

    /// Append a vector of events to this bucket, this vector should be sorted from oldest to
    /// newest.
    #[inline]
    pub fn append(&mut self, other: &mut Vec<Event>) {
        self.events.append(other);
    }

    /// Push a single event to this bucket, returns the global id of it.
    #[inline]
    pub fn push(&mut self, event: Event) -> TransactionId {
        let len = self.events.len() as u64;
        self.events.push(event);
        self.get_offset() + len
    }

    /// Return the number of events in this bucket.
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Update the id of the next canister.
    ///
    /// # Panics
    /// If the metadata is not set yet.
    #[inline]
    pub fn update_next(&mut self, next: Option<Address>) {
        let metadata = self.metadata.as_mut().unwrap();
        metadata.next = next;
    }

    /// Remove the first n item from this bucket and move the offset forward.
    #[inline]
    pub fn remove_first(&mut self, mut n: usize) {
        n = n.min(self.events.len());
        let metadata = self.metadata.as_mut().unwrap();
        metadata.offset += n as u64;
        self.events.drain(0..n);
    }

    /// Return the events in this bucket.
    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_transaction_from_offset_zero() {
        let bucket = BucketData::<u32, u32>::new(0, vec![0, 1, 2, 3]);
        assert_eq!(bucket.get_transaction(0), Some(&0));
        assert_eq!(bucket.get_transaction(1), Some(&1));
        assert_eq!(bucket.get_transaction(2), Some(&2));
        assert_eq!(bucket.get_transaction(3), Some(&3));
        assert_eq!(bucket.get_transaction(4), None);
    }

    #[test]
    fn get_transaction() {
        let bucket = BucketData::<u32, u32>::new(1, vec![1, 2, 3]);
        assert_eq!(bucket.get_transaction(0), None);
        assert_eq!(bucket.get_transaction(1), Some(&1));
        assert_eq!(bucket.get_transaction(2), Some(&2));
        assert_eq!(bucket.get_transaction(3), Some(&3));
        assert_eq!(bucket.get_transaction(4), None);
    }

    #[test]
    fn events_from_offset_zero() {
        let events = (0..=10).into_iter().collect();
        let bucket = BucketData::<u32, u32>::new(0, events);

        let res = bucket.events(None, 3, || 17);
        assert_eq!(res.data, vec![&10, &9, &8]);
        assert_eq!(res.next_offset, 8);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(11), 3, || 17);
        assert_eq!(res.data, vec![&10, &9, &8]);
        assert_eq!(res.next_offset, 8);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(res.next_offset), 3, || 17);
        assert_eq!(res.data, vec![&7, &6, &5]);
        assert_eq!(res.next_offset, 5);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(res.next_offset), 3, || 17);
        assert_eq!(res.data, vec![&4, &3, &2]);
        assert_eq!(res.next_offset, 2);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(res.next_offset), 3, || 17);
        assert_eq!(res.data, vec![&1, &0]);
        assert_eq!(res.next_offset, 0);
        assert_eq!(res.next_canister_id, None);

        let res = bucket.events(Some(1), 3, || 17);
        assert_eq!(res.data, vec![&0]);
        assert_eq!(res.next_offset, 0);
        assert_eq!(res.next_canister_id, None);

        let res = bucket.events(Some(0), 3, || 17);
        assert_eq!(res.data, Vec::<&u32>::new());
        assert_eq!(res.next_offset, 0);
        assert_eq!(res.next_canister_id, None);
    }

    #[test]
    fn events() {
        let events = (11..=20).into_iter().collect();
        let mut bucket = BucketData::<u32, u32>::new(11, events);
        bucket.update_next(Some(16));

        assert_eq!(bucket.get_transaction(11), Some(&11));

        let res = bucket.events(None, 3, || 17);
        assert_eq!(res.data, vec![&20, &19, &18]);
        assert_eq!(res.next_offset, 18);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(res.next_offset), 3, || 17);
        assert_eq!(res.data, vec![&17, &16, &15]);
        assert_eq!(res.next_offset, 15);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(res.next_offset), 3, || 17);
        assert_eq!(res.data, vec![&14, &13, &12]);
        assert_eq!(res.next_offset, 12);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(res.next_offset), 3, || 17);
        assert_eq!(res.data, vec![&11]);
        assert_eq!(res.next_offset, 11);
        assert_eq!(res.next_canister_id, Some(16));

        let res = bucket.events(Some(13), 3, || 17);
        assert_eq!(res.data, vec![&12, &11]);
        assert_eq!(res.next_offset, 11);
        assert_eq!(res.next_canister_id, Some(16));

        // Test next bucket.

        let events = (0..11).into_iter().collect();
        let bucket = BucketData::<u32, u32>::new(0, events);

        let res = bucket.events(Some(11), 3, || 16);
        assert_eq!(res.data, vec![&10, &9, &8]);
        assert_eq!(res.next_offset, 8);
        assert_eq!(res.next_canister_id, Some(16));
    }

    #[test]
    fn push() {
        let mut bucket = BucketData::<u32, u32>::new(0, vec![]);
        assert_eq!(bucket.push(0), 0);

        let events = (11..=20).into_iter().collect();
        let mut bucket = BucketData::<u32, u32>::new(11, events);
        assert_eq!(bucket.push(21), 21);
        assert_eq!(bucket.push(22), 22);
    }

    #[test]
    fn remove_first() {
        let events = (0..=20).into_iter().collect();
        let mut bucket = BucketData::<u32, u32>::new(0, events);
        assert_eq!(bucket.get_transaction(0), Some(&0));
        assert_eq!(bucket.get_transaction(1), Some(&1));
        bucket.remove_first(5);
        assert_eq!(bucket.get_transaction(0), None);
        assert_eq!(bucket.get_transaction(1), None);
        assert_eq!(bucket.get_transaction(4), None);
        assert_eq!(bucket.get_transaction(5), Some(&5));
        assert_eq!(bucket.get_transaction(6), Some(&6));
        bucket.remove_first(5);
        assert_eq!(bucket.get_transaction(0), None);
        assert_eq!(bucket.get_transaction(1), None);
        assert_eq!(bucket.get_transaction(4), None);
        assert_eq!(bucket.get_transaction(5), None);
        assert_eq!(bucket.get_transaction(6), None);
        assert_eq!(bucket.get_transaction(10), Some(&10));
    }

    #[test]
    fn events_large_offset() {
        let events = (0..5).into_iter().collect();
        let bucket = BucketData::<u32, u32>::new(0, events);

        let res = bucket.events(Some(5), 3, || 17);
        assert_eq!(res.data, vec![&4, &3, &2]);
        assert_eq!(res.next_offset, 2);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(6), 3, || 17);
        assert_eq!(res.data, vec![&4, &3]);
        assert_eq!(res.next_offset, 3);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(7), 3, || 17);
        assert_eq!(res.data, vec![&4]);
        assert_eq!(res.next_offset, 4);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(8), 3, || 17);
        assert_eq!(res.data.len(), 0);
        assert_eq!(res.next_offset, 5);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(9), 3, || 17);
        assert_eq!(res.data.len(), 0);
        assert_eq!(res.next_offset, 5);
        assert_eq!(res.next_canister_id, Some(17));

        let res = bucket.events(Some(10), 3, || 17);
        assert_eq!(res.data.len(), 0);
        assert_eq!(res.next_offset, 5);
        assert_eq!(res.next_canister_id, Some(17));
    }
}
