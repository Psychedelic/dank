use crate::ledger::Ledger;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk::*;
use ic_cdk_macros::*;

#[derive(CandidType, Deserialize)]
struct StableStorage {
    ledger: Vec<(Principal, u64)>,
}

#[pre_upgrade]
pub fn pre_upgrade() {
    let ledger = storage::get_mut::<Ledger>().archive();

    let stable = StableStorage { ledger };

    match storage::stable_save((stable,)) {
        Ok(_) => (),
        Err(candid_err) => {
            trap(&format!(
                "An error occurred when saving to stable memory (pre_upgrade): {:?}",
                candid_err
            ));
        }
    };
}

#[post_upgrade]
pub fn post_upgrade() {
    if let Ok((stable,)) = storage::stable_restore::<(StableStorage,)>() {
        storage::get_mut::<Ledger>().load(stable.ledger);
    }
}
