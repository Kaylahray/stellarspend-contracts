/// Shared validation errors for simple reusable helpers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    NegativeAmount,
    InvalidAddress,
}

/// Validates that an amount is not negative.
pub fn validate_amount(amount: i128) -> Result<(), ValidationError> {
    if amount < 0 {
        Err(ValidationError::NegativeAmount)
    } else {
        Ok(())
    }
}

/// Validates a user address string format.
///
/// Accepts classic Stellar-style prefixes (`G` account, `C` contract)
/// and only base32 characters (`A-Z`, `2-7`).
pub fn validate_user_address(address: &soroban_sdk::String) -> Result<(), ValidationError> {
    if address.len() == 0 {
        return Err(ValidationError::InvalidAddress);
    }

    let bytes = address.to_bytes();
    let prefix = bytes.get(0).unwrap_or(0);
    if prefix != b'G' && prefix != b'C' {
        return Err(ValidationError::InvalidAddress);
    }

    for b in bytes.iter() {
        let is_upper_alpha = b >= b'A' && b <= b'Z';
        let is_base32_digit = b >= b'2' && b <= b'7';
        if !is_upper_alpha && !is_base32_digit {
            return Err(ValidationError::InvalidAddress);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_amount, validate_user_address, ValidationError};
    use soroban_sdk::{Env, String};

    #[test]
    fn accepts_zero_and_positive_amounts() {
        assert_eq!(validate_amount(0), Ok(()));
        assert_eq!(validate_amount(1), Ok(()));
        assert_eq!(validate_amount(1_000_000), Ok(()));
    }

    #[test]
    fn rejects_negative_amounts() {
        assert_eq!(validate_amount(-1), Err(ValidationError::NegativeAmount));
        assert_eq!(validate_amount(-99), Err(ValidationError::NegativeAmount));
    }

    #[test]
    fn accepts_valid_user_address() {
        let env = Env::default();
        let address = String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!(validate_user_address(&address), Ok(()));
    }

    #[test]
    fn rejects_invalid_user_address() {
        let env = Env::default();

        let empty = String::from_str(&env, "");
        assert_eq!(validate_user_address(&empty), Err(ValidationError::InvalidAddress));

        let bad_prefix = String::from_str(&env, "XAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        assert_eq!(validate_user_address(&bad_prefix), Err(ValidationError::InvalidAddress));

        let bad_chars = String::from_str(&env, "GINVALID!ADDRESS");
        assert_eq!(validate_user_address(&bad_chars), Err(ValidationError::InvalidAddress));
    }
}
