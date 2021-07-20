use crate::stats::{CountTarget, StatsData};
use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::Deserialize;

pub struct HistoryBuffer {
    transactions: Vec<Transaction>,
}

impl Default for HistoryBuffer {
    fn default() -> Self {
        HistoryBuffer {
            transactions: Vec::with_capacity(50_000),
        }
    }
}

impl HistoryBuffer {
    #[inline]
    pub fn archive(&mut self) -> Vec<Transaction> {
        std::mem::replace(&mut self.transactions, Vec::new())
    }

    #[inline]
    pub fn load(&mut self, mut data: Vec<Transaction>) {
        self.transactions.append(&mut data);
    }

    #[inline]
    pub fn push(&mut self, transaction: Transaction) -> TransactionId {
        StatsData::increment(match &transaction.kind {
            TransactionKind::Transfer { .. } => CountTarget::Transfer,
            TransactionKind::Mint { .. } => CountTarget::Mint,
            TransactionKind::Burn { .. } => CountTarget::Burn,
            TransactionKind::CanisterCalled { .. } => CountTarget::ProxyCall,
            TransactionKind::CanisterCreated { .. } => CountTarget::CanisterCreated,
            TransactionKind::ChargingStationDeployed { .. } => unreachable!(),
        });

        let id = self.transactions.len();
        self.transactions.push(transaction);
        id as TransactionId
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.transactions.len()
    }
}

#[derive(CandidType, Clone, Deserialize)]
pub enum TransactionKind {
    Transfer {
        from: Principal,
        to: Principal,
    },
    Mint {
        to: Principal,
    },
    Burn {
        from: Principal,
        to: Principal,
    },
    CanisterCalled {
        canister: Principal,
        method_name: String,
    },
    CanisterCreated {
        canister: Principal,
    },
    ChargingStationDeployed {
        canister: Principal,
    },
}

#[derive(CandidType, Clone, Deserialize)]
pub struct Transaction {
    pub timestamp: u64,
    pub cycles: u64,
    pub fee: u64,
    pub kind: TransactionKind,
}

#[derive(Deserialize, CandidType)]
pub struct EventsArgs {
    pub from: Option<u64>,
    pub limit: u16,
}

#[derive(CandidType)]
pub struct EventsConnection<'a> {
    pub data: &'a [Transaction],
    pub next_canister_id: Option<Principal>,
}

pub type TransactionId = u64;

#[update]
fn get_transaction(id: TransactionId) -> Option<&'static Transaction> {
    storage::get::<HistoryBuffer>()
        .transactions
        .get(id as usize)
}

#[query]
fn events(args: EventsArgs) -> EventsConnection<'static> {
    let buffer = storage::get::<HistoryBuffer>();
    let from = args.from.unwrap_or(0) as usize;
    let end = from + args.limit.min(512) as usize;

    EventsConnection {
        data: &buffer.transactions[from..end],
        next_canister_id: if end >= buffer.transactions.len() {
            None
        } else {
            Some(id())
        },
    }
}
