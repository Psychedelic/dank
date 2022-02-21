//! Contains source codes related to making Dank compatible with cycles wallet so it can be used
//! by the dfx command line.

use crate::fee::compute_fee;
use crate::history::{HistoryBuffer, Transaction, TransactionKind, TransactionStatus};
use crate::ledger::Ledger;
use crate::management::IsShutDown;
use ic_kit::candid::CandidType;
use ic_kit::interfaces::management::{
    CanisterSettings, CreateCanister, CreateCanisterArgument, WithCanisterId,
};
use ic_kit::interfaces::Method;
use ic_kit::macros::*;
use ic_kit::{get_context, Context, Principal};
use serde::*;

#[derive(CandidType, Deserialize)]
pub struct CallCanisterArgs {
    pub canister: Principal,
    pub method_name: String,
    #[serde(with = "serde_bytes")]
    pub args: Vec<u8>,
    pub cycles: u64,
}

#[derive(CandidType, Deserialize)]
pub struct CallResult {
    #[serde(with = "serde_bytes")]
    pub r#return: Vec<u8>,
}

/// Forward a call to another canister.
#[update(name = "wallet_call")]
pub async fn call(args: CallCanisterArgs) -> Result<CallResult, String> {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    if ic.id() == args.canister {
        return Err("Attempted to call forward on self. This is not allowed.".to_string());
    }

    let deduced_fee = compute_fee(args.cycles);
    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.cycles + deduced_fee)
        .map_err(|_| "Insufficient Balance".to_string())?;

    let method_name = args.method_name.clone();

    match ic
        .call_raw(args.canister.clone(), &method_name, args.args, args.cycles)
        .await
    {
        Ok(x) => {
            let refunded = ic.msg_cycles_refunded();
            let cycles = args.cycles - refunded;
            let actual_fee = compute_fee(cycles);
            let refunded = refunded + (deduced_fee - actual_fee);

            if refunded > 0 {
                ledger.deposit(&caller, refunded);
            }

            ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles,
                fee: actual_fee,
                kind: TransactionKind::CanisterCalled {
                    from: caller.clone(),
                    canister: args.canister.clone(),
                    method_name: args.method_name,
                },
                status: TransactionStatus::SUCCEEDED,
            });

            Ok(CallResult { r#return: x })
        }
        Err((code, msg)) => {
            ledger.deposit(&caller, args.cycles);

            ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles: 0,
                fee: deduced_fee,
                kind: TransactionKind::CanisterCalled {
                    from: caller.clone(),
                    canister: args.canister.clone(),
                    method_name: args.method_name,
                },
                status: TransactionStatus::FAILED,
            });

            Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ))
        }
    }
}

// Create canister call

#[derive(CandidType, Deserialize)]
pub struct CreateCanisterArgs {
    pub cycles: u64,
    pub controller: Option<Principal>,
}

#[update(name = "wallet_create_canister")]
pub async fn create_canister(args: CreateCanisterArgs) -> Result<WithCanisterId, String> {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    let deduced_fee = compute_fee(args.cycles);
    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.cycles + deduced_fee)
        .map_err(|_| "Insufficient Balance".to_string())?;

    let in_args = CreateCanisterArgument {
        settings: Some(CanisterSettings {
            controllers: Some(vec![args.controller.unwrap_or(caller)]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    };

    match CreateCanister::perform_with_payment(
        Principal::management_canister(),
        (in_args,),
        args.cycles,
    )
    .await
    {
        Ok((r,)) => {
            let refunded = ic.msg_cycles_refunded();
            let cycles = args.cycles - refunded;
            let actual_fee = compute_fee(cycles);
            let refunded = refunded + (deduced_fee - actual_fee);

            if refunded > 0 {
                ledger.deposit(&caller, refunded);
            }

            ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles,
                fee: actual_fee,
                kind: TransactionKind::CanisterCreated {
                    from: caller.clone(),
                    canister: r.canister_id,
                },
                status: TransactionStatus::SUCCEEDED,
            });

            Ok(r)
        }
        Err((code, msg)) => {
            ledger.deposit(&caller, args.cycles);

            ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles: 0,
                fee: deduced_fee,
                kind: TransactionKind::CanisterCreated {
                    from: caller.clone(),
                    canister: caller.clone(),
                },
                status: TransactionStatus::FAILED,
            });

            Err(format!(
                "An error happened during the call: {}: {}",
                code as u8, msg
            ))
        }
    }
}

#[derive(CandidType)]
pub struct BalanceResult {
    pub amount: u64,
}

#[query]
pub fn wallet_balance() -> BalanceResult {
    let ic = get_context();
    let ledger = ic.get::<Ledger>();
    let amount = ledger.balance(&ic.caller());
    BalanceResult { amount }
}

#[derive(CandidType, Deserialize)]
pub struct SendCyclesArgs {
    pub canister: Principal,
    pub amount: u64,
}

#[update]
pub async fn wallet_send(args: SendCyclesArgs) -> Result<(), String> {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    let deduced_fee = compute_fee(args.amount);
    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.amount + deduced_fee)
        .map_err(|_| String::from("Insufficient balance."))?;

    #[derive(CandidType)]
    struct DepositCyclesArg {
        canister_id: Principal,
    }

    match ic
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
            let actual_fee = compute_fee(cycles);
            let refunded = refunded + (deduced_fee - actual_fee);

            if refunded > 0 {
                ledger.deposit(&caller, refunded);
            }

            ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles,
                fee: actual_fee,
                kind: TransactionKind::Burn {
                    from: caller.clone(),
                    to: args.canister,
                },
                status: TransactionStatus::SUCCEEDED,
            });

            Ok(())
        }
        Err(_) => {
            ledger.deposit(&caller, args.amount);

            ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles: 0,
                fee: deduced_fee,
                kind: TransactionKind::Burn {
                    from: caller.clone(),
                    to: args.canister,
                },
                status: TransactionStatus::FAILED,
            });

            Err("Call failed.".into())
        }
    }
}

#[update]
pub async fn wallet_create_wallet(_: CreateCanisterArgs) -> Result<WithCanisterId, String> {
    let ic = get_context();
    crate::progress().await;
    Ok(WithCanisterId {
        canister_id: ic.id(),
    })
}
