use crate::common_types::{
    Operation, TxError, TxErrorLegacy, TxReceipt, TxReceiptLegacy, TxRecord,
};
use crate::fee::compute_fee;
use crate::history::{
    HistoryBuffer, Transaction, TransactionId, TransactionKind, TransactionStatus,
};
use crate::management::IsShutDown;
use crate::stats::StatsData;
use crate::utils;
use cycles_minting_canister::{
    IcpXdrConversionRateCertifiedResponse, TokensToCycles, DEFAULT_CYCLES_PER_XDR,
};
use dfn_core::api::call_with_cleanup;
use dfn_protobuf::protobuf;
use ic_kit::candid::{CandidType, Int, Nat};
use ic_kit::macros::*;
use ic_kit::{get_context, ic, ic::call, Context, Principal};
use ic_types::{CanisterId, PrincipalId};
use ledger_canister::{
    account_identifier::{AccountIdentifier, Subaccount},
    tokens::Tokens,
    BlockHeight, BlockRes, CyclesResponse, Memo, NotifyCanisterArgs, Operation as Operate,
    SendArgs,
};
use serde::*;
use std::collections::{HashMap, HashSet};
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
            _ if total_amount == 0 => return Ok(()),
            _ => return Err(TxError::InsufficientBalance),
        };

        if balance == 0 {
            self.balances.remove(&account);
        }

        StatsData::withdraw(total_amount);

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
pub async fn transfer_erc20(to: Principal, amount: Nat) -> TxReceiptLegacy {
    transfer(to, amount).await.map_err(|err| match err {
        TxError::InsufficientAllowance => TxErrorLegacy::InsufficientAllowance,
        _ => TxErrorLegacy::InsufficientBalance,
    })
}

#[update]
pub async fn transfer(to: Principal, amount: Nat) -> TxReceipt {
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

pub type UsedBlocks = HashSet<BlockHeight>;
pub type UsedMapBlocks = HashMap<BlockHeight, BlockHeight>;

#[query(name = "getBlockUsed")]
fn get_block_used() -> &'static HashSet<u64> {
    ic::get::<UsedBlocks>()
}

#[query(name = "isBlockUsed")]
fn is_block_used(block_number: BlockHeight) -> bool {
    ic::get::<UsedBlocks>().contains(&block_number)
}

#[query]
fn get_map_block_used(block_number: BlockHeight) -> Option<&'static BlockHeight> {
    ic::get::<UsedMapBlocks>().get(&block_number)
}

const ICPFEE: Tokens = Tokens::from_e8s(10000);
const MEMO_TOP_UP_CANISTER: u64 = 1347768404_u64;
const LEDGER_CANISTER_ID: CanisterId = CanisterId::from_u64(2);
const MAX_RETRY: u8 = 5;

async fn get_block_info(
    block_height: BlockHeight,
) -> Result<(AccountIdentifier, AccountIdentifier, Tokens), TxError> {
    let BlockRes(block_response) =
        call_with_cleanup(LEDGER_CANISTER_ID, "block_pb", protobuf, block_height)
            .await
            .map_err(|_| TxError::Other)?;

    let block = match block_response.ok_or(TxError::Other)? {
        Ok(encode_block) => encode_block,
        Err(e) => {
            let storage = Principal::from_text(e.to_string()).map_err(|_| TxError::Other)?;
            let storage_canister =
                CanisterId::new(PrincipalId::from(storage)).map_err(|_| TxError::Other)?;
            let BlockRes(block_response) =
                call_with_cleanup(storage_canister, "get_block_pb", protobuf, block_height)
                    .await
                    .map_err(|_| TxError::Other)?;
            block_response
                .ok_or(TxError::Other)?
                .map_err(|_| TxError::Other)?
        }
    }
    .decode()
    .map_err(|_| TxError::Other)?;

    match block.transaction.operation {
        Operate::Transfer {
            from,
            to,
            amount,
            fee: _,
        } => Ok((from, to, amount)),
        _ => {
            return Err(TxError::ErrorOperationStyle);
        }
    }
}

#[update]
pub async fn mint_by_icp(sub_account: Option<Subaccount>, block_height: BlockHeight) -> TxReceipt {
    IsShutDown::guard();

    let caller = ic::caller();

    crate::progress().await;

    let (from, to, amount) = get_block_info(block_height).await?;

    let used_blocks = ic::get_mut::<UsedBlocks>();

    // guard
    if !used_blocks.insert(block_height) {
        return Err(TxError::BlockUsed);
    }

    let caller_account = AccountIdentifier::new(ic_types::PrincipalId::from(caller), sub_account);
    let xtc_account = AccountIdentifier::new(ic_types::PrincipalId::from(ic::id()), None);

    if caller_account != from {
        used_blocks.remove(&block_height);
        return Err(TxError::Unauthorized);
    }

    if xtc_account != to {
        used_blocks.remove(&block_height);
        return Err(TxError::ErrorTo);
    }

    // ====================================================
    // 2 times fee because of "send_dfx" and "notify_dfx"
    let amount = (amount - ICPFEE).map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::AmountTooSmall
    })?;
    let amount = (amount - ICPFEE).map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::AmountTooSmall
    })?;
    // ====================================================

    let cycles_minting_canister = Principal::from_text("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap();

    // ====================================================
    // check xtc fee
    let rate = call::<_, (IcpXdrConversionRateCertifiedResponse,), _>(
        cycles_minting_canister,
        "get_icp_xdr_conversion_rate",
        (),
    )
    .await
    .map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::FetchRateFailed
    })?
    .0;

    let cycles: u64 = (TokensToCycles {
        xdr_permyriad_per_icp: rate.data.xdr_permyriad_per_icp,
        cycles_per_xdr: DEFAULT_CYCLES_PER_XDR.into(),
    })
    .to_cycles(amount)
    .into();

    let fee = compute_fee(0);
    if cycles <= fee {
        used_blocks.remove(&block_height);
        return Err(TxError::InsufficientXTCFee);
    }

    // actual user cycles
    let cycles = cycles - fee;
    // ====================================================

    // ====================================================
    // Burn
    let new_block_height = call::<_, (u64,), _>(
        Principal::from_slice(LEDGER_CANISTER_ID.as_ref()),
        "send_dfx",
        (SendArgs {
            memo: Memo(MEMO_TOP_UP_CANISTER),
            amount,
            fee: ICPFEE,
            from_subaccount: None,
            to: AccountIdentifier::new(
                ic_types::PrincipalId::from(cycles_minting_canister),
                Some(Subaccount::from(&ic_types::PrincipalId::from(ic::id()))),
            ),
            created_at_time: None,
        },),
    )
    .await
    .map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::LedgerTrap
    })?
    .0;
    // ====================================================

    // track `user transferred block` that map to `canister burned block`
    ic::get_mut::<UsedMapBlocks>().insert(block_height, new_block_height);

    // ====================================================
    // Notify - Retry until successful
    // https://github.com/dfinity/sdk/pull/1973
    let mut result: Option<CyclesResponse> = None;
    for _ in (0..MAX_RETRY) {
        match call::<_, (CyclesResponse,), _>(
            Principal::from_slice(LEDGER_CANISTER_ID.as_ref()),
            "notify_dfx",
            (NotifyCanisterArgs {
                block_height: new_block_height,
                max_fee: ICPFEE,
                from_subaccount: None,
                to_canister: ic_types::CanisterId::new(ic_types::PrincipalId::from(
                    cycles_minting_canister,
                ))
                .unwrap(),
                to_subaccount: Some(Subaccount::from(&ic_types::PrincipalId::from(ic::id()))),
            },),
        )
        .await
        {
            Ok(cycles_response) => {
                result = Some(cycles_response.0);
                break;
            }
            Err(_) => continue,
        }
    }
    let result = result.ok_or(TxError::NotifyDfxFailed)?;
    // ====================================================

    // ====================================================
    // Credit XTC
    match result {
        CyclesResponse::ToppedUp(()) => {
            ic::get_mut::<Ledger>().deposit(&caller, cycles);
            Ok(Nat::from(ic::get_mut::<HistoryBuffer>().push(
                Transaction {
                    timestamp: ic::time(),
                    cycles,
                    fee,
                    kind: TransactionKind::Mint { to: caller },
                    status: TransactionStatus::SUCCEEDED,
                },
            )))
        }
        _ => Err(TxError::UnexpectedCyclesResponse),
    }
    // ====================================================
}

#[update]
pub async fn mint_by_icp_recover(
    sub_account: Option<Subaccount>,
    block_height: BlockHeight,
    user_principal: Principal,
) -> TxReceipt {
    IsShutDown::guard();

    crate::progress().await;

    let (from, to, amount) = get_block_info(block_height).await?;

    let used_blocks = ic::get_mut::<UsedBlocks>();

    // guard
    if !used_blocks.insert(block_height) {
        return Err(TxError::BlockUsed);
    }

    let from_account =
        AccountIdentifier::new(ic_types::PrincipalId::from(user_principal), sub_account);
    let xtc_account = AccountIdentifier::new(ic_types::PrincipalId::from(ic::id()), None);

    if from_account != from {
        used_blocks.remove(&block_height);
        return Err(TxError::Unauthorized);
    }

    if xtc_account != to {
        used_blocks.remove(&block_height);
        return Err(TxError::ErrorTo);
    }

    // ====================================================
    // 2 times fee because of "send_dfx" and "notify_dfx"
    let amount = (amount - ICPFEE).map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::AmountTooSmall
    })?;
    let amount = (amount - ICPFEE).map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::AmountTooSmall
    })?;
    // ====================================================

    let cycles_minting_canister = Principal::from_text("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap();

    // ====================================================
    // check xtc fee
    let rate = call::<_, (IcpXdrConversionRateCertifiedResponse,), _>(
        cycles_minting_canister,
        "get_icp_xdr_conversion_rate",
        (),
    )
    .await
    .map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::FetchRateFailed
    })?
    .0;

    let cycles: u64 = (TokensToCycles {
        xdr_permyriad_per_icp: rate.data.xdr_permyriad_per_icp,
        cycles_per_xdr: DEFAULT_CYCLES_PER_XDR.into(),
    })
    .to_cycles(amount)
    .into();

    let fee = compute_fee(0);
    if cycles <= fee {
        used_blocks.remove(&block_height);
        return Err(TxError::InsufficientXTCFee);
    }

    // actual user cycles
    let cycles = cycles - fee;
    // ====================================================

    // ====================================================
    // Burn
    let new_block_height = call::<_, (u64,), _>(
        Principal::from_slice(LEDGER_CANISTER_ID.as_ref()),
        "send_dfx",
        (SendArgs {
            memo: Memo(MEMO_TOP_UP_CANISTER),
            amount,
            fee: ICPFEE,
            from_subaccount: None,
            to: AccountIdentifier::new(
                ic_types::PrincipalId::from(cycles_minting_canister),
                Some(Subaccount::from(&ic_types::PrincipalId::from(ic::id()))),
            ),
            created_at_time: None,
        },),
    )
    .await
    .map_err(|_| {
        used_blocks.remove(&block_height);
        TxError::LedgerTrap
    })?
    .0;
    // ====================================================

    // track `user transferred block` that map to `canister burned block`
    ic::get_mut::<UsedMapBlocks>().insert(block_height, new_block_height);

    // ====================================================
    // Notify - Retry until successful
    // https://github.com/dfinity/sdk/pull/1973
    let mut result: Option<CyclesResponse> = None;
    for _ in (0..MAX_RETRY) {
        match call::<_, (CyclesResponse,), _>(
            Principal::from_slice(LEDGER_CANISTER_ID.as_ref()),
            "notify_dfx",
            (NotifyCanisterArgs {
                block_height: new_block_height,
                max_fee: ICPFEE,
                from_subaccount: None,
                to_canister: ic_types::CanisterId::new(ic_types::PrincipalId::from(
                    cycles_minting_canister,
                ))
                .unwrap(),
                to_subaccount: Some(Subaccount::from(&ic_types::PrincipalId::from(ic::id()))),
            },),
        )
        .await
        {
            Ok(cycles_response) => {
                result = Some(cycles_response.0);
                break;
            }
            Err(_) => continue,
        }
    }
    let result = result.ok_or(TxError::NotifyDfxFailed)?;
    // ====================================================

    // ====================================================
    // Credit XTC
    match result {
        CyclesResponse::ToppedUp(()) => {
            ic::get_mut::<Ledger>().deposit(&user_principal, cycles);
            Ok(Nat::from(ic::get_mut::<HistoryBuffer>().push(
                Transaction {
                    timestamp: ic::time(),
                    cycles,
                    fee,
                    kind: TransactionKind::Mint { to: user_principal },
                    status: TransactionStatus::SUCCEEDED,
                },
            )))
        }
        _ => Err(TxError::UnexpectedCyclesResponse),
    }
    // ====================================================
}

//////////////////// END OF ERC-20 ///////////////////////

#[derive(CandidType, Debug)]
pub enum MintError {
    NotSufficientLiquidity,
}

#[update]
pub async fn mint(to: Principal, _amount: Nat) -> TxReceipt {
    IsShutDown::guard();

    let ic = get_context();
    let caller = ic.caller();

    crate::progress().await;

    let available = ic.msg_cycles_available();
    let fee = compute_fee(available);

    if available <= fee {
        panic!("Cannot mint less than {}", fee);
    }

    let accepted = ic.msg_cycles_accept(available);
    let cycles = accepted - fee;

    let ledger = ic.get_mut::<Ledger>();
    ledger.deposit(&to, cycles);

    let transaction = Transaction {
        timestamp: ic.time(),
        cycles,
        fee,
        kind: TransactionKind::Mint { to },
        status: TransactionStatus::SUCCEEDED,
    };

    Ok(Nat::from(
        ic_kit::ic::get_mut::<HistoryBuffer>().push(transaction.clone()),
    ))
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
                fee: deduced_fee,
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
