/// Compute the fee for the given transaction amount. Any implementation of this method
/// should guarantee that for an amount A > B, compute_fee(A) >= compute_fee(B).
pub fn compute_fee(_: u64) -> u64 {
    2_000_000_000
}
