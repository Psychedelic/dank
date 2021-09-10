/// Compute the fee for the given transaction amount. Any implementation of this method
/// should guarantee that for an amount A > B, compute_fee(A) >= compute_fee(B).
#[cfg(not(test))]
pub fn compute_fee(_: u64) -> u64 {
    2_000_000_000
}

// Used for testing.
#[cfg(test)]
pub fn compute_fee(cycles: u64) -> u64 {
    if cycles > 5_000 {
        3_500_000_000
    } else {
        2_000_000_000
    }
}
