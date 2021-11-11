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

use crate::utils::tx_log;
use cap_sdk::{insert, Event};
use common_types::*;
use ic_kit::candid::Nat;
use std::convert::{From, Into, TryFrom, TryInto};

#[cfg(not(test))]
pub async fn insert_into_cap(tx_record: TxRecordExt) -> TxReceipt {
    let tx_log = tx_log();
    if let Some(failed_tx_record) = tx_log.tx_records.pop_front() {
        insert_into_cap_priv(failed_tx_record).await;
    }
    insert_into_cap_priv(tx_record).await
}

#[cfg(not(test))]
pub async fn insert_into_cap_priv(tx_record: TxRecordExt) -> TxReceipt {
    let insert_res = insert(event_into_indefinite_event(
        &TryInto::<Event>::try_into(tx_record.clone())
            .expect("unable to convert TxRecord to Event"),
    ))
    .await
    .map(|tx_id| Nat::from(tx_id))
    .map_err(|err| TxError::Other);

    if insert_res.is_err() {
        tx_log().tx_records.push_back(tx_record);
    }

    insert_res
}

#[cfg(not(test))]
pub async fn insert_legacy_into_cap<ErrorType: Clone>(
    tx_record: TxRecordExt,
    map_error: ErrorType,
) -> Result<u64, ErrorType> {
    let tx_log = tx_log();
    if let Some(failed_tx_record) = tx_log.tx_records.pop_front() {
        insert_legacy_into_cap_priv(failed_tx_record, map_error.clone()).await;
    }
    insert_legacy_into_cap_priv(tx_record, map_error).await
}

#[cfg(not(test))]
pub async fn insert_legacy_into_cap_priv<ErrorType>(
    tx_record: TxRecordExt,
    map_error: ErrorType,
) -> Result<u64, ErrorType> {
    let insert_res = insert(event_into_indefinite_event(
        &TryInto::<Event>::try_into(tx_record.clone())
            .expect("unable to convert TxRecord to Event"),
    ))
    .await
    .map_err(|err| map_error);

    if insert_res.is_err() {
        tx_log().tx_records.push_back(tx_record);
    }

    insert_res
}

#[cfg(test)]
pub async fn insert_into_cap(tx_record: TxRecordExt) -> TxReceipt {
    Ok(Nat::from(42))
}

#[cfg(test)]
pub async fn insert_legacy_into_cap<ErrorType: Clone>(
    tx_record: TxRecordExt,
    map_error: ErrorType,
) -> Result<u64, ErrorType> {
    Ok(42)
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
