use crate::backend::Backend;
use crate::data::*;
use crate::flush::{HistoryFlusher, ProgressResult};
use crate::ic::IcBackend;
use ic_cdk::export::Principal;
use xtc_history_common::types::*;

pub mod backend;
pub mod data;
pub mod flush;
pub mod ic;
pub mod mock;

/// A smart history buffer which wraps the bucket and flusher together to provide a bucket
/// implementation that can automatically scale up and flush its data to other canisters to
/// open room for more events.
pub struct History<Address = Principal, Storage: Backend<Address> = IcBackend> {
    data: HistoryData<Address>,
    flusher: Option<HistoryFlusher<Address, Storage>>,
    // configs
    chunk_size: usize,
    flush_threshold: usize,
}

impl<Address: Clone + std::cmp::PartialEq, Storage: Backend<Address>> History<Address, Storage> {
    /// Create a new history instance with the given configuration values.
    ///
    /// # Panics
    /// If flush threshold is smaller than the chunk size.
    pub fn new(flush_threshold: usize, chunk_size: usize) -> History {
        assert!(
            flush_threshold > chunk_size,
            "Flush threshold should be larger than the chunk size"
        );

        History {
            data: HistoryData::default(),
            flusher: None,
            chunk_size,
            flush_threshold,
        }
    }

    /// Return the total number of events inserted into the history.
    #[inline]
    pub fn len(&self) -> u64 {
        self.data.size()
    }

    #[inline]
    pub async fn get_transaction(&self, id: TransactionId) -> Option<Transaction> {
        self.data.get_transaction::<Storage>(id).await
    }

    #[inline]
    pub fn events(&self, offset: Option<u64>, limit: u16) -> EventsConnection<Address> {
        self.data.events::<Storage>(offset, limit as usize)
    }

    /// Push a new transaction to the history events log.
    /// This method should only be called from an update.
    pub fn push(&mut self, event: Transaction) -> TransactionId {
        let id = self.data.push(event);

        if self.data.len() == self.flush_threshold && self.flusher.is_none() {
            self.flusher = Some(HistoryFlusher::new(
                self.data.bucket_exists(),
                self.chunk_size,
            ));
        }

        id
    }

    /// Perform an async task related to the history.
    /// Returns whether the call resulted in an async call or not.
    ///
    /// This method should be called at the beginning of the update calls.
    #[inline]
    pub async fn progress(&mut self) -> bool {
        match &mut self.flusher {
            Some(flusher) => {
                let result = flusher.progress(&mut self.data).await;

                match result {
                    ProgressResult::Ok => true,
                    ProgressResult::Blocked => false,
                    ProgressResult::Done => {
                        self.flusher = None;
                        false
                    }
                }
            }
            None => false,
        }
    }
}

impl<Address: Clone, Storage: Backend<Address>> History<Address, Storage> {
    #[inline]
    pub fn archive(&self) -> HistoryArchiveBorrowed<Address> {
        // Prevent upgrades during an active flush.
        assert!(
            self.flusher.is_none(),
            "History flush in progress, try again later."
        );

        self.data.archive()
    }

    #[inline]
    pub fn load(&mut self, archive: HistoryArchive<Address>) {
        self.data.load(archive);
    }

    #[inline]
    pub fn load_v0(&mut self, data: Vec<Transaction>) {
        self.data.load_v0(data)
    }
}
