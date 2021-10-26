use crate::backend::Backend;
use ic_cdk::export::candid::{CandidType, Principal};
use serde::Deserialize;
use std::convert::From;
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
#[derive(CandidType, Debug)]
pub struct HistoryArchiveBorrowed<'e, 'b, Address: 'b = Principal> {
    pub offset: TransactionId,
    pub events: &'e Vec<Transaction>,
    pub buckets: &'b Vec<(TransactionId, Address)>,
}

/// The result of deserializing a HistoryArchiveBorrowed.
#[derive(CandidType, Deserialize)]
pub struct HistoryArchive<Address = Principal> {
    pub offset: TransactionId,
    pub events: Vec<Transaction>,
    pub buckets: Vec<(TransactionId, Address)>,
}

#[derive(CandidType, Deserialize)]
pub struct HistoryArchiveV0<Address = Principal> {
    pub offset: TransactionId,
    pub events: Vec<TransactionV0>,
    pub buckets: Vec<(TransactionId, Address)>,
}

// TODO: ticking time bomb, we need to integrate the history service, as we
// are only converting the transactions in the current bucket
impl From<HistoryArchiveV0> for HistoryArchive {
    fn from(history_archive_v0: HistoryArchiveV0) -> HistoryArchive {
        HistoryArchive {
            offset: history_archive_v0.offset,
            events: history_archive_v0
                .events
                .iter()
                .map(|transaction| transaction.into())
                .collect(),
            buckets: history_archive_v0.buckets,
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::MockBackend;

    /// Generate a fake transaction with the given id, the id is inserted as the timestamp
    /// for the transaction.
    #[inline]
    fn tx(id: u64) -> Transaction {
        Transaction {
            timestamp: id,
            cycles: 0,
            fee: 0,
            kind: TransactionKind::Mint {
                to: Principal::management_canister(),
            },
            status: TransactionStatus::SUCCEEDED,
        }
    }

    #[async_std::test]
    async fn load_v0() {
        let mut data = HistoryData::<u32>::default();
        let transactions = (0..10).map(tx).collect::<Vec<Transaction>>();
        data.load_v0(transactions);
        assert_eq!(data.get_transaction::<MockBackend>(0).await, Some(tx(0)));
        assert_eq!(data.push(tx(10)), 10);
        assert_eq!(data.get_transaction::<MockBackend>(10).await, Some(tx(10)));
    }

    #[async_std::test]
    async fn archive() {
        let mut data = HistoryData::<u32>::default();

        // Insert 10 items.
        (0..10).map(tx).for_each(|event| {
            data.push(event);
        });
        // Create a bucket at this point.
        data.insert_bucket(17);
        // Remove the items.
        data.remove_first(10);

        // Insert the next 10 items.
        (10..20).map(tx).for_each(|event| {
            data.push(event);
        });
        // Create a bucket at this point.
        data.insert_bucket(18);
        // Remove the items.
        data.remove_first(10);

        // Insert the next 10 items.
        (20..30).map(tx).for_each(|event| {
            data.push(event);
        });

        let transactions = (20..30).map(tx).collect::<Vec<Transaction>>();
        let archive = data.archive();
        assert_eq!(archive.offset, 20);
        assert_eq!(archive.events, &transactions);
        assert_eq!(archive.buckets, &vec![(0, 17), (10, 18)]);

        assert_eq!(data.get_transaction::<MockBackend>(25).await, Some(tx(25)));
    }

    #[async_std::test]
    async fn load() {
        let mut data = HistoryData::<u32>::default();
        data.load(HistoryArchive {
            offset: 20,
            buckets: vec![(0, 17), (10, 18)],
            events: (20..30).map(tx).collect(),
        });

        assert_eq!(data.get_transaction::<MockBackend>(25).await, Some(tx(25)));
    }

    #[test]
    fn get_bucket_for() {
        let mut data = HistoryData::<u32>::default();
        data.load(HistoryArchive {
            offset: 30,
            buckets: vec![(0, 17), (10, 18), (20, 19)],
            events: (30..40).map(tx).collect(),
        });

        for i in 0..10 {
            assert_eq!(data.get_bucket_for(i), Some(&17));
        }

        for i in 10..20 {
            assert_eq!(data.get_bucket_for(i), Some(&18));
        }

        for i in 20..30 {
            assert_eq!(data.get_bucket_for(i), Some(&19));
        }
    }
}
