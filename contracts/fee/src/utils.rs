use crate::storage::MAX_FEE_BPS;

/// Calculates a fee based on the given amount and basis points (bps).
/// 
/// Formula: (amount * bps) / 10000
/// 
/// Returns `Some(fee)` on success, or `None` if an overflow occurs or bps is invalid.
pub fn compute_fee(amount: i128, bps: u32) -> Option<i128> {
    if bps > MAX_FEE_BPS {
        return None;
    }

    amount
        .checked_mul(bps as i128)?
        .checked_div(MAX_FEE_BPS as i128)
}
