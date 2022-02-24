#![allow(warnings)]

mod common_types;
mod cycles_wallet;
mod fee;
mod history;
mod ledger;
mod management;
mod meta;
mod stats;
mod upgrade;
mod utils;

#[cfg(test)]
mod tests;
use crate::common_types::*;
use crate::cycles_wallet::*;
use crate::history::*;
use crate::ledger::*;
use crate::stats::*;

use ic_cdk::export::candid::export_service;
use ic_kit::candid::{candid_method, CandidType, Int, Nat};
use ic_kit::interfaces::management::WithCanisterId;
use ic_kit::macros::*;
use ic_kit::Principal;
use ledger_canister::{BlockHeight, Subaccount, Transaction};

use std::collections::HashSet;

/// Perform only one pending async task, returns whether an async call was performed
/// as the result of calling this method or not.
/// This method should only be called from updates.
///
/// Currently only the history has async tasks, but in future there might
/// be more things following this design pattern for handling tasks.
#[inline]
pub async fn progress() -> bool {
    use ic_kit::{get_context, Context};

    let ic = get_context();
    let history = ic.get_mut::<history::HistoryBuffer>();
    history.progress().await
}

#[test]
#[should_panic]
fn integer_underflow_in_release_build_test() // test integer overflow in release mode
{
    // disable compile time checks for overflow for constant expressions
    #![allow(arithmetic_overflow)]

    let u: u8 = 0 - 1;
    sink(u);
}

#[test]
#[should_panic]
fn integer_overflow_in_release_build_test() // test integer overflow in release mode
{
    // disable compile time checks for overflow for constant expressions
    #![allow(arithmetic_overflow)]

    let u: u8 = 255 + 1;
    sink(u);
}

// Declarding function called sink, that has side-effects cause by the #[no_mangle]
// attribute. A function with side-effects and the parameters will not be optimized
// away by rustc.
#[cfg(test)]
#[no_mangle]
fn sink(_: u8) {}

// cargo run --bin candid
// When run on native this prints the candid service definition of this
// canister, from the methods annotated with `candid_method` above.
//
// Note that `cargo test` calls `main`, and `export_service` (which defines
// `__export_service` in the current scope) needs to be called exactly once. So
// in addition to `not(target_arch = "wasm32")` we have a `not(test)` guard here
// to avoid calling `export_service`, which we need to call in the test below.
#[cfg(not(any(test)))]
fn main() {
    // The line below generates did types and service definition from the
    // methods annotated with `candid_method` above. The definition is then
    // obtained with `__export_service()`.
    std::print!("{}", export_candid());
}

// ---------------- CANDID -----------------------

export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}
