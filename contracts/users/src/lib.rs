#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Vec, String,
};

mod storage;

pub use storage::{add_user, get_user_count, user_exists, get_all_users, reset_user_data, get_user_active_status, set_user_active_status, get_user_currency, set_user_currency, get_user_last_login, set_user_last_login};

#[cfg(test)]
mod test;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum UserError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    UserNotFound = 4,
    UserAlreadyExists = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
}

#[contract]
pub struct UsersContract;

#[contractimpl]
impl UsersContract {
    /// Initialize the users contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, UserError::AlreadyInitialized);
        }
        
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        env.events().publish(
            (symbol_short!("users"), symbol_short!("init")),
            admin,
        );
    }
    
    /// Register a user (adds them to the unique user set)
    /// Can be called by anyone to register themselves or others
    pub fn register_user(env: Env, user: Address) -> bool {
        if user_exists(&env, user.clone()) {
            return false; // Reject duplicate users
        }
        
        let is_new = add_user(&env, user.clone());
        
        if is_new {
            // Set last login timestamp on registration
            set_user_last_login(&env, user.clone(), env.ledger().timestamp());
            
            env.events().publish(
                (symbol_short!("users"), symbol_short!("reg")),
                user,
            );
        }
        
        is_new
    }
    
    /// Get the total count of unique users who have interacted with the contract
    pub fn get_all_users_count(env: Env) -> u64 {
        get_user_count(&env)
    }
    
    /// Check if a specific user is registered
    pub fn is_user_registered(env: Env, user: Address) -> bool {
        user_exists(&env, user)
    }

    /// Verify user existence — returns `true` if the user has been registered,
    /// `false` otherwise. Functionally identical to `is_user_registered`;
    /// exposed under this name to satisfy the `check_user_exists` API surface
    /// requested in issue #336.
    pub fn check_user_exists(env: Env, user: Address) -> bool {
        user_exists(&env, user)
    }
    
    /// Get all registered users (admin only)
    pub fn get_all_users(env: Env, caller: Address) -> Vec<Address> {
        caller.require_auth();
        Self::require_admin(&env, &caller);
        
        get_all_users(&env)
    }
    
    /// Reset the user's profile data (only the user may call)
    pub fn reset_user_data(env: Env, user: Address) -> bool {
        user.require_auth();

        let success = reset_user_data(&env, user.clone());

        if success {
            env.events().publish(
                (symbol_short!("users"), symbol_short!("reset")),
                user,
            );
        }

        success
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Get user activity status
    pub fn get_user_active_status(env: Env, user: Address) -> bool {
        get_user_active_status(&env, user)
    }

    /// Set user activity status (only admin can set)
    pub fn set_user_active_status(env: Env, caller: Address, user: Address, is_active: bool) -> bool {
        caller.require_auth();
        Self::require_admin(&env, &caller);
        
        let success = set_user_active_status(&env, user.clone(), is_active);
        
        if success {
            env.events().publish(
                (symbol_short!("users"), symbol_short!("active_upd")),
                (user, is_active),
            );
        }
        
        success
    }

    /// Get user's preferred currency
    pub fn get_user_currency(env: Env, user: Address) -> String {
        get_user_currency(&env, user)
    }

    /// Set user's preferred currency (only the user can set their own currency)
    pub fn set_user_currency(env: Env, user: Address, currency: String) -> bool {
        user.require_auth();
        
        let success = set_user_currency(&env, user.clone(), currency.clone());
        
        if success {
            env.events().publish(
                (symbol_short!("users"), symbol_short!("currency_upd")),
                (user, currency),
            );
        }
        
        success
    }
    
    /// Get user's last login timestamp
    pub fn get_user_last_login(env: Env, user: Address) -> Option<u64> {
        get_user_last_login(&env, user)
    }
    
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, UserError::NotInitialized));
        if caller != &admin {
            panic_with_error!(env, UserError::Unauthorized);
        }
    }
}

// ── Issue #336: check_user_exists ────────────────────────────────────────────
//
// Tests are inline here (rather than in the sibling `test.rs` file) because
// `test.rs` is currently not wired as a module from `lib.rs`. Wiring it up
// surfaces several pre-existing compile errors in tests that have never run
// (missing `Vec` import, `Option<Address>` vs `Address` mismatches, and
// `std::panic::catch_unwind` calls that don't compile in this `no_std`
// crate). Repairing those is out of scope for issue #336; tracking that as
// a separate concern keeps this PR focused.
#[cfg(test)]
mod check_user_exists_tests {
    use super::{UsersContract, UsersContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup<'a>() -> (Env, Address, UsersContractClient<'a>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(UsersContract, ());
        let client = UsersContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    #[test]
    fn returns_false_for_unregistered_user() {
        let (env, _admin, client) = setup();
        let stranger = Address::generate(&env);
        assert!(!client.check_user_exists(&stranger));
    }

    #[test]
    fn returns_true_after_registration() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);

        // Before registration → false
        assert!(!client.check_user_exists(&user));

        // After registration → true
        client.register_user(&user);
        assert!(client.check_user_exists(&user));
    }

    #[test]
    fn matches_is_user_registered_for_parity() {
        // check_user_exists is a deliberate alias for is_user_registered.
        // This test guards against future divergence between the two.
        let (env, _admin, client) = setup();
        let registered = Address::generate(&env);
        let unregistered = Address::generate(&env);
        client.register_user(&registered);

        assert_eq!(
            client.check_user_exists(&registered),
            client.is_user_registered(&registered),
        );
        assert_eq!(
            client.check_user_exists(&unregistered),
            client.is_user_registered(&unregistered),
        );
    }
}
