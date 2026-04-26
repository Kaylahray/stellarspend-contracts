#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, String, Vec,
};

mod storage;

pub use storage::{
    add_user, deactivate_user as storage_deactivate_user, get_all_users, get_default_currency,
    get_user_count, is_user_active, reset_user_data, set_default_currency, user_exists,
};

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
        let is_new = add_user(&env, user.clone());
        
        if is_new {
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

    /// Set default currency preference for a registered user.
    pub fn set_default_currency(env: Env, user: Address, currency: String) {
        user.require_auth();

        if !user_exists(&env, user.clone()) {
            panic_with_error!(&env, UserError::UserNotFound);
        }

        set_default_currency(&env, user.clone(), currency.clone());

        env.events().publish(
            (symbol_short!("users"), symbol_short!("def_curr")),
            (user, currency),
        );
    }

    /// Get default currency preference for a user.
    pub fn get_default_currency(env: Env, user: Address) -> Option<String> {
        get_default_currency(&env, user)
    }

    /// Deactivate a registered user account (only the user may call).
    pub fn deactivate_user(env: Env, user: Address) -> bool {
        user.require_auth();

        let success = storage_deactivate_user(&env, user.clone());

        if success {
            env.events().publish(
                (symbol_short!("users"), symbol_short!("deact")),
                user,
            );
        }

        success
    }

    /// Return whether the given user account is active.
    pub fn is_user_active(env: Env, user: Address) -> bool {
        is_user_active(&env, user)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
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
