//! Contains source codes related to making Dank compatible with cycles wallet so it can be used
//! by the dfx command line.

use crate::history::{HistoryBuffer, Transaction, TransactionKind};
use crate::ledger::Ledger;
use crate::management::IsShutDown;
use crate::meta::meta;
use ic_cdk::export::candid::{CandidType, Nat, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::*;

#[query]
fn name() -> Option<&'static str> {
    Some(meta().name)
}

#[derive(CandidType, Deserialize)]
struct CallCanisterArgs {
    canister: Principal,
    method_name: String,
    #[serde(with = "serde_bytes")]
    args: Vec<u8>,
    cycles: u64,
}

#[derive(CandidType, Deserialize)]
struct CallResult {
    #[serde(with = "serde_bytes")]
    r#return: Vec<u8>,
}

/// Forward a call to another canister.
#[update(name = "wallet_call")]
async fn call(args: CallCanisterArgs) -> Result<CallResult, String> {
    IsShutDown::guard();
    let user = caller();
    if api::id() == user {
        return Err("Attempted to call forward on self. This is not allowed.".to_string());
    }

    let ledger = storage::get_mut::<Ledger>();
    ledger
        .withdraw(&user, args.cycles)
        .map_err(|_| "Insufficient Balance".to_string())?;

    match api::call::call_raw(
        args.canister.clone(),
        &args.method_name,
        args.args,
        args.cycles,
    )
    .await
    {
        Ok(x) => {
            let refunded = api::call::msg_cycles_refunded();
            let cycles = args.cycles - refunded;
            let transaction = Transaction {
                timestamp: api::time(),
                cycles,
                fee: 0,
                kind: TransactionKind::CanisterCalled {
                    from: user.clone(),
                    canister: args.canister.clone(),
                    method_name: args.method_name,
                },
            };

            storage::get_mut::<HistoryBuffer>().push(transaction);

            if refunded > 0 {
                ledger.deposit(user, refunded);
            }

            Ok(CallResult { r#return: x })
        }
        Err((code, msg)) => {
            ledger.deposit(user, args.cycles);

            Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ))
        }
    }
}

// Create canister call

#[derive(CandidType, Deserialize)]
struct CreateCanisterArgs {
    cycles: u64,
    controller: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
struct CreateResult {
    canister_id: Principal,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct CanisterSettings {
    pub controller: Option<Principal>,
    pub compute_allocation: Option<Nat>,
    pub memory_allocation: Option<Nat>,
    pub freezing_threshold: Option<Nat>,
}

#[update(name = "wallet_create_canister")]
async fn create_canister(args: CreateCanisterArgs) -> Result<CreateResult, String> {
    IsShutDown::guard();
    let user = caller();
    if api::id() == user {
        return Err("Attempted to call forward on self. This is not allowed.".to_string());
    }

    let ledger = storage::get_mut::<Ledger>();
    ledger
        .withdraw(&user, args.cycles)
        .map_err(|_| "Insufficient Balance".to_string())?;

    #[derive(CandidType)]
    struct In {
        settings: Option<CanisterSettings>,
    }

    let in_arg = In {
        settings: Some(CanisterSettings {
            controller: Some(args.controller.unwrap_or_else(|| caller())),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    };

    let create_result = match api::call::call_with_payment::<(In,), (CreateResult,)>(
        Principal::management_canister(),
        "create_canister",
        (in_arg,),
        args.cycles,
    )
    .await
    {
        Ok((x,)) => {
            let refunded = api::call::msg_cycles_refunded();
            let cycles = args.cycles - refunded;
            let transaction = Transaction {
                timestamp: api::time(),
                cycles,
                fee: 0,
                kind: TransactionKind::CanisterCreated {
                    from: user.clone(),
                    canister: x.canister_id,
                },
            };

            storage::get_mut::<HistoryBuffer>().push(transaction);

            if refunded > 0 {
                ledger.deposit(user, refunded);
            }

            x
        }
        Err((code, msg)) => {
            ledger.deposit(user, args.cycles);
            return Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ));
        }
    };

    Ok(create_result)
}

#[derive(CandidType)]
struct BalanceResult {
    amount: u64,
}

#[query]
fn wallet_balance() -> BalanceResult {
    let ledger = storage::get::<Ledger>();
    let amount = ledger.balance(&caller());
    BalanceResult { amount }
}

#[derive(CandidType, Deserialize)]
struct SendCyclesArgs {
    canister: Principal,
    amount: u64,
}

#[update]
async fn wallet_send(args: SendCyclesArgs) -> Result<(), String> {
    IsShutDown::guard();
    let user = caller();
    let ledger = storage::get_mut::<Ledger>();

    ledger
        .withdraw(&user, args.amount)
        .map_err(|_| String::from("Insufficient balance."))?;

    #[derive(CandidType)]
    struct DepositCyclesArg {
        canister_id: Principal,
    }

    let (result, refunded) = match api::call::call_with_payment(
        args.canister.clone(),
        "wallet_receive",
        (),
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
                    to: args.canister,
                },
            };

            storage::get_mut::<HistoryBuffer>().push(transaction);

            (Ok(()), refunded)
        }
        Err(_) => (Err("Call failed.".into()), args.amount),
    };

    if refunded > 0 {
        ledger.deposit(user, refunded);
    }

    result
}

#[update]
async fn wallet_create_wallet(_: CreateCanisterArgs) -> Result<CreateResult, String> {
    crate::progress().await;
    Ok(CreateResult {
        canister_id: api::id(),
    })
}
