use derive_builder::*;
use derive_new::*;
use ic_kit::{
    candid::{CandidType, Int, Nat},
    Principal,
};
use std::convert::{TryFrom, TryInto};
use xtc_history_common::types::*;

type Time = Int;

#[derive(CandidType, Clone)]
pub enum Operation {
    approve,
    mint,
    transfer,
    transferFrom,
    burn,
    canisterCalled,
    canisterCreated,
}

#[derive(CandidType)]
pub struct Metadata<'a> {
    pub decimals: u8,
    pub fee: Nat,
    pub logo: &'a str,
    pub name: &'a str,
    pub owner: Principal,
    pub symbol: &'a str,
    pub totalSupply: Nat,
}

#[derive(CandidType, Debug, Eq, PartialEq)]
pub enum TxError {
    InsufficientAllowance,
    InsufficientBalance,
}

pub type TxReceipt = Result<Nat, TxError>;

#[derive(CandidType, Clone, new)]
pub struct TxRecord {
    pub caller: Option<Principal>,
    pub from: Principal,
    pub to: Principal,
    pub amount: Nat,
    pub fee: Nat,
    pub op: Operation,
    pub timestamp: Time,
    pub index: Nat,
}

impl TxRecord {
    pub fn setIndex(mut self, index: Nat) -> Self {
        self.index = index;
        self
    }
}

impl TryFrom<Transaction> for TxRecord {
    type Error = ();
    fn try_from(transaction: Transaction) -> Result<TxRecord, ()> {
        Ok(match transaction.kind {
            TransactionKind::Approve { from, to } => TxRecord::new(
                None,
                from,
                to,
                Nat::from(transaction.cycles),
                Nat::from(transaction.fee),
                Operation::approve,
                Int::from(transaction.timestamp),
                Nat::from(0),
            ),
            TransactionKind::Transfer { from, to } => TxRecord::new(
                None,
                from,
                to,
                Nat::from(transaction.cycles),
                Nat::from(transaction.fee),
                Operation::transfer,
                Int::from(transaction.timestamp),
                Nat::from(0),
            ),
            TransactionKind::TransferFrom { caller, from, to } => TxRecord::new(
                Some(caller),
                from,
                to,
                Nat::from(transaction.cycles),
                Nat::from(transaction.fee),
                Operation::transferFrom,
                Int::from(transaction.timestamp),
                Nat::from(0),
            ),
            TransactionKind::Mint { to } => TxRecord::new(
                None,
                to,
                to,
                Nat::from(transaction.cycles),
                Nat::from(transaction.fee),
                Operation::mint,
                Int::from(transaction.timestamp),
                Nat::from(0),
            ),
            TransactionKind::Burn { from, to } => TxRecord::new(
                None,
                from,
                to,
                Nat::from(transaction.cycles),
                Nat::from(transaction.fee),
                Operation::burn,
                Int::from(transaction.timestamp),
                Nat::from(0),
            ),
            TransactionKind::CanisterCalled {
                from,
                canister,
                method_name,
            } => TxRecord::new(
                None,
                from,
                canister,
                Nat::from(transaction.cycles),
                Nat::from(transaction.fee),
                Operation::canisterCalled,
                Int::from(transaction.timestamp),
                Nat::from(0),
            ),
            TransactionKind::CanisterCreated { from, canister } => TxRecord::new(
                None,
                from,
                canister,
                Nat::from(transaction.cycles),
                Nat::from(transaction.fee),
                Operation::canisterCreated,
                Int::from(transaction.timestamp),
                Nat::from(0),
            ),
        })
    }
}
