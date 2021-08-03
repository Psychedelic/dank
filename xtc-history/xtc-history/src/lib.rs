use crate::flush::{HistoryFlusher, ProgressResult};
use xtc_history_types::Transaction;

mod flush;

pub struct History {
    flusher: Option<HistoryFlusher>,
}

impl History {
    pub fn push(&mut self, event: Transaction) {
    }

    #[inline]
    pub async fn progress(&mut self) -> bool {
        match &mut self.flusher {
            Some(flusher) => match flusher.progress().await {
                ProgressResult::Ok => true,
                ProgressResult::Blocked => false,
                ProgressResult::Done => {
                    self.flusher = None;
                    false
                }
            },
            None => false,
        }
    }
}
