use crate::history::{HistoryBuffer, Transaction, TransactionKind};
use crate::ledger::Ledger;
use crate::management;
use crate::stats::StatsData;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;

#[derive(CandidType, Clone, Deserialize)]
enum TransactionKindV0 {
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
struct TransactionV0 {
    pub timestamp: u64,
    pub cycles: u64,
    pub fee: u64,
    pub kind: TransactionKindV0,
}

impl From<TransactionV0> for Transaction {
    fn from(tx: TransactionV0) -> Self {
        Transaction {
            timestamp: tx.timestamp,
            cycles: tx.cycles,
            fee: tx.fee,
            kind: tx.kind.into(),
        }
    }
}

impl From<TransactionKindV0> for TransactionKind {
    fn from(tx: TransactionKindV0) -> Self {
        match tx {
            TransactionKindV0::Transfer { from, to } => TransactionKind::Transfer { from, to },
            TransactionKindV0::Mint { to } => TransactionKind::Mint { to },
            TransactionKindV0::Burn { from, to } => TransactionKind::Burn { from, to },
            TransactionKindV0::CanisterCalled {
                canister,
                method_name,
            } => TransactionKind::CanisterCalled {
                from: Principal::anonymous(),
                canister,
                method_name,
            },
            TransactionKindV0::CanisterCreated { canister } => TransactionKind::CanisterCreated {
                from: Principal::anonymous(),
                canister,
            },
            TransactionKindV0::ChargingStationDeployed { canister } => {
                TransactionKind::ChargingStationDeployed {
                    from: Principal::anonymous(),
                    canister,
                }
            }
        }
    }
}

#[derive(CandidType, Deserialize)]
struct StableStorageV0 {
    ledger: Vec<(Principal, u64)>,
    history: Vec<TransactionV0>,
    controller: Principal,
    stats: StatsData,
}

#[derive(CandidType, Deserialize)]
struct StableStorage {
    ledger: Vec<(Principal, u64)>,
    history: Vec<Transaction>,
    controller: Principal,
    stats: StatsData,
}

#[pre_upgrade]
pub fn pre_upgrade() {
    let ledger = storage::get_mut::<Ledger>().archive();
    let history = storage::get_mut::<HistoryBuffer>().archive();
    let controller = management::Controller::get_principal();

    let stable = StableStorage {
        ledger,
        history,
        controller,
        stats: StatsData::get(),
    };

    match storage::stable_save((stable,)) {
        Ok(_) => (),
        Err(candid_err) => {
            trap(&format!(
                "An error occurred when saving to stable memory (pre_upgrade): {:?}",
                candid_err
            ));
        }
    };
}

#[post_upgrade]
pub fn post_upgrade() {
    if let Ok((stable,)) = storage::stable_restore::<(StableStorageV0,)>() {
        let mut history = Vec::with_capacity(stable.history.len());
        for event in stable.history {
            history.push(event.into());
        }

        storage::get_mut::<Ledger>().load(stable.ledger);
        storage::get_mut::<HistoryBuffer>().load(history);
        management::Controller::load(stable.controller);
        StatsData::load(stable.stats);
    }
}
