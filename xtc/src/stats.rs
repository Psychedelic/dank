use crate::history::HistoryBuffer;
use ic_kit::candid::{CandidType, Nat};
use ic_kit::macros::*;
use ic_kit::{get_context, Context};
use serde::Deserialize;

#[derive(Deserialize, CandidType, Clone, Default)]
pub struct StatsDataV0 {
    supply: Nat,
    fee: Nat,
    history_events: u64,
    balance: u64,
    // Usage statistics
    transfers_count: u64,
    transfers_from_count: u64,
    approvals_count: u64,
    mints_count: u64,
    burns_count: u64,
    proxy_calls_count: u64,
    canisters_created_count: u64,
}

impl From<StatsDataV0> for StatsData {
    fn from(s: StatsDataV0) -> Self {
        StatsData {
            supply: s.supply,
            fee: s.fee,
            history_events: s.history_events,
            balance: s.balance,
            transfers_count: s.transfers_count,
            transfers_from_count: s.transfers_from_count,
            approvals_count: s.approvals_count,
            mints_count: s.mints_count,
            burns_count: s.burns_count,
            proxy_calls_count: s.proxy_calls_count,
            canisters_created_count: s.canisters_created_count,
        }
    }
}

#[derive(Deserialize, CandidType, Clone, Default)]
pub struct StatsData {
    pub supply: Nat,
    pub fee: Nat,
    pub history_events: u64,
    pub balance: u64,
    // Usage statistics
    pub transfers_count: u64,
    pub transfers_from_count: u64,
    pub approvals_count: u64,
    pub mints_count: u64,
    pub burns_count: u64,
    pub proxy_calls_count: u64,
    pub canisters_created_count: u64,
}

pub enum CountTarget {
    Transfer,
    TransferFrom,
    Approve,
    Mint,
    Burn,
    ProxyCall,
    CanisterCreated,
}

impl StatsData {
    #[inline]
    pub fn load(data: StatsData) {
        let ic = get_context();
        let stats = ic.get_mut::<StatsData>();
        *stats = data;
    }

    #[inline]
    pub fn get() -> StatsData {
        let ic = get_context();
        let stats = ic.get_mut::<StatsData>();
        stats.history_events = ic.get::<HistoryBuffer>().len() as u64;
        stats.balance = ic.balance();
        stats.clone()
    }

    #[inline]
    pub fn increment(target: CountTarget) {
        let ic = get_context();
        let stats = ic.get_mut::<StatsData>();
        match target {
            CountTarget::Transfer => stats.transfers_count += 1,
            CountTarget::TransferFrom => stats.transfers_from_count += 1,
            CountTarget::Approve => stats.approvals_count += 1,
            CountTarget::Mint => stats.mints_count += 1,
            CountTarget::Burn => stats.burns_count += 1,
            CountTarget::ProxyCall => stats.proxy_calls_count += 1,
            CountTarget::CanisterCreated => stats.canisters_created_count += 1,
        }
    }

    #[inline]
    pub fn deposit(amount: u64) {
        let ic = get_context();
        let stats = ic.get_mut::<StatsData>();
        stats.supply += amount;
    }

    #[inline]
    pub fn withdraw(amount: u64) {
        let ic = get_context();
        let stats = ic.get_mut::<StatsData>();
        stats.supply -= amount;
    }

    #[inline]
    pub fn capture_fee(amount: u64) {
        let ic = get_context();
        let stats = ic.get_mut::<StatsData>();
        stats.fee += amount;
    }
}

#[query]
fn stats() -> StatsData {
    StatsData::get()
}

#[query(name = "totalSupply")]
fn total_supply() -> Nat {
    StatsData::get().supply
}

#[query(name = "historySize")]
fn history_size() -> Nat {
    let stats_data = StatsData::get();
    Nat::from(
        stats_data.transfers_count
            + stats_data.transfers_from_count
            + stats_data.approvals_count
            + stats_data.mints_count
            + stats_data.burns_count
            + stats_data.proxy_calls_count
            + stats_data.canisters_created_count,
    )
}
