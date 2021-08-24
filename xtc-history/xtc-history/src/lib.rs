use crate::flush::{HistoryFlusher, ProgressResult};
use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;
use ic_cdk::{id, trap};
use serde::Deserialize;
use std::collections::BTreeMap;
use xtc_history_types::{EventsConnection, Transaction, TransactionId};

mod flush;

pub struct History {
    next_event_id: TransactionId,
    buffer: Vec<Transaction>,
    buckets: Vec<(TransactionId, Principal)>,
    flusher: Option<HistoryFlusher>,
    // configs
    chunk_size: usize,
    flush_threshold: usize,
}

#[derive(CandidType)]
pub struct HistoryArchiveBorrowed<'e, 'b> {
    cursor: TransactionId,
    events: &'e Vec<Transaction>,
    buckets: &'b Vec<(TransactionId, Principal)>,
}

#[derive(CandidType, Deserialize)]
pub struct HistoryArchive {
    cursor: TransactionId,
    events: Vec<Transaction>,
    buckets: Vec<(TransactionId, Principal)>,
}

impl History {
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
    pub fn len(&self) -> u64 {
        self.next_event_id
    }

    #[inline]
    pub fn get_transaction(&self, id: TransactionId) -> Option<&Transaction> {
        todo!()
    }

    pub fn events(&self, from: u64, limit: u16) -> EventsConnection {
        todo!()
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

impl History {
    #[inline]
    pub fn archive(&self) -> HistoryArchiveBorrowed {
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
    pub fn load(&mut self, mut archive: HistoryArchive) {
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
