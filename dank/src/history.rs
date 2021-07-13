use ic_cdk::export::candid::{CandidType, Nat};
use ic_cdk::export::Principal;
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
    pub fn push(&mut self, transaction: Transaction) -> Nat {
        let id = self.transactions.len();
        self.transactions.push(transaction);
        Nat::from(id)
    }
}

#[derive(CandidType, Clone, Deserialize)]
pub enum TransactionKind {
    Transfer {
        from: Principal,
        to: Principal,
    },
    Deposit {
        to: Principal,
    },
    Withdraw {
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
    pub from: Option<u32>,
    pub limit: u16,
}

#[derive(CandidType)]
pub struct EventsConnection<'a> {
    pub data: &'a [Transaction],
    pub next_canister_id: Option<Principal>,
}
