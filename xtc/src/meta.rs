use ic_cdk::export::candid::CandidType;
use ic_cdk::*;
use ic_cdk_macros::*;

#[derive(CandidType)]
struct TokenMetaData<'a> {
    name: &'a str,
    symbol: &'a str,
    decimal: u8,
    features: Vec<&'a str>,
}

#[query]
fn meta() -> TokenMetaData<'static> {
    TokenMetaData {
        name: "Dank",
        symbol: "XTC",
        decimal: 12,
        features: vec!["history"],
    }
}

#[update]
fn meta_certified() -> TokenMetaData<'static> {
    meta()
}

#[query]
fn name() -> &'static str {
    meta().name
}
