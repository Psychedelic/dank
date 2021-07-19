use ic_cdk::export::candid::{CandidType, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::*;

#[derive(Deserialize)]
struct PerformDepositArgs {
    canister: Principal,
    account: Option<Principal>,
    cycles: u64,
}

#[derive(CandidType, Deserialize)]
enum DepositError {
    NotSufficientLiquidity,
}

#[update]
async fn perform_deposit(args: PerformDepositArgs) -> Result<u64, DepositError> {
    let account = match args.account {
        Some(account) => account,
        None => caller(),
    };

    match api::call::call_with_payment(args.canister, "deposit", (Some(account),), args.cycles)
        .await
    {
        Ok((res,)) => res,
        Err(e) => trap(&format!("Call failed with code={:?}: {}", e.0, e.1)),
    }
}

#[update]
fn balance() -> u64 {
    api::canister_balance()
}

#[update]
fn get_available_cycles() -> u64 {
  api::call::msg_cycles_available()
}
