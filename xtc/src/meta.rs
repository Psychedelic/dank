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
        name: "Cycles",
        symbol: "XTC",
        decimal: 12,
        features: vec!["history"],
    }
}

#[update]
fn meta_certified() -> TokenMetaData<'static> {
    meta()
}

// Disabled as the `name` clashes with a similar method in the cycles wallet
// #[query]
// fn name() -> &'static str {
//     meta().name
// }

#[query]
fn symbol() -> &'static str {
    meta().symbol
}

#[query]
fn decimal() -> u8 {
    meta().decimal
}
