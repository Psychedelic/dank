use crate::fee::compute_fee;
use crate::history::{HistoryBuffer, Transaction, TransactionId, TransactionKind};
use crate::management::IsShutDown;
use crate::meta::meta;
use crate::stats::StatsData;
use ic_kit::candid::{CandidType, Nat};
use ic_kit::macros::*;
use ic_kit::{get_context, Context, Principal};
use serde::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct Ledger {
    // stores the cycle balance hold by the principal
    balances: HashMap<Principal, u64>,

    // stores the allowances, approving Principal -> spender principal -> cycle balanace
    allowances: HashMap<Principal, HashMap<Principal, u64>>,
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
    fn cleanupAllowances(&mut self, allower: &Principal, spender: &Principal) {
        if let Some(spender_to_amount) = self.allowances.get_mut(&allower) {
            spender_to_amount.remove(&spender);
            if spender_to_amount.is_empty() {
                self.allowances.remove(allower);
            }
        }
    }

    #[inline]
    pub fn approve(&mut self, allower: &Principal, spender: &Principal, amount: u64) {
        if amount == 0 {
            self.cleanupAllowances(allower, spender);
        } else {
            *(self
                .allowances
                .entry(*allower)
                .or_default()
                .entry(*spender)
                .or_default()) = amount;
        }
    }

    /***************************************************************************

        1. Allower can allow more money to Spender than Allower's internal balance
        2. Calling alowances twice replaces the previous allowance
        3. Calling allownces with zero amount clears the allowance from the internal
           map.

    ***************************************************************************/

    #[inline]
    pub fn allowances(&self, allower: &Principal, spender: &Principal) -> u64 {
        *self
            .allowances
            .get(&allower)
            .map(|spender_to_amount| spender_to_amount.get(&spender))
            .flatten()
            .unwrap_or(&0)
    }

    /***************************************************************************

        This method is allowed to violate the invariants of the ledger upon return:
        for example by changing only the allowances and return an error without
        changing the balance. The callers of this functions might use trap
        to revert the state of the ledger.

    ***************************************************************************/

    #[inline]
    pub fn transferFrom(
        &mut self,
        allower: &Principal,
        spender: &Principal,
        amount: u64,
    ) -> Result<(), ErrorDetails> {
        let allowance = self.allowances(allower, spender);
        if allowance < amount {
            return Err(InsufficientAllowanceError.clone());
        }

        self.approve(allower, spender, allowance - amount);

        self.transfer(allower, spender, amount)?;

        Ok(())
    }

    #[inline]
    pub fn transfer(
        &mut self,
        from: &Principal,
        to: &Principal,
        amount: u64,
    ) -> Result<(), ErrorDetails> {
        self.withdrawErc20(from, amount)?;
        self.deposit(to, amount);
        Ok(())
    }

    #[inline]
    pub fn balance(&self, account: &Principal) -> u64 {
        *(self.balances.get(account).unwrap_or(&0))
    }

    #[inline]
    pub fn deposit(&mut self, account: &Principal, amount: u64) {
        StatsData::deposit(amount);
        *(self.balances.entry(*account).or_default()) += amount;
    }

    #[inline]
    pub fn withdrawErc20(&mut self, account: &Principal, amount: u64) -> Result<(), ErrorDetails> {
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

        Ok(())
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

#[derive(CandidType, Clone, Debug, PartialEq)]
pub enum APIError {
    InsufficientBalance,
    InsufficientAllowance,
    Unknown,
}

#[derive(CandidType, Clone, Debug)]
pub struct ErrorDetails {
    msg: &'static str,
    code: APIError,
}

static InsufficientBalanceError: ErrorDetails = ErrorDetails {
    msg: "Insufficient Balance",
    code: APIError::InsufficientBalance,
};

static InsufficientAllowanceError: ErrorDetails = ErrorDetails {
    msg: "Insufficient Allowance",
    code: APIError::InsufficientAllowance,
};

static UnknownError: ErrorDetails = ErrorDetails {
    msg: "Unknown",
    code: APIError::Unknown,
};

// Disabled as the `name` clashes with a similar method in the cycles wallet
// #[query]
// fn name() -> &'static str {
//     meta().name
// }

#[query]
fn symbol() -> &'static str {
    meta().symbol
}

#[query]
fn decimal() -> u8 {
    meta().decimal
}

#[query]
fn totalSupply() -> Nat {
    StatsData::get().supply
}

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
    ledger.deposit(&args.to, args.amount);

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
    ledger.deposit(&account, cycles);

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
        ledger.deposit(&caller, refunded);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::APIError;
    use super::Ledger;
    use ic_kit::{MockContext, Principal};

    fn alice() -> Principal {
        Principal::from_text("fterm-bydaq-aaaaa-aaaaa-c").unwrap()
    }

    fn bob() -> Principal {
        Principal::from_text("ai7t5-aibaq-aaaaa-aaaaa-c").unwrap()
    }

    fn charlie() -> Principal {
        Principal::from_text("hozae-racaq-aaaaa-aaaaa-c").unwrap()
    }

    #[test]
    fn approve_and_allowances() {
        MockContext::new().inject();

        let mut ledger = Ledger::default();

        // empty ledger has zero allowance
        assert_eq!(ledger.allowances(&alice(), &bob()), 0);

        // inserting non-zero into empty ledger and read back the allowance
        ledger = Ledger::default();
        ledger.approve(&alice(), &bob(), 1000);
        assert_eq!(
            ledger
                .allowances
                .get(&alice())
                .unwrap()
                .get(&bob())
                .unwrap(),
            &1000
        );
        assert_eq!(ledger.allowances(&alice(), &bob()), 1000);

        // overriding allowance with non-zero and read back the allowance
        ledger = Ledger::default();
        ledger.approve(&alice(), &bob(), 1000);
        ledger.approve(&alice(), &bob(), 2000);
        assert_eq!(
            ledger
                .allowances
                .get(&alice())
                .unwrap()
                .get(&bob())
                .unwrap(),
            &2000
        );
        assert_eq!(ledger.allowances(&alice(), &bob()), 2000);

        // overriding allowance with zero and read back the allowance
        ledger = Ledger::default();
        ledger.approve(&alice(), &bob(), 1000);
        ledger.approve(&alice(), &bob(), 0);
        // allowance removed from the ledger
        assert!(ledger.allowances.get(&alice()).is_none());
        assert!(ledger.allowances.is_empty());
        // allowance returns zero
        assert_eq!(ledger.allowances(&alice(), &bob()), 0);

        // alice approve more than one person
        ledger = Ledger::default();
        ledger.approve(&alice(), &bob(), 1000);
        ledger.approve(&alice(), &charlie(), 2000);
        assert_eq!(ledger.allowances(&alice(), &bob()), 1000);
        assert_eq!(ledger.allowances(&alice(), &charlie()), 2000);
        // remove bob's allowance, charlie still has his
        ledger.approve(&alice(), &bob(), 0);
        assert_eq!(ledger.allowances(&alice(), &bob()), 0);
        assert_eq!(ledger.allowances(&alice(), &charlie()), 2000);
        //bob is removed from the allowances map
        assert!(ledger
            .allowances
            .get(&alice())
            .unwrap()
            .get(&bob())
            .is_none());
    }

    #[test]
    fn approve_and_transfer() {
        MockContext::new().inject();

        // alice has less balance than she approved bob to retrieve
        let mut ledger = Ledger::default();
        ledger.deposit(&alice(), 500);
        ledger.approve(&alice(), &bob(), 1000);
        assert_eq!(ledger.balance(&alice()), 500);
        assert_eq!(ledger.balance(&bob()), 0);
        ledger.transferFrom(&alice(), &bob(), 400);
        // alowances changed
        assert_eq!(ledger.allowances(&alice(), &bob()), 600);
        // balances changed
        assert_eq!(ledger.balance(&alice()), 100);
        assert_eq!(ledger.balance(&bob()), 400);
        // bob tries withdrawing all his allowance, but alice doesn't have enough money
        assert_eq!(
            ledger.transferFrom(&alice(), &bob(), 600).unwrap_err().code,
            APIError::InsufficientBalance
        );

        // alice has more balance then, she approved bob to retrieve
        let mut ledger = Ledger::default();
        ledger.deposit(&alice(), 1000);
        ledger.approve(&alice(), &bob(), 500);
        assert_eq!(ledger.balance(&alice()), 1000);
        assert_eq!(ledger.balance(&bob()), 0);
        //ledger.transferFrom(&alice(), &bob(), 600);
        assert_eq!(
            ledger.transferFrom(&alice(), &bob(), 600).unwrap_err().code,
            APIError::InsufficientAllowance
        );
        // alowances didn't change
        assert_eq!(ledger.allowances(&alice(), &bob()), 500);
        // balances didn't change
        assert_eq!(ledger.balance(&alice()), 1000);
        assert_eq!(ledger.balance(&bob()), 0);
    }

    #[test]
    fn balance() {
        MockContext::new().inject();

        let mut ledger = Ledger::default();
        assert_eq!(ledger.balance(&alice()), 0);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&charlie()), 0);

        // Deposit should work.
        ledger.deposit(&alice(), 1000);
        assert_eq!(ledger.balance(&alice()), 1000);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&charlie()), 0);

        assert!(ledger.withdraw(&alice(), 100).is_ok());
        assert_eq!(ledger.balance(&alice()), 900);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&charlie()), 0);

        assert!(ledger.withdraw(&alice(), 1000).is_err());
        assert_eq!(ledger.balance(&alice()), 900);

        ledger.deposit(&alice(), 100);
        assert!(ledger.withdraw(&alice(), 1000).is_ok());
        assert_eq!(ledger.balance(&alice()), 0);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&charlie()), 0);
    }
}
