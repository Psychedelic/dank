use crate::common_types::TxLog;
use ic_kit::candid::{Int, Nat};
use num_bigint::Sign;
use std::convert::TryFrom;

pub fn convert_nat_to_u64(num: Nat) -> Result<u64, String> {
    let u64_digits = num.0.to_u64_digits();

    match u64_digits.len() {
        0 => Ok(0),
        1 => Ok(u64_digits[0]),
        _ => Err("Nat -> Nat64 conversion failed".to_string()),
    }
}

pub fn convert_nat_to_u32(num: Nat) -> Result<u32, String> {
    let u32_digits = num.0.to_u32_digits();
    match u32_digits.len() {
        0 => Ok(0),
        1 => Ok(u32_digits[0]),
        _ => Err("Nat -> Nat32 conversion failed".to_string()),
    }
}

pub fn convert_int_to_u64(num: Int) -> Result<u64, String> {
    let u64_digits = num.0.to_u64_digits();

    if let Sign::Minus = u64_digits.0 {
        return Err("negative number cannot be converted to u64".to_string());
    }

    match u64_digits.1.len() {
        0 => Ok(0),
        1 => Ok(u64_digits.1[0]),
        _ => Err("Int -> Nat64 conversion failed".to_string()),
    }
}

pub fn tx_log<'a>() -> &'a mut TxLog {
    ic_kit::ic::get_mut::<TxLog>()
}
