use crate::history::HistoryBuffer;
use ic_kit::candid::{CandidType, Nat};
use ic_kit::macros::*;
use ic_kit::{get_context, Context};
use serde::Deserialize;

#[derive(Deserialize, CandidType, Clone, Default)]
pub struct StatsData {
    supply: Nat,
    history_events: u64,
    balance: u64,
    // Usage statistics
    transfers_count: u64,
    mints_count: u64,
    burns_count: u64,
    proxy_calls_count: u64,
    canisters_created_count: u64,
}

pub enum CountTarget {
    Transfer,
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
}

#[query]
fn stats() -> StatsData {
    StatsData::get()
}
