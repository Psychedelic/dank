use ic_cdk::export::candid::CandidType;
use ic_cdk_macros::*;

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
