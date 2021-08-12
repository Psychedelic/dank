use ic_cdk::export::candid::{CandidType, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::*;

#[derive(Deserialize, CandidType)]
struct PerformMintArgs {
    canister: Principal,
    account: Option<Principal>,
    cycles: u64,
}

#[derive(CandidType, Deserialize)]
enum MintError {
    NotSufficientLiquidity,
}

#[update]
async fn perform_mint(args: PerformMintArgs) -> Result<u64, MintError> {
    let account = match args.account {
        Some(account) => account,
        None => caller(),
    };

    match api::call::call_with_payment(args.canister, "mint", (Some(account),), args.cycles).await {
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

#[update]
fn whoami() -> Principal {
    caller()
}
