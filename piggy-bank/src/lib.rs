use ic_kit::candid::{CandidType, Nat};
use ic_kit::macros::*;
use ic_kit::*;
use serde::*;

#[derive(Deserialize, CandidType)]
struct PerformMintArgs {
    canister: Principal,
    account: Option<Principal>,
    cycles: u64,
}

#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
pub enum TxError {
    InsufficientAllowance,
    InsufficientBalance,
    ErrorOperationStyle,
    Unauthorized,
    LedgerTrap,
    ErrorTo,
    Other,
    BlockUsed,
    AmountTooSmall,
}

pub type TxReceipt = Result<Nat, TxError>;

#[update]
async fn perform_mint(args: PerformMintArgs) -> TxReceipt {
    let ic = get_context();

    let account = match args.account {
        Some(account) => account,
        None => ic.caller(),
    };

    if ic.balance() < args.cycles {
        return Err(TxError::InsufficientBalance);
    }

    match ic
        .call_with_payment(args.canister, "mint", (account, Nat::from(0)), args.cycles)
        .await
    {
        Ok((r,)) => r,
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
                    .response::<TxReceipt>(Ok(Nat::from(17)))
                    .expect_arguments((alice, Nat::from(0))),
            )
            .inject();

        assert_eq!(
            perform_mint(PerformMintArgs {
                canister: Principal::management_canister(),
                account: None,
                cycles: 300
            })
            .await,
            Ok(Nat::from(17))
        );

        MockContext::new()
            .with_caller(alice.clone())
            .with_handler(
                Method::new()
                    .expect_cycles(140)
                    .response::<TxReceipt>(Ok(Nat::from(18)))
                    .expect_arguments((bob, Nat::from(0))),
            )
            .inject();

        assert_eq!(
            perform_mint(PerformMintArgs {
                canister: Principal::management_canister(),
                account: Some(bob),
                cycles: 140
            })
            .await,
            Ok(Nat::from(18))
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
