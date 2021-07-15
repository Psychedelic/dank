mod history;
mod ledger;
mod proxy;
mod upgrade;

use ic_cdk::caller;
use ic_cdk::export::Principal;
use ic_cdk_macros::*;

#[query]
fn name() -> String {
    String::from("Dank")
}

#[update]
fn whoami() -> Principal {
    caller()
}
