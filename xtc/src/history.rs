use crate::stats::{CountTarget, StatsData};
use ic_cdk::*;
use ic_cdk_macros::*;
use xtc_history::{History, HistoryArchive, HistoryArchiveBorrowed};

pub use xtc_history_types::*;

pub struct HistoryBuffer {
    history: History,
}

impl Default for HistoryBuffer {
    fn default() -> Self {
        HistoryBuffer {
            history: History::new(10_000, 2000),
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

            trap("Transaction is expected to have a non-zero amount.")
        }

        StatsData::increment(match &transaction.kind {
            TransactionKind::Transfer { .. } => CountTarget::Transfer,
            TransactionKind::Mint { .. } => CountTarget::Mint,
            TransactionKind::Burn { .. } => CountTarget::Burn,
            TransactionKind::CanisterCalled { .. } => CountTarget::ProxyCall,
            TransactionKind::CanisterCreated { .. } => CountTarget::CanisterCreated,
        });

        transaction.timestamp = transaction.timestamp / 1000000;
        self.history.push(transaction)
    }

    #[inline]
    pub fn len(&self) -> u64 {
        self.history.len()
    }
}

#[update]
async fn get_transaction(id: TransactionId) -> Option<Transaction> {
    storage::get::<HistoryBuffer>()
        .history
        .get_transaction(id)
        .await
}

#[query]
fn events(args: EventsArgs) -> EventsConnection<'static> {
    let from = args.from.unwrap_or(0);
    let limit = args.limit.min(512);

    storage::get::<HistoryBuffer>().history.events(from, limit)
}
