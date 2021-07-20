use crate::history::HistoryBuffer;
use ic_cdk::export::candid::{CandidType, Nat};
use ic_cdk::*;
use ic_cdk_macros::*;
use serde::Deserialize;

#[derive(Deserialize, CandidType, Clone)]
pub struct StatsData {
    supply: Nat,
    history_events: u64,
    balance: u64,
    // Usage statistics
    transfers_count: u64,
    deposits_count: u64,
    withdraw_count: u64,
    proxy_calls_count: u64,
    canisters_created_count: u64,
}

impl Default for StatsData {
    fn default() -> Self {
        StatsData {
            supply: Nat::from(0),
            history_events: 0,
            balance: 0,
            transfers_count: 0,
            deposits_count: 0,
            withdraw_count: 0,
            proxy_calls_count: 0,
            canisters_created_count: 0,
        }
    }
}

pub enum CountTarget {
    Transfer,
    Deposit,
    Withdraw,
    ProxyCall,
    CanisterCreated,
}

impl StatsData {
    #[inline]
    pub fn load(data: StatsData) {
        let stats = storage::get_mut::<StatsData>();
        *stats = data;
    }

    #[inline]
    pub fn get() -> StatsData {
        let stats = storage::get_mut::<StatsData>();
        stats.history_events = storage::get::<HistoryBuffer>().len() as u64;
        stats.balance = api::canister_balance();
        stats.clone()
    }

    #[inline]
    pub fn increment(target: CountTarget) {
        let stats = storage::get_mut::<StatsData>();
        match target {
            CountTarget::Transfer => stats.transfers_count += 1,
            CountTarget::Deposit => stats.deposits_count += 1,
            CountTarget::Withdraw => stats.withdraw_count += 1,
            CountTarget::ProxyCall => stats.proxy_calls_count += 1,
            CountTarget::CanisterCreated => stats.canisters_created_count += 1,
        }
    }

    #[inline]
    pub fn deposit(amount: u64) {
        let stats = storage::get_mut::<StatsData>();
        stats.supply += amount;
    }

    #[inline]
    pub fn withdraw(amount: u64) {
        let stats = storage::get_mut::<StatsData>();
        stats.supply -= amount;
    }
}

#[query]
fn stats() -> StatsData {
    StatsData::get()
}
