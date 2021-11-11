use crate::common_types::TxRecord;
use crate::stats::{CountTarget, StatsData};
use crate::utils::convert_nat_to_u64;
use ic_kit::{candid::Nat, get_context, macros::*, Context, Principal};
use std::cmp::min;
use std::convert::TryInto;
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
            history: History::<Principal, IcBackend>::new(504_000, 5_000),
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
        assert_ne!(
            transaction.fee, 0,
            "transaction is expected to have a non-zero amount"
        );

        StatsData::capture_fee(transaction.fee);
        StatsData::increment(match &transaction.kind {
            TransactionKind::Transfer { .. } => CountTarget::Transfer,
            TransactionKind::TransferFrom { .. } => CountTarget::TransferFrom,
            TransactionKind::Approve { .. } => CountTarget::Approve,
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

    pub fn history(&self) -> &History {
        &self.history
    }
}

//////////////////// BEGIN OF ERC-20 ///////////////////////

#[update(name = "getTransaction")]
pub async fn get_transaction_erc20(index: Nat) -> TxRecord {
    cap_sdk::get_transaction(convert_nat_to_u64(index).expect("nat to u64 conversion failed"))
        .await
        .expect("unable to retrieve transaction from CAP")
        .try_into()
        .expect("unable to convert Event to TxRecord")
}

// TODO: ticking time bomb, we need to integrate the history service, as we
// are always using the internal buffer here
#[update(name = "getTransactions")]
pub fn get_transactions(start: Nat, limit: Nat) -> Vec<TxRecord> {
    unimplemented!();
}

//////////////////// END OF ERC-20 ///////////////////////

#[update]
pub async fn get_transaction(id: TransactionId) -> Option<Transaction> {
    unimplemented!();
}

#[query]
fn events(args: EventsArgs) -> EventsConnection<'static> {
    unimplemented!();
}
