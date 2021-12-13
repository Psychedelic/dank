use crate::fee::compute_fee;
use crate::ledger::Ledger;
use ic_kit::candid::{CandidType, Nat};
use ic_kit::interfaces::management::WithCanisterId;
use ic_kit::{async_test, Context, MockContext, Principal, RejectionCode};
use ic_kit::{mock_principals, Method, RawHandler};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

fn reset_ledger(ctx: &mut MockContext) {
    let ledger = ctx.get_mut::<Ledger>();
    ledger.deposit(&Principal::anonymous(), 3 * 10_000_000_000_000);
    ledger.load(vec![
        (mock_principals::alice(), 10_000_000_000_000),
        (mock_principals::bob(), 10_000_000_000_000),
        (mock_principals::john(), 10_000_000_000_000),
    ]);
}

/// General method to test the behaviour of fee implementation of methods.
async fn test_with_call_fee<T: CandidType + Clone, O, E: Debug>(
    response: T,
    cb: Box<dyn Fn(u64) -> Pin<Box<dyn Future<Output = Result<O, E>>>>>,
) {
    let ctx = MockContext::new()
        .with_caller(mock_principals::alice())
        .inject();

    // Zero.
    reset_ledger(ctx);
    ctx.clear_handlers();
    ctx.use_handler(
        Method::new()
            .response(response.clone())
            .cycles_consume(2_000),
    );
    cb(0).await.expect("Unexpected error.");
    ctx.call_state_reset();

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::alice()),
        10_000_000_000_000 - 0 - compute_fee(0)
    );

    // Consumes all
    reset_ledger(ctx);
    cb(1_000).await.expect("Unexpected error.");
    ctx.call_state_reset();

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::alice()),
        10_000_000_000_000 - 1_000 - compute_fee(1_000)
    );

    // With refund.
    reset_ledger(ctx);
    cb(10_000).await.expect("Unexpected error.");
    ctx.call_state_reset();

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::alice()),
        10_000_000_000_000 - 2_000 - compute_fee(2_000)
    );

    // With error.
    reset_ledger(ctx);
    ctx.clear_handlers();
    ctx.use_handler(RawHandler::raw(Box::new(|_, _, _, _| {
        Err((
            RejectionCode::DestinationInvalid,
            "Canister not found.".into(),
        ))
    })));
    cb(10_000).await.err().expect("Expected Err response.");
    ctx.call_state_reset();

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::alice()),
        10_000_000_000_000 - compute_fee(10_000)
    );
}

#[async_test]
async fn wallet_call_fee() {
    use crate::cycles_wallet::*;
    test_with_call_fee(
        (),
        Box::new(|cycles| {
            Box::pin(async move {
                call(CallCanisterArgs {
                    canister: mock_principals::john(),
                    method_name: "xxx".to_string(),
                    args: vec![],
                    cycles,
                })
                .await
            })
        }),
    )
    .await;
}

#[async_test]
async fn create_canister_fee() {
    use crate::cycles_wallet::*;
    test_with_call_fee(
        WithCanisterId {
            canister_id: mock_principals::xtc(),
        },
        Box::new(|cycles| {
            Box::pin(async move {
                create_canister(CreateCanisterArgs {
                    cycles,
                    controller: None,
                })
                .await
            })
        }),
    )
    .await;
}

#[async_test]
async fn send_fee() {
    use crate::cycles_wallet::*;
    test_with_call_fee(
        (),
        Box::new(|cycles| {
            Box::pin(async move {
                wallet_send(SendCyclesArgs {
                    canister: mock_principals::xtc(),
                    amount: cycles,
                })
                .await
            })
        }),
    )
    .await;
}

#[async_test]
async fn burn_fee() {
    use crate::ledger::*;
    test_with_call_fee(
        (),
        Box::new(|cycles| {
            Box::pin(async move {
                burn(BurnArguments {
                    canister_id: mock_principals::xtc(),
                    amount: cycles,
                })
                .await
            })
        }),
    )
    .await;
}

#[async_test]
async fn transfer_fee() {
    use crate::ledger::*;

    let ctx = MockContext::new()
        .with_caller(mock_principals::alice())
        .inject();

    reset_ledger(ctx);

    transfer(mock_principals::bob(), Nat::from(5000))
        .await
        .expect("Unexpected error.");

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::alice()),
        10_000_000_000_000 - 5_000 - compute_fee(5_000)
    );

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::bob()),
        10_000_000_000_000 + 5_000
    );
}

#[async_test]
#[should_panic]
async fn transfer_fee_zero() {
    use crate::ledger::*;

    let ctx = MockContext::new()
        .with_caller(mock_principals::alice())
        .inject();

    reset_ledger(ctx);

    transfer(mock_principals::bob(), Nat::from(0))
        .await
        .expect("Unexpected error.");

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::alice()),
        10_000_000_000_000 - 0 - compute_fee(0)
    );

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::bob()),
        10_000_000_000_000 + 0
    );
}

#[async_test]
async fn mint_fee() {
    use crate::ledger::*;

    let ctx = MockContext::new()
        .with_caller(mock_principals::alice())
        .with_msg_cycles(50_000_000_000)
        .inject();

    mint(mock_principals::alice(), Nat::from(0))
        .await
        .expect("Unexpected error.");

    assert_eq!(
        ctx.get::<Ledger>().balance(&mock_principals::alice()),
        50_000_000_000 - compute_fee(50_000_000_000)
    );
}
