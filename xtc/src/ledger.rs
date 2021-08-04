use crate::history::{HistoryBuffer, Transaction, TransactionId, TransactionKind};
use crate::management::IsShutDown;
use crate::stats::StatsData;
use ic_cdk::export::candid::{CandidType, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct Ledger(HashMap<Principal, u64>);

impl Default for Ledger {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl Ledger {
    pub fn archive(&mut self) -> Vec<(Principal, u64)> {
        let map = std::mem::replace(&mut self.0, HashMap::new());
        map.into_iter()
            .filter(|(_, balance)| *balance > 0)
            .collect()
    }

    pub fn load(&mut self, archive: Vec<(Principal, u64)>) {
        self.0 = archive.into_iter().collect();
        self.0.reserve(25_000 - self.0.len());
    }

    #[inline]
    pub fn balance(&self, account: &Principal) -> u64 {
        match self.0.get(account) {
            Some(balance) => *balance,
            None => 0,
        }
    }

    #[inline]
    pub fn deposit(&mut self, account: Principal, amount: u64) {
        StatsData::deposit(amount);
        match self.0.entry(account) {
            Entry::Occupied(mut e) => {
                e.insert(*e.get() + amount);
            }
            Entry::Vacant(e) => {
                e.insert(amount);
            }
        }
    }

    #[inline]
    pub fn withdraw(&mut self, account: &Principal, amount: u64) -> Result<(), ()> {
        let balance = match self.0.get_mut(&account) {
            Some(balance) if *balance >= amount => {
                *balance -= amount;
                *balance
            }
            _ => return Err(()),
        };

        if balance == 0 {
            self.0.remove(&account);
        }

        StatsData::withdraw(amount);

        Ok(())
    }
}

#[update]
pub fn balance(account: Option<Principal>) -> u64 {
    crate::progress().await;
    let ledger = storage::get::<Ledger>();
    ledger.balance(&account.unwrap_or_else(|| caller()))
}

#[derive(Deserialize, CandidType)]
struct TransferArguments {
    to: Principal,
    amount: u64,
    // TODO(qt3ie) Notify argument.
}

#[derive(CandidType)]
enum TransferError {
    InsufficientBalance,
    AmountTooLarge,
    CallFailed,
    Unknown,
}

#[update]
async fn transfer(args: TransferArguments) -> Result<TransactionId, TransferError> {
    IsShutDown::guard();
    crate::progress().await;

    let user = caller();
    let ledger = storage::get_mut::<Ledger>();

    ledger
        .withdraw(&user, args.amount)
        .map_err(|_| TransferError::InsufficientBalance)?;
    ledger.deposit(args.to, args.amount);

    let transaction = Transaction {
        timestamp: api::time(),
        cycles: args.amount,
        fee: 0,
        kind: TransactionKind::Transfer {
            from: user,
            to: args.to,
        },
    };

    let id = storage::get_mut::<HistoryBuffer>().push(transaction);
    Ok(id)
}

#[derive(CandidType)]
enum MintError {
    NotSufficientLiquidity,
}

#[update]
async fn mint(account: Option<Principal>) -> Result<TransactionId, MintError> {
    IsShutDown::guard();
    crate::progress().await;

    let account = account.unwrap_or_else(|| caller());
    let available = api::call::msg_cycles_available();
    let accepted = api::call::msg_cycles_accept(available);

    let ledger = storage::get_mut::<Ledger>();
    ledger.deposit(account.clone(), accepted);

    let transaction = Transaction {
        timestamp: api::time(),
        cycles: accepted,
        fee: 0,
        kind: TransactionKind::Mint { to: account },
    };

    let id = storage::get_mut::<HistoryBuffer>().push(transaction);
    Ok(id)
}

#[derive(Deserialize, CandidType)]
struct BurnArguments {
    canister_id: Principal,
    amount: u64,
}

#[derive(CandidType)]
enum BurnError {
    InsufficientBalance,
    InvalidTokenContract,
    NotSufficientLiquidity,
}

#[update]
async fn burn(args: BurnArguments) -> Result<TransactionId, BurnError> {
    IsShutDown::guard();
    let user = caller();
    let ledger = storage::get_mut::<Ledger>();

    ledger
        .withdraw(&user, args.amount)
        .map_err(|_| BurnError::InsufficientBalance)?;

    #[derive(CandidType)]
    struct DepositCyclesArg {
        canister_id: Principal,
    }

    let deposit_cycles_arg = DepositCyclesArg {
        canister_id: args.canister_id,
    };

    let (result, refunded) = match api::call::call_with_payment(
        Principal::management_canister(),
        "deposit_cycles",
        (deposit_cycles_arg,),
        args.amount.into(),
    )
    .await
    {
        Ok(()) => {
            let refunded = api::call::msg_cycles_refunded();
            let cycles = args.amount - refunded;
            let transaction = Transaction {
                timestamp: api::time(),
                cycles,
                fee: 0,
                kind: TransactionKind::Burn {
                    from: user.clone(),
                    to: args.canister_id,
                },
            };

            let id = storage::get_mut::<HistoryBuffer>().push(transaction);

            (Ok(id), refunded)
        }
        Err(_) => (Err(BurnError::InvalidTokenContract), args.amount),
    };

    if refunded > 0 {
        ledger.deposit(user, refunded);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::Ledger;
    use ic_cdk::export::candid::Principal;

    fn alice() -> Principal {
        Principal::from_text("fterm-bydaq-aaaaa-aaaaa-c").unwrap()
    }

    fn bob() -> Principal {
        Principal::from_text("ai7t5-aibaq-aaaaa-aaaaa-c").unwrap()
    }

    fn john() -> Principal {
        Principal::from_text("hozae-racaq-aaaaa-aaaaa-c").unwrap()
    }

    #[test]
    fn balance() {
        let mut ledger = Ledger::default();
        assert_eq!(ledger.balance(&alice()), 0);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);

        // Deposit should work.
        ledger.deposit(alice(), 1000);
        assert_eq!(ledger.balance(&alice()), 1000);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);

        assert!(ledger.withdraw(&alice(), 100).is_ok());
        assert_eq!(ledger.balance(&alice()), 900);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);

        assert!(ledger.withdraw(&alice(), 1000).is_err());
        assert_eq!(ledger.balance(&alice()), 900);

        ledger.deposit(alice(), 100);
        assert!(ledger.withdraw(&alice(), 1000).is_ok());
        assert_eq!(ledger.balance(&alice()), 0);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);
    }
}
