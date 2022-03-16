use crate::common_types::Metadata;
use crate::fee::compute_fee;
use crate::stats::StatsData;
use ic_kit::macros::*;
use ic_kit::{
    candid::{candid_method, CandidType, Nat},
    ic,
};

#[query(name = "getMetadata")]
#[candid_method(query, rename = "getMetadata")]
pub fn get_metadata() -> Metadata<'static> {
    Metadata {
        decimals: 12,
        fee: Nat::from(compute_fee(0)),
        logo: "data:image/png;base64,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",
        name: "Special Drawing Rights",
        owner: ic::id(),
        symbol: "SDR",
        totalSupply: StatsData::get().supply
    }
}

#[query(name=nameErc20)]
#[candid_method(query, rename = "nameErc20")]
fn name_erc20() -> &'static str {
    name()
}

#[query]
#[candid_method(query)]
fn name() -> &'static str {
    get_metadata().name
}

#[query]
#[candid_method(query)]
fn symbol() -> &'static str {
    get_metadata().symbol
}

#[query]
#[candid_method(query)]
pub async fn decimals() -> u8 {
    get_metadata().decimals
}

#[query]
#[candid_method(query)]
fn logo() -> &'static str {
    get_metadata().logo
}

#[query]
#[candid_method(query)]
fn version() -> &'static str {
    "0.0.1"
}
