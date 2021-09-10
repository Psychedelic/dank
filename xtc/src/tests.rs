use crate::fee::compute_fee;
use crate::ledger::Ledger;
use ic_kit::candid::CandidType;
use ic_kit::interfaces::management::WithCanisterId;
use ic_kit::{async_test, Context, MockContext, Principal, RejectionCode};
use ic_kit::{mock_principals, Method, RawHandler};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

fn reset_ledger(ctx: &mut MockContext) {
    let ledger = ctx.get_mut::<Ledger>();
    ledger.deposit(Principal::anonymous(), 3 * 10_000_000_000_000);
    ledger.load(vec![
        (mock_principals::alice(), 10_000_000_000_000),
        (mock_principals::bob(), 10_000_000_000_000),
        (mock_principals::john(), 10_000_000_000_000),
    ]);
}

/// General method to test the behaviour of fee implementation of methods.
async fn test_fee<T: CandidType + Clone, O, E: Debug>(
    response: T,
    cb: Box<dyn Fn(u64) -> Pin<Box<dyn Future<Output = Result<O, E>>>>>,
) {
    let ctx = MockContext::new()
        .with_caller(mock_principals::alice())
        .inject();

    // Consumes all
    reset_ledger(ctx);
    ctx.clear_handlers();
    ctx.use_handler(
        Method::new()
            .response(response.clone())
            .cycles_consume(2_000),
    );
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
        10_000_000_000_000
    );
}

#[async_test]
async fn wallet_call_fee() {
    use crate::cycles_wallet::*;
    test_fee(
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
    test_fee(
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
    test_fee(
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
