//! Contains source codes related to making Dank compatible with cycles wallet so it can be used
//! by the dfx command line.

use crate::fee::compute_fee;
use crate::history::{HistoryBuffer, Transaction, TransactionKind};
use crate::ledger::Ledger;
use crate::management::IsShutDown;
use crate::meta::meta;
use ic_kit::candid::CandidType;
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
            let transaction = Transaction {
                timestamp: ic.time(),
                cycles,
                fee: actual_fee,
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
            ledger.deposit(caller, args.cycles + deduced_fee);
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
            let actual_fee = compute_fee(cycles);
            let refunded = refunded + (deduced_fee - actual_fee);
            let transaction = Transaction {
                timestamp: ic.time(),
                cycles,
                fee: actual_fee,
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
            ledger.deposit(caller, args.cycles + deduced_fee);
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

    let deduced_fee = compute_fee(args.amount);
    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.amount + deduced_fee)
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
            let actual_fee = compute_fee(cycles);
            let refunded = refunded + (deduced_fee - actual_fee);
            let transaction = Transaction {
                timestamp: ic.time(),
                cycles,
                fee: actual_fee,
                kind: TransactionKind::Burn {
                    from: caller.clone(),
                    to: args.canister,
                },
            };

            ic.get_mut::<HistoryBuffer>().push(transaction);

            (Ok(()), refunded)
        }
        Err(_) => (Err("Call failed.".into()), args.amount + deduced_fee),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::Ledger;
    use ic_kit::{async_test, Context, MockContext, RejectionCode};
    use ic_kit::{mock_principals, Method, RawHandler};

    /// Init a mock ledger that sets an initial 10TC balance for alice, bob, john.
    fn init_ledger(ctx: &mut MockContext) {
        let ledger = ctx.get_mut::<Ledger>();
        ledger.deposit(mock_principals::alice(), 10_000_000_000);
        ledger.deposit(mock_principals::bob(), 10_000_000_000);
        ledger.deposit(mock_principals::john(), 10_000_000_000);
    }

    #[async_test]
    async fn wallet_call_fee() {
        use crate::cycles_wallet::{call, CallCanisterArgs};

        // Create a context that has an inter-canister call handler which accepts upto 2_000 cycles
        // on every call.
        let ctx = MockContext::new()
            .with_consume_cycles_handler(2_000)
            .with_caller(mock_principals::alice())
            .inject();

        init_ledger(ctx);

        call(CallCanisterArgs {
            canister: mock_principals::john(),
            method_name: "xxx".to_string(),
            args: vec![],
            cycles: 1_000,
        })
        .await
        .expect("Unexpected failure.");
        ctx.call_state_reset();

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::alice()),
            10_000_000_000 - 1_000 - compute_fee(1_000)
        );

        // Test when there is a refund.
        ctx.update_caller(mock_principals::bob());

        call(CallCanisterArgs {
            canister: mock_principals::john(),
            method_name: "xxx".to_string(),
            args: vec![],
            cycles: 2_500,
        })
        .await
        .expect("Unexpected failure.");
        ctx.call_state_reset();

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::bob()),
            10_000_000_000 - 2_000 - compute_fee(2_000)
        );

        // Test when the call fails
        ctx.update_caller(mock_principals::john());
        ctx.clear_handlers();
        ctx.use_handler(RawHandler::raw(Box::new(|_, _, _, _| {
            Err((
                RejectionCode::DestinationInvalid,
                "Canister not found.".into(),
            ))
        })));

        call(CallCanisterArgs {
            canister: mock_principals::john(),
            method_name: "xxx".to_string(),
            args: vec![],
            cycles: 2_500,
        })
        .await
        .err()
        .expect("Expected an Err response.");
        ctx.call_state_reset();

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::john()),
            10_000_000_000
        );
    }

    #[async_test]
    async fn create_canister_fee() {
        let ctx = MockContext::new()
            .with_handler(
                Method::new()
                    .cycles_consume(2_000)
                    .response(WithCanisterId {
                        canister_id: mock_principals::xtc(),
                    }),
            )
            .with_caller(mock_principals::alice())
            .inject();

        init_ledger(ctx);

        create_canister(CreateCanisterArgs {
            cycles: 1_000,
            controller: None,
        })
        .await
        .expect("Unexpected error.");
        ctx.call_state_reset();

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::alice()),
            10_000_000_000 - 1_000 - compute_fee(1_000)
        );

        // With refund.

        ctx.update_caller(mock_principals::bob());
        create_canister(CreateCanisterArgs {
            cycles: 2_500,
            controller: None,
        })
        .await
        .expect("Unexpected error.");
        ctx.call_state_reset();

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::bob()),
            10_000_000_000 - 2_000 - compute_fee(2_000)
        );

        // With error.

        ctx.update_caller(mock_principals::john());
        ctx.clear_handlers();
        ctx.use_handler(RawHandler::raw(Box::new(|_, _, _, _| {
            Err((
                RejectionCode::DestinationInvalid,
                "Canister not found.".into(),
            ))
        })));

        create_canister(CreateCanisterArgs {
            cycles: 2_500,
            controller: None,
        })
        .await
        .err()
        .expect("Expected Err response.");

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::john()),
            10_000_000_000
        );
    }

    #[async_test]
    async fn send_fee() {
        let ctx = MockContext::new()
            .with_consume_cycles_handler(2_000)
            .with_caller(mock_principals::alice())
            .inject();

        init_ledger(ctx);

        wallet_send(SendCyclesArgs {
            canister: mock_principals::xtc(),
            amount: 1_000,
        })
        .await
        .expect("Unexpected error.");
        ctx.call_state_reset();

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::alice()),
            10_000_000_000 - 1_000 - compute_fee(1_000)
        );

        // With refund.

        ctx.update_caller(mock_principals::bob());
        wallet_send(SendCyclesArgs {
            canister: mock_principals::xtc(),
            amount: 2_500,
        })
        .await
        .expect("Unexpected error.");
        ctx.call_state_reset();

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::bob()),
            10_000_000_000 - 2_000 - compute_fee(2_000)
        );

        // With error.

        ctx.update_caller(mock_principals::john());
        ctx.clear_handlers();
        ctx.use_handler(RawHandler::raw(Box::new(|_, _, _, _| {
            Err((
                RejectionCode::DestinationInvalid,
                "Canister not found.".into(),
            ))
        })));

        wallet_send(SendCyclesArgs {
            canister: mock_principals::xtc(),
            amount: 2_500,
        })
        .await
        .err()
        .expect("Expected Err response.");

        assert_eq!(
            ctx.get::<Ledger>().balance(&mock_principals::john()),
            10_000_000_000
        );
    }
}
