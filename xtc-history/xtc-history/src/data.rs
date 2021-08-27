use crate::backend::Backend;
use ic_cdk::export::candid::{CandidType, Principal};
use serde::Deserialize;
use xtc_history_common::bucket::*;
use xtc_history_common::types::*;

/// All of the data inside the main canister's history. This structure combines a bucket to manage
/// the events living in the main canister with a mapping of all the buckets created to provide
/// fast lookups to any transaction in existence and keep the state of bucket in sync with the
/// list of buckets, so the system is in a valid state at anytime.
pub struct HistoryData<Address = Principal> {
    bucket: BucketData<Address, Transaction>,
    buckets: Vec<(TransactionId, Address)>,
}

/// A borrow of all the data required to reconstruct the HistoryData efficient to be serialized.
#[derive(CandidType)]
pub struct HistoryArchiveBorrowed<'e, 'b, Address: 'b = Principal> {
    offset: TransactionId,
    events: &'e Vec<Transaction>,
    buckets: &'b Vec<(TransactionId, Address)>,
}

/// The result of deserializing a HistoryArchiveBorrowed.
#[derive(CandidType, Deserialize)]
pub struct HistoryArchive<Address = Principal> {
    offset: TransactionId,
    events: Vec<Transaction>,
    buckets: Vec<(TransactionId, Address)>,
}

impl<T> Default for HistoryData<T> {
    fn default() -> Self {
        let mut bucket = BucketData::default();
        bucket.set_metadata(SetBucketMetadataArgs {
            from: 0,
            next: None,
        });

        HistoryData {
            bucket,
            buckets: Vec::new(),
        }
    }
}

impl<Address> HistoryData<Address> {
    /// Push an event to the history buffer and return the transaction id for that event.
    #[inline]
    pub fn push(&mut self, event: Transaction) -> TransactionId {
        self.bucket.push(event)
    }

    /// Insert a new bucket at the current offset.
    #[inline]
    pub fn insert_bucket(&mut self, address: Address)
    where
        Address: Clone,
    {
        self.buckets
            .push((self.bucket.get_offset(), address.clone()));
        self.bucket.update_next(Some(address));
    }

    /// Remove the first n items from the main canister's buffer.
    #[inline]
    pub fn remove_first(&mut self, n: usize) {
        self.bucket.remove_first(n)
    }

    /// Return true if the history is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty() && self.bucket.get_events().is_empty()
    }

    /// Return the events in the local canister.
    #[inline]
    pub fn get_events(&self) -> &Vec<Transaction> {
        self.bucket.get_events()
    }

    /// Return the current active bucket canister.
    ///
    /// # Panics
    /// If there is no bucket created yet.
    #[inline]
    pub fn get_bucket(&self) -> &Address {
        self.bucket.get_next().unwrap()
    }

    /// Return the bucket that should contain the given transaction.
    #[inline]
    pub fn get_bucket_for(&self, id: TransactionId) -> Option<&Address> {
        if self.buckets.is_empty() {
            return None;
        }

        let index = match self.buckets.binary_search_by(|(x, _)| x.cmp(&id)) {
            Ok(index) => index,
            Err(index) => index - 1,
        };

        Some(&self.buckets[index].1)
    }

    /// Return the total number of elements in the history.
    #[inline]
    pub fn size(&self) -> u64 {
        self.bucket.get_offset() + self.bucket.len() as u64
    }

    /// Return the number of items in the local buffer, use `size` if you want to get the
    /// number of entire records inserted into the history.
    #[inline]
    pub fn len(&self) -> usize {
        self.bucket.len()
    }

    /// Return the SetBucketMetadataArgs for the current state of the data.
    #[inline]
    pub fn get_metadata(&self) -> SetBucketMetadataArgs<Address>
    where
        Address: Clone,
    {
        SetBucketMetadataArgs {
            from: self.bucket.get_offset(),
            next: self.bucket.get_next().cloned(),
        }
    }

    /// Return false if there is no bucket created yet.
    #[inline]
    pub fn bucket_exists(&self) -> bool {
        self.buckets.len() > 0
    }

    /// Return the transaction with the given id using the provided backend storage as type.
    #[inline]
    pub async fn get_transaction<S>(&self, id: TransactionId) -> Option<Transaction>
    where
        S: Backend<Address>,
    {
        if id >= self.bucket.get_offset() {
            self.bucket.get_transaction(id).cloned()
        } else if let Some(canister_id) = self.get_bucket_for(id) {
            S::lookup_transaction(canister_id, id).await.unwrap()
        } else {
            None
        }
    }

    /// Return a page from the events.
    #[inline]
    pub fn events<S>(&self, offset: Option<u64>, limit: usize) -> EventsConnection<Address>
    where
        S: Backend<Address>,
        Address: Clone,
    {
        let offset = offset.unwrap_or(self.size());
        if offset >= self.bucket.get_offset() {
            self.bucket.events(Some(offset), limit, || S::id())
        } else {
            EventsConnection {
                data: Vec::new(),
                next_offset: offset,
                next_canister_id: self.get_bucket_for(offset).cloned(),
            }
        }
    }
}

impl<Address> HistoryData<Address> {
    /// Create an archive for the current data.
    pub fn archive(&self) -> HistoryArchiveBorrowed<Address> {
        HistoryArchiveBorrowed {
            offset: self.bucket.get_offset(),
            events: self.bucket.get_events(),
            buckets: &self.buckets,
        }
    }

    /// Load the data from an archive.
    ///
    /// # Panics
    /// If the history is not currently empty.
    pub fn load(&mut self, archive: HistoryArchive<Address>)
    where
        Address: Clone,
    {
        assert!(
            self.is_empty(),
            "Cannot load data when current buffer is not empty."
        );

        let next = if archive.buckets.is_empty() {
            None
        } else {
            Some(archive.buckets[archive.buckets.len() - 1].1.clone())
        };

        self.bucket = BucketData::new(archive.offset, archive.events);
        self.bucket.update_next(next);
        self.buckets = archive.buckets;
    }

    /// Load the data from an archive generated by previous versions of XTC.
    ///
    /// # Panics
    /// If the history is not currently empty.
    pub fn load_v0(&mut self, mut data: Vec<Transaction>) {
        assert!(
            self.is_empty(),
            "Cannot load data when current buffer is not empty."
        );

        self.bucket.append(&mut data);
    }
}
