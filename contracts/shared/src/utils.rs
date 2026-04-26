use soroban_sdk::{Env, Symbol};

/// Shared validation errors for simple reusable helpers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    NegativeAmount,
}

/// Validates that an amount is not negative.
pub fn validate_amount(amount: i128) -> Result<(), ValidationError> {
    if amount < 0 {
        Err(ValidationError::NegativeAmount)
    } else {
        Ok(())
    }
}

/// Increment a counter in storage and return the new value
pub fn increment_counter(env: &Env, counter_key: &Symbol) -> u64 {
    let mut counter: u64 = env
        .storage()
        .persistent()
        .get(counter_key)
        .unwrap_or(0);
    
    counter += 1;
    env.storage()
        .persistent()
        .set(counter_key, &counter);
    
    counter
}

#[cfg(test)]
mod tests {
    use super::{validate_amount, ValidationError, increment_counter};
    use soroban_sdk::{Env, Symbol};

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
    fn increment_counter_works() {
        let env = Env::default();
        let counter_key = Symbol::new(&env, "test_counter");
        
        // First increment should return 1
        assert_eq!(increment_counter(&env, &counter_key), 1);
        
        // Second increment should return 2
        assert_eq!(increment_counter(&env, &counter_key), 2);
        
        // Third increment should return 3
        assert_eq!(increment_counter(&env, &counter_key), 3);
    }
}
