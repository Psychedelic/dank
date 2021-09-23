use crate::stats::{CountTarget, StatsData};
use ic_kit::Principal;
use ic_kit::{ic, macros::*};
use xtc_history::data::{HistoryArchive, HistoryArchiveBorrowed};
use xtc_history::History;

use xtc_history::ic::IcBackend;
pub use xtc_history_common::types::*;

pub struct HistoryBuffer {
    history: History,
}

impl Default for HistoryBuffer {
    fn default() -> Self {
        HistoryBuffer {
            history: History::<Principal, IcBackend>::new(5_000, 2000),
        }
    }
}

impl HistoryBuffer {
    #[inline]
    pub fn archive(&mut self) -> HistoryArchiveBorrowed {
        self.history.archive()
    }

    #[inline]
    pub fn load(&mut self, archive: HistoryArchive) {
        self.history.load(archive);
    }

    #[inline]
    pub fn load_v0(&mut self, data: Vec<Transaction>) {
        self.history.load_v0(data);
    }

    #[inline]
    pub async fn progress(&mut self) -> bool {
        self.history.progress().await
    }

    #[inline]
    pub fn push(&mut self, mut transaction: Transaction) -> TransactionId {
        if transaction.cycles == 0 {
            if let TransactionKind::CanisterCalled { .. } = transaction.kind {
                // In case it is a call to another canister, just return zero, this number
                // is not returned from the method, so it should be fine.
                return 0;
            }

            if transaction.fee == 0 {
                panic!("Transaction is expected to have a non-zero amount.")
            }
        }

        StatsData::capture_fee(transaction.fee);
        StatsData::increment(match &transaction.kind {
            TransactionKind::Transfer { .. } => CountTarget::Transfer,
            TransactionKind::Mint { .. } => CountTarget::Mint,
            TransactionKind::Burn { .. } => CountTarget::Burn,
            TransactionKind::CanisterCalled { .. } => CountTarget::ProxyCall,
            TransactionKind::CanisterCreated { .. } => CountTarget::CanisterCreated,
        });

        transaction.timestamp /= 1000000;
        self.history.push(transaction)
    }

    #[inline]
    pub fn len(&self) -> u64 {
        self.history.len()
    }
}

#[update]
async fn get_transaction(id: TransactionId) -> Option<Transaction> {
    ic::get::<HistoryBuffer>().history.get_transaction(id).await
}

#[query]
fn events(args: EventsArgs) -> EventsConnection<'static> {
    let offset = args.offset;
    let limit = args.limit.min(512);

    ic::get::<HistoryBuffer>().history.events(offset, limit)
}
