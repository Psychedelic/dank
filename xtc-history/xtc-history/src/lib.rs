use crate::backend::Backend;
use crate::flush::{HistoryFlusher, ProgressResult};
use crate::ic::IcBackend;
use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;
use serde::Deserialize;
use xtc_history_common::types::*;

pub mod backend;
mod flush;
pub mod ic;
pub mod mock;

pub struct History<Address = Principal, Storage: Backend<Address> = IcBackend> {
    next_event_id: TransactionId,
    buffer: Vec<Transaction>,
    buckets: Vec<(TransactionId, Address)>,
    flusher: Option<HistoryFlusher<Address, Storage>>,
    // configs
    chunk_size: usize,
    flush_threshold: usize,
}

#[derive(CandidType)]
pub struct HistoryArchiveBorrowed<'e, 'b, Address: 'b = Principal> {
    cursor: TransactionId,
    events: &'e Vec<Transaction>,
    buckets: &'b Vec<(TransactionId, Address)>,
}

#[derive(CandidType, Deserialize)]
pub struct HistoryArchive<Address = Principal> {
    cursor: TransactionId,
    events: Vec<Transaction>,
    buckets: Vec<(TransactionId, Address)>,
}

impl<Address: Clone + std::cmp::PartialEq, Storage: Backend<Address>> History<Address, Storage> {
    #[inline]
    pub fn len(&self) -> u64 {
        self.next_event_id
    }

    /// Return the bucket that should contain the given transaction.
    #[inline]
    fn get_bucket_for(&self, id: TransactionId) -> Option<&Address> {
        if self.buckets.is_empty() {
            return None;
        }

        let index = match self.buckets.binary_search_by(|(x, _)| x.cmp(&id)) {
            Ok(index) => index,
            Err(index) => index - 1,
        };

        Some(&self.buckets[index].1)
    }

    #[inline]
    pub async fn get_transaction(&self, id: TransactionId) -> Option<Transaction> {
        let buffer_len = self.buffer.len() as u64;
        let local_start = self.next_event_id - buffer_len;

        if id >= local_start {
            let index = (id - local_start) as usize;
            self.buffer.get(index).cloned()
        } else if let Some(canister_id) = self.get_bucket_for(id) {
            Storage::lookup_transaction(canister_id, id).await.unwrap()
        } else {
            None
        }
    }

    pub fn events(&self, offset: Option<u64>, limit: u16) -> EventsConnection<Address> {
        let offset = offset.unwrap_or(self.next_event_id.checked_sub(1).unwrap_or(0));
        let buffer_len = self.buffer.len() as u64;
        let local_start = self.next_event_id - buffer_len;

        if offset >= local_start {
            let take = limit as usize + 1;
            let e = (offset - local_start) as usize;
            let s = e.checked_sub(take).unwrap_or(0);

            let mut data = &self.buffer[s..e];
            let next_canister_id = if data.len() > limit as usize {
                data = &data[1..];
                Some(Storage::id())
            } else if self.buckets.is_empty() {
                None
            } else {
                Some(self.buckets[self.buckets.len() - 1].1.clone())
            };

            EventsConnection {
                data: data.into_iter().rev().collect(),
                next_offset: offset - data.len() as u64,
                next_canister_id,
            }
        } else {
            EventsConnection {
                data: Vec::with_capacity(0),
                next_offset: offset,
                next_canister_id: self.get_bucket_for(offset).cloned(),
            }
        }
    }

    /// Push a new transaction to the history events log.
    /// This method should only be called from an update.
    pub fn push(&mut self, event: Transaction) -> TransactionId {
        let id = self.next_event_id;
        self.buffer.push(event);
        self.next_event_id += 1;

        // Start the flush process if the conditions are met.
        if self.buffer.len() == self.flush_threshold && self.flusher.is_none() {
            self.flusher = Some(HistoryFlusher::new(
                self.next_event_id,
                self.buffer.len(),
                self.buckets.len() > 0,
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
                let result = flusher.progress(&mut self.buckets, &mut self.buffer).await;

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

impl<Address, Storage: Backend<Address>> History<Address, Storage> {
    pub fn new(flush_threshold: usize, chunk_size: usize) -> History {
        assert!(
            flush_threshold > chunk_size,
            "Flush threshold should be larger than the chunk size"
        );

        // just to be safe. (avoid memory allocation during flush, we don't know if there is
        // memory available or not.)
        let extra = flush_threshold / chunk_size * 5 + 64;

        History {
            next_event_id: 0,
            buffer: Vec::with_capacity(flush_threshold + extra),
            buckets: Vec::with_capacity(100),
            flusher: None,
            chunk_size,
            flush_threshold,
        }
    }

    #[inline]
    pub fn archive(&self) -> HistoryArchiveBorrowed<Address> {
        // Prevent upgrades during an active flush.
        assert!(
            self.flusher.is_none(),
            "History flush in progress, try again later."
        );
        HistoryArchiveBorrowed {
            cursor: self.next_event_id,
            events: &self.buffer,
            buckets: &self.buckets,
        }
    }

    #[inline]
    pub fn load(&mut self, mut archive: HistoryArchive<Address>) {
        assert!(self.buffer.is_empty() && self.buckets.is_empty());
        self.next_event_id = archive.cursor;
        self.buffer.append(&mut archive.events);
        self.buckets.append(&mut archive.buckets);
    }

    #[inline]
    pub fn load_v0(&mut self, mut data: Vec<Transaction>) {
        assert!(self.buffer.is_empty() && self.buckets.is_empty());
        self.buffer.append(&mut data);
        self.next_event_id = self.buffer.len() as TransactionId;
    }
}
