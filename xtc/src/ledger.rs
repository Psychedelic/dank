use crate::common_types::{Operation, TxError, TxReceipt, TxRecord};
use crate::fee::compute_fee;
use crate::history::{
    HistoryBuffer, Transaction, TransactionId, TransactionKind, TransactionStatus,
};
use crate::management::IsShutDown;
use crate::meta::meta;
use crate::stats::StatsData;
use crate::utils;
use ic_kit::candid::{CandidType, Int, Nat};
use ic_kit::macros::*;
use ic_kit::{get_context, Context, Principal};
use serde::*;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Default)]
pub struct Ledger {
    // stores the cycle balance hold by the principal
    balances: HashMap<Principal, u64>,

    // stores the allowances, approving Principal -> spender principal -> cycle balanace
    allowances: HashMap<(Principal, Principal), u64>,
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
    fn cleanup_allowances(&mut self, allower: &Principal, spender: &Principal) {
        self.allowances.remove(&(*allower, *spender));
    }

    /// 1. Allower can allow more money to Spender than Allower's internal balance
    /// 2. Calling alowances twice replaces the previous allowance
    /// 3. Calling allownces with zero amount clears the allowance from the internal
    ///    map.
    #[inline]
    pub fn approve(
        &mut self,
        allower: &Principal,
        spender: &Principal,
        amount: u64,
        fee: u64,
    ) -> Result<(), TxError> {
        assert_ne!(
            allower, spender,
            "allower and spender users cannot be the same"
        );
        if self.balance(&allower) < fee {
            return Err(TxError::InsufficientBalance);
        }

        self.withdraw_erc20(&allower, 0, fee)?;

        if amount == 0 {
            self.cleanup_allowances(allower, spender);
        } else {
            // the allower will pay for the future transferFrom fees, so the total allowed amount equals to amount + fee
            *(self.allowances.entry((*allower, *spender)).or_default()) = amount + fee;
        }

        Ok(())
    }

    #[inline]
    pub fn allowance(&self, allower: &Principal, spender: &Principal) -> u64 {
        *self.allowances.get(&(*allower, *spender)).unwrap_or(&0)
    }

    /// 1. The fee is deducted from the caller's balance as opposed to the allower balance.
    /// This fee deduction is necessary to prevent attacks, when an attacker
    /// can initiate multiple small transfer_from to drain the allower's entire
    /// balance as fee payment.
    /// 2. The allowance is decreased by the transferred amount.
    /// 3. Early checks on balance amount and allowance are done to make sure
    /// the subsequent state updates cannot fail.
    /// 4 No need to check if allower==caller, as it is already checked in approve
    /// function.
    #[inline]
    pub fn transfer_from(
        &mut self,
        caller: &Principal,
        allower: &Principal,
        spender: &Principal,
        amount: u64,
        fee: u64,
    ) -> Result<(), TxError> {
        assert_ne!(amount, 0, "transfer amount cannot be zero");

        let total_amount = amount + fee;
        let allowance = self.allowance(allower, caller);
        if allowance < total_amount {
            return Err(TxError::InsufficientAllowance);
        }

        if self.balance(&allower) < total_amount {
            return Err(TxError::InsufficientBalance);
        }

        self.approve(allower, caller, allowance - total_amount, 0);
        self.withdraw_erc20(&allower, 0, fee);
        self.transfer(allower, spender, amount, 0)?;

        Ok(())
    }

    /// 1. Early checks on balance amount is done in withdrawErc20 to make sure
    /// the transfer will be successful, as there will be no refund of fees.
    #[inline]
    pub fn transfer(
        &mut self,
        from: &Principal,
        to: &Principal,
        amount: u64,
        fee: u64,
    ) -> Result<(), TxError> {
        assert_ne!(amount, 0, "transfer amount cannot be zero");
        assert_ne!(from, to, "from and to users cannot be the same");
        self.withdraw_erc20(from, amount, fee)?;
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
    pub fn withdraw_erc20(
        &mut self,
        account: &Principal,
        amount: u64,
        fee: u64,
    ) -> Result<(), TxError> {
        let total_amount = fee + amount;

        let balance = match self.balances.get_mut(&account) {
            Some(balance) if *balance >= total_amount => {
                *balance -= total_amount;
                *balance
            }
            _ if amount + fee == 0 => return Ok(()),
            _ => return Err(TxError::InsufficientBalance),
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

//////////////////// BEGIN OF ERC-20 ///////////////////////

#[query(name=balanceOf)]
pub async fn balance_of(account: Principal) -> Nat {
    let ledger = ic_kit::get_context().get::<Ledger>();
    Nat::from(ledger.balance(&account))
}

#[derive(Deserialize, CandidType)]
pub struct TransferArguments {
    pub to: Principal,
    pub amount: u64,
}

#[derive(CandidType, Debug)]
pub enum TransferError {
    InsufficientBalance,
    AmountTooLarge,
    CallFailed,
    Unknown,
}

#[query]
pub async fn allowance(from: Principal, to: Principal) -> Nat {
    return get_context().get::<Ledger>().allowance(&from, &to).into();
}

#[update]
pub async fn approve(to: Principal, amount: Nat) -> TxReceipt {
    IsShutDown::guard();
    use ic_cdk::export::candid;
    let caller = ic_kit::ic::caller();

    crate::progress().await;

    let ledger = ic_kit::ic::get_mut::<Ledger>();
    let amount_u64: u64 =
        utils::convert_nat_to_u64(amount).expect("Amount cannot be represented as u64");
    let fee = compute_fee(amount_u64);

    ledger.approve(&caller, &to, amount_u64, fee)?;

    let transaction = Transaction {
        timestamp: ic_kit::ic::time(),
        cycles: amount_u64,
        fee,
        kind: TransactionKind::Approve {
            from: caller,
            to: to,
        },
        status: TransactionStatus::SUCCEEDED,
    };

    Ok(Nat::from(
        ic_kit::ic::get_mut::<HistoryBuffer>().push(transaction.clone()),
    ))
}

#[update(name=transferErc20)]
pub async fn transfer_erc20(to: Principal, amount: Nat) -> TxReceipt {
    IsShutDown::guard();

    let caller = ic_kit::ic::caller();

    crate::progress().await;

    let ledger = ic_kit::ic::get_mut::<Ledger>();
    let amount_u64: u64 =
        utils::convert_nat_to_u64(amount).expect("transfer failed - unable to convert amount");
    let fee = compute_fee(amount_u64);
    ledger.transfer(&caller, &to, amount_u64, fee)?;

    let transaction = Transaction {
        timestamp: ic_kit::ic::time(),
        cycles: amount_u64,
        fee,
        kind: TransactionKind::Transfer {
            from: caller,
            to: to,
        },
        status: TransactionStatus::SUCCEEDED,
    };

    Ok(Nat::from(
        ic_kit::ic::get_mut::<HistoryBuffer>().push(transaction.clone()),
    ))
}

#[update(name=transferFrom)]
pub async fn transfer_from(from: Principal, to: Principal, amount: Nat) -> TxReceipt {
    IsShutDown::guard();

    let caller = ic_kit::ic::caller();

    crate::progress().await;

    let ledger = ic_kit::ic::get_mut::<Ledger>();
    let amount_u64: u64 =
        utils::convert_nat_to_u64(amount).expect("transfer failed - unable to convert amount");
    let fee = compute_fee(amount_u64);
    ledger.transfer_from(&caller, &from, &to, amount_u64, fee)?;

    let transaction = Transaction {
        timestamp: ic_kit::ic::time(),
        cycles: amount_u64,
        fee,
        kind: TransactionKind::TransferFrom {
            caller: caller,
            from: from,
            to: to,
        },
        status: TransactionStatus::SUCCEEDED,
    };

    Ok(Nat::from(
        ic_kit::ic::get_mut::<HistoryBuffer>().push(transaction.clone()),
    ))
}

//////////////////// END OF ERC-20 ///////////////////////

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
        status: TransactionStatus::SUCCEEDED,
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
        status: TransactionStatus::SUCCEEDED,
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

    match ic
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

            if refunded > 0 {
                ledger.deposit(&caller, refunded);
            }

            let id = ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles,
                fee: actual_fee,
                kind: TransactionKind::Burn {
                    from: caller.clone(),
                    to: args.canister_id,
                },
                status: TransactionStatus::SUCCEEDED,
            });

            Ok(id)
        }
        Err(_) => {
            ledger.deposit(&caller, args.amount);

            ic.get_mut::<HistoryBuffer>().push(Transaction {
                timestamp: ic.time(),
                cycles: 0,
                fee: args.amount,
                kind: TransactionKind::Burn {
                    from: caller.clone(),
                    to: args.canister_id,
                },
                status: TransactionStatus::FAILED,
            });
            Err(BurnError::InvalidTokenContract)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Ledger;
    use super::TxError;
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
    #[should_panic]
    fn approval_to_self() {
        let mut ledger = Ledger::default();

        // alice tries to approve herself
        ledger.approve(&alice(), &alice(), 1000, 0);
    }

    #[test]
    #[should_panic]
    fn transfer_from_zero_amount() {
        let mut ledger = Ledger::default();

        ledger.approve(&alice(), &bob(), 1000, 0);
        ledger.transfer_from(&bob(), &alice(), &bob(), 0, 0);
    }

    #[test]
    fn approve_and_allowances() {
        MockContext::new().inject();

        let mut ledger = Ledger::default();

        // empty ledger has zero allowance
        assert_eq!(ledger.allowance(&alice(), &bob()), 0);

        // inserting non-zero into empty ledger and read back the allowance
        ledger = Ledger::default();
        assert_eq!(ledger.approve(&alice(), &bob(), 1000, 0), Ok(()));
        assert_eq!(ledger.allowances.get(&(alice(), bob())).unwrap(), &1000);
        assert_eq!(ledger.allowance(&alice(), &bob()), 1000);

        // overriding allowance with non-zero and read back the allowance
        ledger = Ledger::default();
        assert_eq!(ledger.approve(&alice(), &bob(), 1000, 0), Ok(()));
        assert_eq!(ledger.approve(&alice(), &bob(), 2000, 0), Ok(()));
        assert_eq!(ledger.allowances.get(&(alice(), bob())).unwrap(), &2000);
        assert_eq!(ledger.allowance(&alice(), &bob()), 2000);

        // overriding allowance with zero and read back the allowance
        ledger = Ledger::default();
        assert_eq!(ledger.approve(&alice(), &bob(), 1000, 0), Ok(()));
        assert_eq!(ledger.approve(&alice(), &bob(), 0, 0), Ok(()));
        // allowance removed from the ledger
        assert!(ledger.allowances.get(&(alice(), bob())).is_none());
        assert!(ledger.allowances.is_empty());
        // allowance returns zero
        assert_eq!(ledger.allowance(&alice(), &bob()), 0);

        // alice approve more than one person
        ledger = Ledger::default();
        assert_eq!(ledger.approve(&alice(), &bob(), 1000, 0), Ok(()));
        assert_eq!(ledger.approve(&alice(), &charlie(), 2000, 0), Ok(()));
        assert_eq!(ledger.allowance(&alice(), &bob()), 1000);
        assert_eq!(ledger.allowance(&alice(), &charlie()), 2000);
        // remove bob's allowance, charlie still has his
        assert_eq!(ledger.approve(&alice(), &bob(), 0, 0), Ok(()));
        assert_eq!(ledger.allowance(&alice(), &bob()), 0);
        assert_eq!(ledger.allowance(&alice(), &charlie()), 2000);
        //bob is removed from the allowances map
        assert!(ledger.allowances.get(&(alice(), bob())).is_none());
    }

    #[test]
    fn approve_and_transfer_no_fees() {
        MockContext::new().inject();

        // alice has less balance than she approved bob to retrieve
        let mut ledger = Ledger::default();
        ledger.deposit(&alice(), 500);
        assert_eq!(ledger.approve(&alice(), &bob(), 1000, 0), Ok(()));
        assert_eq!(ledger.balance(&alice()), 500);
        assert_eq!(ledger.balance(&bob()), 0);
        // charlie tries to initiate the transfer from alice to bob
        assert_eq!(
            ledger
                .transfer_from(&charlie(), &alice(), &bob(), 400, 0)
                .unwrap_err(),
            TxError::InsufficientAllowance
        );
        assert_eq!(
            ledger.transfer_from(&bob(), &alice(), &charlie(), 400, 0),
            Ok(())
        );
        // alowances changed
        assert_eq!(ledger.allowance(&alice(), &bob()), 600);
        // balances changed
        assert_eq!(ledger.balance(&alice()), 100);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&charlie()), 400);
        // bob tries withdrawing all his allowance, but alice doesn't have enough money
        assert_eq!(
            ledger
                .transfer_from(&bob(), &alice(), &charlie(), 600, 0)
                .unwrap_err(),
            TxError::InsufficientBalance
        );

        // alice has more balance then, she approved bob to retrieve
        let mut ledger = Ledger::default();
        ledger.deposit(&alice(), 1000);
        assert_eq!(ledger.approve(&alice(), &bob(), 500, 0), Ok(()));
        assert_eq!(ledger.balance(&alice()), 1000);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&charlie()), 0);
        // bob tries to retrieve more than his allowance
        assert_eq!(
            ledger
                .transfer_from(&bob(), &alice(), &charlie(), 600, 0)
                .unwrap_err(),
            TxError::InsufficientAllowance
        );
        // alowances didn't change
        assert_eq!(ledger.allowance(&alice(), &bob()), 500);
        // balances didn't change
        assert_eq!(ledger.balance(&alice()), 1000);
        assert_eq!(ledger.balance(&bob()), 0);
        assert_eq!(ledger.balance(&charlie()), 0);
    }

    #[test]
    fn approve_and_transfer_with_fees() {
        MockContext::new().inject();

        let mut ledger = Ledger::default();
        ledger.deposit(&alice(), 500);
        assert_eq!(ledger.approve(&alice(), &bob(), 1000, 10), Ok(()));
        assert_eq!(ledger.balance(&alice()), 490);
        assert_eq!(ledger.balance(&bob()), 0);
        // the actual approved amount contains the fees
        assert_eq!(ledger.allowance(&alice(), &bob()), 1010);
        assert_eq!(
            ledger.transfer_from(&bob(), &alice(), &bob(), 400, 10),
            Ok(())
        );

        // bob received the right amount
        assert_eq!(ledger.balance(&bob()), 400);
        // alice balance decreased with the value and with the fee
        ledger.deposit(&alice(), 90);
        // the allowance decreased with the value and with the fee
        assert_eq!(ledger.allowance(&alice(), &bob()), 600);
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

#[update]
pub async fn balance(account: Option<Principal>) -> u64 {
    let ic = get_context();
    let caller = ic.caller();
    crate::progress().await;
    let ledger = ic.get::<Ledger>();
    ledger.balance(&account.unwrap_or(caller))
}
