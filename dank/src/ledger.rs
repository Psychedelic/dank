use ic_cdk::export::candid::{CandidType, Nat, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::*;
use std::collections::HashMap;

pub struct Ledger(HashMap<Principal, u64>);

impl Default for Ledger {
    fn default() -> Self {
        Self(HashMap::with_capacity(25_000))
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
    }
}

#[update]
pub fn balance(account: Option<Principal>) -> u64 {
    let ledger = storage::get::<Ledger>();
    let account = match account {
        Some(account) => account,
        None => caller(),
    };
    ledger.0.get(&account).cloned().unwrap_or(0)
}

#[derive(Deserialize)]
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
fn transfer(args: TransferArguments) -> Result<Nat, TransferError> {
    let ledger = storage::get_mut::<Ledger>();

    let sender_balance = match ledger.0.get_mut(&caller()) {
        None => return Err(TransferError::InsufficientBalance),
        Some(balance) if *balance < args.amount => return Err(TransferError::InsufficientBalance),
        Some(balance) => balance,
    };

    *sender_balance = *sender_balance - args.amount;

    let recipient = ledger.0.entry(args.to).or_insert(0);
    *recipient = *recipient + args.amount;

    // TODO(qti3e) Implement the history module.
    Ok(Nat::from(0))
}

#[derive(CandidType)]
enum DepositError {
    NotSufficientLiquidity,
}

#[update]
fn deposit(account: Option<Principal>) -> Result<Nat, DepositError> {
    let account = match account {
        Some(account) => account,
        None => caller(),
    };

    let available = api::call::msg_cycles_available();
    let accepted = api::call::msg_cycles_accept(available);

    let ledger = storage::get_mut::<Ledger>();
    let balance = ledger.0.entry(account).or_insert(0);
    *balance += accepted;

    // TODO(qti3e) History module.
    Ok(Nat::from(0))
}

#[derive(Deserialize)]
struct WithdrawArguments {
    amount: u64,
    canister: Principal,
}

#[derive(CandidType)]
enum WithdrawError {
    InsufficientBalance,
    InvalidTokenContract,
    NotSufficientLiquidity,
}

#[update]
async fn withdraw(args: WithdrawArguments) -> Result<Nat, WithdrawError> {
    let ledger = storage::get_mut::<Ledger>();

    let balance = match ledger.0.get_mut(&caller()) {
        None => return Err(WithdrawError::InsufficientBalance),
        Some(balance) if *balance < args.amount => return Err(WithdrawError::InsufficientBalance),
        Some(balance) => balance,
    };

    *balance -= args.amount;

    #[derive(CandidType)]
    struct DepositCyclesArg {
        canister_id: Principal,
    }

    let deposit_cycles_arg = DepositCyclesArg {
        canister_id: args.canister,
    };

    let (result, refund) = match api::call::call_with_payment(
        Principal::management_canister(),
        "deposit_cycles",
        (deposit_cycles_arg,),
        args.amount.into(),
    )
    .await
    {
        Ok(()) => (Ok(Nat::from(0)), api::call::msg_cycles_refunded()),
        Err(_) => (Err(WithdrawError::InvalidTokenContract), args.amount),
    };

    if refund > 0 {
        let balance = ledger.0.entry(caller()).or_insert(0);
        *balance += refund;
    }

    result
}
