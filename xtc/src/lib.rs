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

// TODO(qti3e): Fix this mess when kit v0.5 is out. there shouldn't be any unsafe code.
use crate::fee::compute_fee;
use crate::ledger::Ledger;
use ic_kit::ic;
use ic_kit::macros::inspect_message;
use ic_kit_sys::ic0;

#[inspect_message]
fn inspect_message() {
    // get the raw message size
    let message_size = unsafe { ic0::msg_arg_data_size() as usize };

    let method_name = ic_cdk::api::call::method_name();

    // exception: wallet_call should actually accept large payload, but only if user has
    // the fee to pay.
    if method_name == "wallet_call" {
        // fee is currently constant, but let's pass 100B as min fee that should be available.
        let fee = compute_fee(100_000_000_000);
        let ledger = ic::get_mut::<Ledger>();
        let caller = ic_cdk::caller();
        let balance = ledger.balance(&caller);

        // accept and don't continue.
        if balance > fee {
            ic_cdk::api::call::accept_message();
            return;
        }
    }

    // only accept messages with a payload smaller than 250kb.
    // based on our candid the largest fixed size payload we can accept is only 2 principals
    // encode(opt principal, opt principal) -> 83 bytes
    // encode(opt principal, opt principal, opt principal) -> 119 bytes
    // the only unsized type used is our good friend `nat`, which is dynamically sized
    // but supposing a u128 is only legal, it is 16 bytes, for nat.
    // (nat, nat) -> ~40bytes
    // here we allow an error margin, and reject messages larger than 500bytes.
    if message_size > 500 {
        return;
    }

    ic_cdk::api::call::accept_message();
}

// inspect message mess ->

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
