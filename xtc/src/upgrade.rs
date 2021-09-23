use crate::history::HistoryBuffer;
use crate::ledger::Ledger;
use crate::management;
use crate::stats::{StatsData, StatsDataV0};
use ic_kit::candid::CandidType;
use ic_kit::Principal;
use ic_kit::{ic, macros::*};
use serde::Deserialize;
use xtc_history::data::{HistoryArchive, HistoryArchiveBorrowed};

#[derive(CandidType, Deserialize)]
struct StableStorageV0 {
    ledger: Vec<(Principal, u64)>,
    history: HistoryArchive,
    controller: Principal,
    stats: StatsDataV0,
}

#[derive(CandidType)]
struct StableStorageBorrowed<'h> {
    ledger: Vec<(Principal, u64)>,
    history: HistoryArchiveBorrowed<'h, 'h>,
    controller: Principal,
    stats: StatsData,
}

#[derive(CandidType, Deserialize)]
struct StableStorage {
    ledger: Vec<(Principal, u64)>,
    history: HistoryArchive,
    controller: Principal,
    stats: StatsData,
}

#[pre_upgrade]
pub fn pre_upgrade() {
    let ledger = ic::get_mut::<Ledger>().archive();
    let history = ic::get_mut::<HistoryBuffer>().archive();
    let controller = management::Controller::get_principal();

    let stable = StableStorageBorrowed {
        ledger,
        history,
        controller,
        stats: StatsData::get(),
    };

    match ic::stable_store((stable,)) {
        Ok(_) => (),
        Err(candid_err) => {
            panic!(
                "An error occurred when saving to stable memory (pre_upgrade): {:?}",
                candid_err
            );
        }
    };
}

#[post_upgrade]
pub fn post_upgrade() {
    let (stable,) =
        ic::stable_restore::<(StableStorageV0,)>().expect("Failed to read from stable storage.");
    ic::get_mut::<Ledger>().load(stable.ledger);
    ic::get_mut::<HistoryBuffer>().load(stable.history);
    management::Controller::load(stable.controller);
    StatsData::load(stable.stats.into());
}
