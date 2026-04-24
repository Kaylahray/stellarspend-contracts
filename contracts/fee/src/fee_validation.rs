use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FeeValidationError {
    FeeTooLow = 1,
    FeeTooHigh = 2,
}

pub fn validate_fee(fee: i128, min_fee: i128, max_fee: i128) -> Result<(), FeeValidationError> {
    if fee < min_fee {
        return Err(FeeValidationError::FeeTooLow);
    }
    if fee > max_fee {
        return Err(FeeValidationError::FeeTooHigh);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_fee() {
        assert!(validate_fee(50, 10, 100).is_ok());
    }

    #[test]
    fn test_fee_too_low() {
        assert_eq!(validate_fee(5, 10, 100), Err(FeeValidationError::FeeTooLow));
    }

    #[test]
    fn test_fee_too_high() {
        assert_eq!(validate_fee(200, 10, 100), Err(FeeValidationError::FeeTooHigh));
    }
}
