use crate::nft_ledger::NFSLedger;
use crate::types::*;

pub use ic_kit::candid::Principal;
pub use ic_kit::ic::trap;

pub fn caller() -> Principal {
    ic_kit::ic::caller()
}

pub fn ledger<'a>() -> &'a mut NFSLedger {
    ic_kit::ic::get_mut::<NFSLedger>()
}

pub fn token_level_metadata<'a>() -> &'a mut TokenLevelMetadata {
    ic_kit::ic::get_mut::<TokenLevelMetadata>()
}

pub fn expect_caller(input_principal: &Principal) {
    if (&caller() != input_principal) {
        trap("input_principal is different from caller");
    }
}
