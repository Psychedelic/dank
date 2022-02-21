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
