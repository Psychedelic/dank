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
        let offset = offset.unwrap_or(max).min(max);

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

        EventsConnection {
            data: data.into_iter().rev().collect(),
            next_offset: bucket_offset + start as u64,
            next_canister_id: if has_more {
                Some(get_id())
            } else if let Some(address) = &self.metadata.as_ref().unwrap().next {
                Some(address.clone())
            } else {
                None
            },
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