use crate::flush::{HistoryFlusher, ProgressResult};
use ic_cdk::export::Principal;
use xtc_history_types::{Transaction, TransactionId};

mod flush;

pub struct History {
    flusher: Option<HistoryFlusher>,
    data: Vec<Transaction>,
    head: Option<Principal>,
    cursor: TransactionId,
    chunk_size: usize,
    flush_threshold: usize,
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
            head: None,
            cursor: 0,
            chunk_size,
            flush_threshold,
        }
    }

    /// Push a new transaction to the history events log.
    /// This method should only be called from an update.
    pub fn push(&mut self, event: Transaction) {
        self.data.push(event);
        self.cursor += 1;

        // Start the flush process if the conditions are met.
        if self.data.len() == self.flush_threshold && self.flusher.is_none() {
            // Number of history pushes we expect during the flush.
            let capacity = self.flush_threshold / self.chunk_size * 3;
            let empty_vec = Vec::with_capacity(capacity);
            let data = std::mem::replace(&mut self.data, empty_vec);
            let head = self.head.clone();
            self.flusher = Some(HistoryFlusher::new(
                data,
                head,
                self.cursor,
                self.chunk_size,
            ));
        }
    }

    /// Perform an async task related to the history.
    /// Returns whether the call resulted in an async call or not.
    ///
    /// This method should be called at the beginning of the update calls.
    #[inline]
    pub async fn progress(&mut self) -> bool {
        match &mut self.flusher {
            Some(flusher) => {
                let result = flusher.progress().await;

                // If the head has changed, update it.
                if flusher.head != self.head {
                    self.head = flusher.head.clone();
                }

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
