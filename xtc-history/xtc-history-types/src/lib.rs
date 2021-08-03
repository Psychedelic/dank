use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;
use serde::Deserialize;

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

#[derive(Deserialize, CandidType)]
pub struct SetBucketMetadataArgs {
    pub from: TransactionId,
    pub next: Option<Principal>,
}
