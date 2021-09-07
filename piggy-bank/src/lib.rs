use ic_kit::candid::CandidType;
use ic_kit::macros::*;
use ic_kit::*;
use serde::*;

#[derive(Deserialize, CandidType)]
struct PerformMintArgs {
    canister: Principal,
    account: Option<Principal>,
    cycles: u64,
}

#[derive(CandidType, Deserialize, Debug)]
enum MintError {
    NotSufficientLiquidity,
}

#[update]
async fn perform_mint(args: PerformMintArgs) -> Result<u64, MintError> {
    let ic = get_context();

    let account = match args.account {
        Some(account) => account,
        None => ic.caller(),
    };

    if ic.balance() < args.cycles {
        return Err(MintError::NotSufficientLiquidity);
    }

    match ic
        .call_with_payment(args.canister, "mint", (Some(account),), args.cycles)
        .await
    {
        Ok((r,)) => Ok(r),
        Err(e) => ic.trap(&format!("Call failed with code={:?}: {}", e.0, e.1)),
    }
}

#[update]
fn balance() -> u64 {
    get_context().balance()
}

#[update]
fn get_available_cycles() -> u64 {
    get_context().msg_cycles_available()
}

#[update]
fn whoami() -> Principal {
    get_context().caller()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_test]
    async fn test_mint() {
        let alice = Principal::from_text("ai7t5-aibaq-aaaaa-aaaaa-c").unwrap();
        let bob = Principal::from_text("hozae-racaq-aaaaa-aaaaa-c").unwrap();

        MockContext::new()
            .with_caller(alice.clone())
            .with_handler(
                Method::new()
                    .expect_cycles(300)
                    .response(17u64)
                    .expect_arguments((Some(&alice),)),
            )
            .inject();

        assert_eq!(
            perform_mint(PerformMintArgs {
                canister: Principal::management_canister(),
                account: None,
                cycles: 300
            })
            .await
            .unwrap(),
            17
        );

        MockContext::new()
            .with_caller(alice.clone())
            .with_handler(
                Method::new()
                    .expect_cycles(140)
                    .response(18u64)
                    .expect_arguments((Some(&bob),)),
            )
            .inject();

        assert_eq!(
            perform_mint(PerformMintArgs {
                canister: Principal::management_canister(),
                account: Some(bob),
                cycles: 140
            })
            .await
            .unwrap(),
            18
        );
    }

    #[async_test]
    async fn test_balance() {
        MockContext::new().with_balance(1027).inject();
        assert_eq!(balance(), 1027);
    }

    #[async_test]
    async fn test_get_available_cycles() {
        MockContext::new().with_msg_cycles(1027).inject();
        assert_eq!(get_available_cycles(), 1027);
    }

    #[async_test]
    async fn test_whoami() {
        let alice = Principal::from_text("ai7t5-aibaq-aaaaa-aaaaa-c").unwrap();
        MockContext::new().with_caller(alice).inject();

        assert_eq!(whoami(), alice);
    }
}
