use crate::history::HistoryBuffer;
use crate::ledger::{Ledger, UsedBlocks, UsedMapBlocks};
use crate::management;
use crate::stats::{StatsData, StatsDataV0};
use ic_kit::candid::CandidType;
use ic_kit::macros::*;
use ic_kit::{ic, Context, Principal};
use serde::Deserialize;
use xtc_history::data::{HistoryArchive, HistoryArchiveBorrowed, HistoryArchiveV0};

#[derive(CandidType, Deserialize)]
struct StableStorageV0 {
    ledger: Vec<(Principal, u64)>,
    history: HistoryArchiveV0,
    controller: Principal,
    stats: StatsDataV0,
    used_blocks: UsedBlocks,
    used_map_blocks: UsedMapBlocks,
}

#[derive(CandidType)]
struct StableStorageBorrowed<'h> {
    ledger: Vec<(Principal, u64)>,
    history: HistoryArchiveBorrowed<'h, 'h>,
    controller: Principal,
    stats: StatsData,
    used_blocks: &'static UsedBlocks,
    used_map_blocks: &'static UsedMapBlocks,
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

    let used_blocks = ic::get_mut::<UsedBlocks>();
    let used_map_blocks = ic::get_mut::<UsedMapBlocks>();

    let stable = StableStorageBorrowed {
        ledger,
        history,
        controller,
        stats: StatsData::get(),
        used_blocks,
        used_map_blocks,
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
    ic::get_mut::<HistoryBuffer>().load(stable.history.into());
    management::Controller::load(stable.controller);
    StatsData::load(stable.stats.into());
    ic::store::<UsedBlocks>(stable.used_blocks);
    ic::store::<UsedMapBlocks>(stable.used_map_blocks);
}
