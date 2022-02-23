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
        logo: "data:image/jpeg;base64,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",
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
