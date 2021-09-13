use ic_kit::candid::CandidType;
use ic_kit::macros::*;

#[derive(CandidType)]
pub struct TokenMetaData<'a> {
    pub name: &'a str,
    pub symbol: &'a str,
    pub decimal: u8,
    pub features: Vec<&'a str>,
}

#[query]
pub fn meta() -> TokenMetaData<'static> {
    TokenMetaData {
        name: "XTC Cycles",
        symbol: "XTC",
        decimal: 12,
        features: vec!["history"],
    }
}

#[update]
fn meta_certified() -> TokenMetaData<'static> {
    meta()
}
