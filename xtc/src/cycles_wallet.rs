//! Contains source codes related to making Dank compatible with cycles wallet so it can be used
//! by the dfx command line.

use crate::history::{HistoryBuffer, Transaction, TransactionKind};
use crate::ledger::Ledger;
use crate::management::IsShutDown;
use crate::meta::meta;
use ic_kit::candid::{CandidType};
use ic_kit::interfaces::management::{
    CanisterSettings, CreateCanister, CreateCanisterArgument, WithCanisterId,
};
use ic_kit::interfaces::Method;
use ic_kit::macros::*;
use ic_kit::{get_context, Context, Principal};
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

    let ic = get_context();
    let caller = ic.caller();

    if ic.id() == args.canister {
        return Err("Attempted to call forward on self. This is not allowed.".to_string());
    }

    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.cycles)
        .map_err(|_| "Insufficient Balance".to_string())?;

    let method_name = args.method_name.clone();

    match ic
        .call_raw(
            args.canister.clone(),
            &method_name,
            args.args,
            args.cycles,
        )
        .await
    {
        Ok(x) => {
            let refunded = ic.msg_cycles_refunded();
            let cycles = args.cycles - refunded;
            let transaction = Transaction {
                timestamp: ic.time(),
                cycles,
                fee: 0,
                kind: TransactionKind::CanisterCalled {
                    from: caller.clone(),
                    canister: args.canister.clone(),
                    method_name: args.method_name,
                },
            };

            ic.get_mut::<HistoryBuffer>().push(transaction);

            if refunded > 0 {
                ledger.deposit(caller, refunded);
            }

            Ok(CallResult { r#return: x })
        }
        Err((code, msg)) => {
            ledger.deposit(caller, args.cycles);

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

#[update(name = "wallet_create_canister")]
async fn create_canister(args: CreateCanisterArgs) -> Result<WithCanisterId, String> {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.cycles)
        .map_err(|_| "Insufficient Balance".to_string())?;

    let in_args = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![args.controller.unwrap_or(caller)]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    };

    let create_result = match CreateCanister::perform_with_payment(
        ic,
        Principal::management_canister(),
        (in_args,),
        args.cycles,
    )
    .await
    {
        Ok((r,)) => {
            let refunded = ic.msg_cycles_refunded();
            let cycles = args.cycles - refunded;
            let transaction = Transaction {
                timestamp: ic.time(),
                cycles,
                fee: 0,
                kind: TransactionKind::CanisterCreated {
                    from: caller.clone(),
                    canister: r.canister_id,
                },
            };

            ic.get_mut::<HistoryBuffer>().push(transaction);

            if refunded > 0 {
                ledger.deposit(caller, refunded);
            }

            r
        }
        Err((code, msg)) => {
            ledger.deposit(caller, args.cycles);
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
    let ic = get_context();
    let ledger = ic.get::<Ledger>();
    let amount = ledger.balance(&ic.caller());
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

    let ic = get_context();
    let caller = ic.caller();
    let ledger = ic.get_mut::<Ledger>();

    ledger
        .withdraw(&caller, args.amount)
        .map_err(|_| String::from("Insufficient balance."))?;

    #[derive(CandidType)]
    struct DepositCyclesArg {
        canister_id: Principal,
    }

    let (result, refunded) = match ic
        .call_with_payment(
            args.canister.clone(),
            "wallet_receive",
            (),
            args.amount.into(),
        )
        .await
    {
        Ok(()) => {
            let refunded = ic.msg_cycles_refunded();
            let cycles = args.amount - refunded;
            let transaction = Transaction {
                timestamp: ic.time(),
                cycles,
                fee: 0,
                kind: TransactionKind::Burn {
                    from: caller.clone(),
                    to: args.canister,
                },
            };

            ic.get_mut::<HistoryBuffer>().push(transaction);

            (Ok(()), refunded)
        }
        Err(_) => (Err("Call failed.".into()), args.amount),
    };

    if refunded > 0 {
        ledger.deposit(caller, refunded);
    }

    result
}

#[update]
async fn wallet_create_wallet(_: CreateCanisterArgs) -> Result<WithCanisterId, String> {
    let ic = get_context();
    crate::progress().await;
    Ok(WithCanisterId {
        canister_id: ic.id(),
    })
}
