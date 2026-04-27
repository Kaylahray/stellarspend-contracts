#![no_std]

use soroban_sdk::{Env, String};

pub mod utils;
pub mod errors;

pub use errors::SharedError;

pub const SHARED_VERSION: &str = "0.1.0";

pub fn get_version(env: Env) -> String {
    String::from_str(&env, SHARED_VERSION)
}

#[cfg(test)]
mod tests {
    use super::get_version;
    use soroban_sdk::{Env, String};

    #[test]
    fn returns_shared_version() {
        let env = Env::default();
        let version = get_version(env);
        assert_eq!(version, String::from_str(&Env::default(), "0.1.0"));
    }
}
