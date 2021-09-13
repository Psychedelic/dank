use crate::fee::compute_fee;
use crate::history::{HistoryBuffer, Transaction, TransactionId, TransactionKind};
use crate::management::IsShutDown;
use crate::stats::StatsData;
use ic_kit::candid::CandidType;
use ic_kit::macros::*;
use ic_kit::{get_context, Context, Principal};
use serde::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct Ledger {
    // stores the cycle balance hold by the principal
    balances: HashMap<Principal, u64>,
}

impl Ledger {
    pub fn archive(&mut self) -> Vec<(Principal, u64)> {
        std::mem::take(&mut self.balances)
            .into_iter()
            .filter(|(_, balance)| *balance > 0)
            .collect()
    }

    pub fn load(&mut self, archive: Vec<(Principal, u64)>) {
        self.balances = archive.into_iter().collect();
        self.balances.reserve(25_000 - self.balances.len());
    }

    #[inline]
    pub fn balance(&self, account: &Principal) -> u64 {
        *(self.balances.get(account).unwrap_or(&0))
    }

    #[inline]
    pub fn deposit(&mut self, account: Principal, amount: u64) {
        StatsData::deposit(amount);
        *(self.balances.entry(account).or_default()) += amount;
    }

    #[inline]
    pub fn withdraw_erc20(
        &mut self,
        account: &Principal,
        amount: u64,
    ) -> Result<u64, ErrorDetails> {
        let balance = match self.balances.get_mut(&account) {
            Some(balance) if *balance >= amount => {
                *balance -= amount;
                *balance
            }
            _ => return Err(InsufficientBalanceError.clone()),
        };

        if balance == 0 {
            self.balances.remove(&account);
        }

        StatsData::withdraw(amount);

        Ok(balance)
    }

    #[inline]
    pub fn withdraw(&mut self, account: &Principal, amount: u64) -> Result<(), ()> {
        let balance = match self.balances.get_mut(&account) {
            Some(balance) if *balance >= amount => {
                *balance -= amount;
                *balance
            }
            _ => return Err(()),
        };

        if balance == 0 {
            self.balances.remove(&account);
        }

        StatsData::withdraw(amount);

        Ok(())
    }
}

#[derive(CandidType, Clone)]
enum APIError {
    InsufficientBalance,
    Unknown,
}

#[derive(CandidType, Clone)]
struct ErrorDetails {
    msg: &'static str,
    code: APIError,
}

static InsufficientBalanceError: ErrorDetails = ErrorDetails {
    msg: "Insufficient Balance",
    code: APIError::InsufficientBalance,
};
static UnknownError: ErrorDetails = ErrorDetails {
    msg: "Unknown",
    code: APIError::Unknown,
};

#[update]
pub async fn balance(account: Option<Principal>) -> u64 {
    let ic = get_context();
    let caller = ic.caller();
    crate::progress().await;
    let ledger = ic.get::<Ledger>();
    ledger.balance(&account.unwrap_or(caller))
}

#[derive(Deserialize, CandidType)]
pub struct TransferArguments {
    pub to: Principal,
    pub amount: u64,
    // TODO(qt3ie) Notify argument.
}

#[derive(CandidType, Debug)]
pub enum TransferError {
    InsufficientBalance,
    AmountTooLarge,
    CallFailed,
    Unknown,
}

#[update]
pub async fn transfer(args: TransferArguments) -> Result<TransactionId, TransferError> {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    crate::progress().await;

    let fee = compute_fee(args.amount);
    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.amount + fee)
        .map_err(|_| TransferError::InsufficientBalance)?;
    ledger.deposit(args.to, args.amount);

    let transaction = Transaction {
        timestamp: ic.time(),
        cycles: args.amount,
        fee,
        kind: TransactionKind::Transfer {
            from: caller,
            to: args.to,
        },
    };

    let id = ic.get_mut::<HistoryBuffer>().push(transaction);
    Ok(id)
}

#[derive(CandidType, Debug)]
pub enum MintError {
    NotSufficientLiquidity,
}

#[update]
pub async fn mint(account: Option<Principal>) -> Result<TransactionId, MintError> {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    crate::progress().await;

    let account = account.unwrap_or(caller);
    let available = ic.msg_cycles_available();
    let fee = compute_fee(available);

    if available <= fee {
        panic!("Cannot mint less than {}", fee);
    }

    let accepted = ic.msg_cycles_accept(available);
    let cycles = accepted - fee;

    let ledger = ic.get_mut::<Ledger>();
    ledger.deposit(account.clone(), cycles);

    let transaction = Transaction {
        timestamp: ic.time(),
        cycles,
        fee,
        kind: TransactionKind::Mint { to: account },
    };

    let id = ic.get_mut::<HistoryBuffer>().push(transaction);
    Ok(id)
}

#[derive(Deserialize, CandidType)]
pub struct BurnArguments {
    pub canister_id: Principal,
    pub amount: u64,
}

#[derive(CandidType, Debug)]
pub enum BurnError {
    InsufficientBalance,
    InvalidTokenContract,
    NotSufficientLiquidity,
}

#[update]
pub async fn burn(args: BurnArguments) -> Result<TransactionId, BurnError> {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    let deduced_fee = compute_fee(args.amount);
    let ledger = ic.get_mut::<Ledger>();
    ledger
        .withdraw(&caller, args.amount + deduced_fee)
        .map_err(|_| BurnError::InsufficientBalance)?;

    #[derive(CandidType)]
    struct DepositCyclesArg {
        canister_id: Principal,
    }

    let deposit_cycles_arg = DepositCyclesArg {
        canister_id: args.canister_id,
    };

    let (result, refunded) = match ic
        .call_with_payment(
            Principal::management_canister(),
            "deposit_cycles",
            (deposit_cycles_arg,),
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
                    to: args.canister_id,
                },
            };

            let id = ic.get_mut::<HistoryBuffer>().push(transaction);

            (Ok(id), refunded)
        }
        Err(_) => (
            Err(BurnError::InvalidTokenContract),
            args.amount + deduced_fee,
        ),
    };

    if refunded > 0 {
        ledger.deposit(caller, refunded);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::Ledger;
    use ic_kit::{MockContext, Principal};

    fn alice() -> Principal {
        Principal::from_text("fterm-bydaq-aaaaa-aaaaa-c").unwrap()
    }

    fn bob() -> Principal {
        Principal::from_text("ai7t5-aibaq-aaaaa-aaaaa-c").unwrap()
    }

    fn john() -> Principal {
        Principal::from_text("hozae-racaq-aaaaa-aaaaa-c").unwrap()
    }

    #[test]
    fn balance() {
        MockContext::new().inject();

        let mut ledger = Ledger::default();
        assert_eq!(ledger.balance(&alice()), 0);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);

        // Deposit should work.
        ledger.deposit(alice(), 1000);
        assert_eq!(ledger.balance(&alice()), 1000);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);

        assert!(ledger.withdraw(&alice(), 100).is_ok());
        assert_eq!(ledger.balance(&alice()), 900);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);

        assert!(ledger.withdraw(&alice(), 1000).is_err());
        assert_eq!(ledger.balance(&alice()), 900);

        ledger.deposit(alice(), 100);
        assert!(ledger.withdraw(&alice(), 1000).is_ok());
        assert_eq!(ledger.balance(&alice()), 0);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&john()), 0);
    }
}
