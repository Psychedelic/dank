use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;
use serde::Deserialize;

#[derive(CandidType, Clone, Deserialize, PartialOrd, PartialEq, Debug)]
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
        from: Principal,
        canister: Principal,
        method_name: String,
    },
    CanisterCreated {
        from: Principal,
        canister: Principal,
    },
}

#[derive(CandidType, Clone, Deserialize, PartialOrd, PartialEq, Debug)]
pub struct Transaction {
    pub timestamp: u64,
    pub cycles: u64,
    pub fee: u64,
    pub kind: TransactionKind,
}

#[derive(Deserialize, CandidType)]
pub struct EventsArgs {
    pub offset: Option<u64>,
    pub limit: u16,
}

#[derive(CandidType)]
pub struct EventsConnection<'a, Address = Principal, Event: 'a = Transaction> {
    pub data: Vec<&'a Event>,
    pub next_offset: TransactionId,
    pub next_canister_id: Option<Address>,
}

pub type TransactionId = u64;

#[derive(Deserialize, CandidType)]
pub struct SetBucketMetadataArgs<Address = Principal> {
    pub from: TransactionId,
    pub next: Option<Address>,
}
