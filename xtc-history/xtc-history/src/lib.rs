use crate::flush::{HistoryFlusher, ProgressResult};
use crate::slice_con::SliceCon;
use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;
use ic_cdk::{id, trap};
use serde::Deserialize;
use std::collections::BTreeMap;
use xtc_history_types::{EventsConnectionOwned, Transaction, TransactionId};

mod flush;
mod slice_con;

pub struct History {
    flusher: Option<HistoryFlusher>,
    data: Vec<Transaction>,
    buckets: BucketsList,
    cursor: TransactionId,
    chunk_size: usize,
    flush_threshold: usize,
}

pub struct BucketsList {
    buckets: BTreeMap<TransactionId, Principal>,
    head: Option<Principal>,
}

#[derive(CandidType)]
pub struct HistoryArchiveBorrowed<'e, 'b> {
    cursor: TransactionId,
    events: &'e Vec<Transaction>,
    buckets: &'b BTreeMap<TransactionId, Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct HistoryArchive {
    cursor: TransactionId,
    events: Vec<Transaction>,
    buckets: BTreeMap<TransactionId, Principal>,
}

impl History {
    pub fn new(flush_threshold: usize, chunk_size: usize) -> History {
        assert!(
            flush_threshold > chunk_size,
            "Flush threshold should be larger than the chunk size"
        );

        History {
            flusher: None,
            data: Vec::with_capacity(flush_threshold),
            buckets: BucketsList::default(),
            cursor: 0,
            chunk_size,
            flush_threshold,
        }
    }

    #[inline]
    pub fn archive(&self) -> HistoryArchiveBorrowed {
        // Prevent upgrades during an active flush.
        assert!(
            self.flusher.is_none(),
            "Flush in progress, try again later."
        );
        HistoryArchiveBorrowed {
            cursor: self.cursor,
            events: &self.data,
            buckets: &self.buckets.buckets,
        }
    }

    #[inline]
    pub fn load(&mut self, mut archive: HistoryArchive) {
        assert!(self.data.is_empty() && self.buckets.buckets.is_empty());
        self.cursor = archive.cursor;
        self.data.append(&mut archive.events);
        self.buckets.buckets.append(&mut archive.buckets);
        self.buckets.set_head();
    }

    #[inline]
    pub fn load_v0(&mut self, mut data: Vec<Transaction>) {
        assert!(self.data.is_empty() && self.buckets.buckets.is_empty());
        self.data.append(&mut data);
        self.cursor = self.data.len() as TransactionId;
    }

    #[inline]
    pub fn len(&self) -> TransactionId {
        self.cursor
    }

    #[inline]
    fn get_index(&self, id: TransactionId) -> (usize, usize) {
        // 00 01 02 03 04 05             6 - 6 = 0
        // 06 07 08 09 10 11            12 - 6 = 6
        // Get(8)  -> Get(8 - (12 - 6)) -> Get(2)
        // 12 13 14 15                  16 - 4 = 12
        // Get(14) -> Get(14 - (16 - 4)) -> Get(2)
        // 16 17                        18 - (4 + 2) = 12   # Previous line in flusher
        // Get(15) -> Get(15 - 12) -> Get(3) ->  3 < 4 -> Get(Flusher(3))
        // Get(16) -> Get(16 - 12) -> Get(4) -> !4 < 4 -> Get(4 - 4) -> Get(0)
        // Get(17) -> Get(17 - 12) -> Get(5) -> !5 < 4 -> Get(5 - 4) -> Get(1)
        let flusher_size = self.flusher.as_ref().map(|f| f.data.len()).unwrap_or(0);
        let size = flusher_size + self.data.len();
        let tmp = self.cursor - size as TransactionId;

        if id < tmp {
            trap("Transaction not in this canister.")
        }

        let index = (id - tmp) as usize;

        (flusher_size, index)
    }

    #[inline]
    pub fn get_transaction(&self, id: TransactionId) -> Option<&Transaction> {
        let (flusher_size, index) = self.get_index(id);
        if index < flusher_size {
            Some(&self.flusher.as_ref().unwrap().data[index])
        } else {
            let index = index - flusher_size;
            if index >= self.data.len() {
                trap("Transaction ID is larger than expected.");
            }
            Some(&self.data[index])
        }
    }

    pub fn events(&self, from: u64, limit: u16) -> EventsConnectionOwned {
        let limit = limit as usize + 1;
        let (flusher_size, mut index) = self.get_index(from);
        let mut end_index = index + limit;
        let mut events = Vec::with_capacity(limit);

        if index < flusher_size {
            let flusher_data = &self.flusher.as_ref().unwrap().data;
            let flusher_end_index = end_index.min(flusher_size);

            for i in index..flusher_end_index {
                events.push(flusher_data[i].clone());
            }

            index = index - flusher_size;
            end_index = end_index - flusher_size;
        }

        let data = &self.data;
        for i in index..end_index.min(data.len()) {
            events.push(data[i].clone());
        }

        let size = events.len();
        events.pop();

        EventsConnectionOwned {
            data: events,
            next_canister_id: if size == limit {
                Some(id())
            } else {
                self.buckets.head
            },
        }
    }

    /// Push a new transaction to the history events log.
    /// This method should only be called from an update.
    pub fn push(&mut self, event: Transaction) -> TransactionId {
        let id = self.cursor;
        self.data.push(event);
        self.cursor += 1;

        // Start the flush process if the conditions are met.
        if self.data.len() == self.flush_threshold && self.flusher.is_none() {
            // Number of history pushes we expect during the flush.
            let capacity = self.flush_threshold / self.chunk_size * 3;
            let empty_vec = Vec::with_capacity(capacity);
            let data = std::mem::replace(&mut self.data, empty_vec);
            let head = self.buckets.head.clone();
            self.flusher = Some(HistoryFlusher::new(
                data,
                head,
                self.cursor,
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
                let result = flusher.progress(&mut self.buckets).await;

                match result {
                    ProgressResult::Ok => true,
                    ProgressResult::Blocked => false,
                    ProgressResult::Done => {
                        // The flusher.data is pre-allocated, it's a waste to re-allocate and let
                        // it go, so we swap it with the self.data, so self.data is pre-allocated,
                        // and since the data inside of it is old, and we don't need it anymore,
                        // we clear it, but there has been some inserts to the history while the
                        // flush was taking place, we have written them to what is now `flush.data`
                        // so move all of those events to `self.data`.
                        std::mem::swap(&mut self.data, &mut flusher.data);
                        self.data.clear();
                        self.data.append(&mut flusher.data);
                        self.flusher = None;
                        false
                    }
                }
            }
            None => false,
        }
    }
}

impl BucketsList {
    #[inline]
    pub fn insert(&mut self, id: Principal, from: TransactionId) {
        self.head = Some(id.clone());
        self.buckets.insert(from, id);
    }

    #[inline]
    pub fn get_head(&self) -> Option<&Principal> {
        self.head.as_ref()
    }

    /// Sync the .head with the actual head, always call when you modify .buckets
    /// manually.
    #[inline]
    pub(crate) fn set_head(&mut self) {
        let last = self.buckets.iter().rev().take(1).next();
        self.head = match last {
            None => None,
            Some((_, id)) => Some(id.clone()),
        };
    }
}

impl Default for BucketsList {
    fn default() -> Self {
        BucketsList {
            buckets: BTreeMap::new(),
            head: None,
        }
    }
}
