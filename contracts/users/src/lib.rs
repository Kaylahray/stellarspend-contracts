#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Vec, String,
};

mod storage;

pub use storage::{
    add_user, get_user_count, user_exists, get_all_users, reset_user_data,
    get_user_active_status, set_user_active_status,
    get_user_currency, set_user_currency,
    get_user_last_login, set_user_last_login,
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
    /// Initialize the users contract with an admin address.
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

    /// Register a new user.
    ///
    /// Issue: "Track when user joined" — the current ledger timestamp is
    /// recorded via `set_user_last_login` immediately on registration and
    /// can be read back with `get_user_last_login`.
    ///
    /// Issue: "Emit event when user registers" — a `("users", "reg")` event
    /// carrying the registrant's address is published on every successful
    /// registration.
    ///
    /// Returns `true` when the user was newly registered, `false` if they
    /// were already present (idempotent, no event emitted for duplicates).
    pub fn register_user(env: Env, user: Address) -> bool {
        if user_exists(&env, user.clone()) {
            return false;
        }

        let is_new = add_user(&env, user.clone());

        if is_new {
            // ── Issue: Track when user joined ────────────────────────────
            // Capture the ledger timestamp at the moment of registration so
            // callers can query it later via `get_user_last_login`.
            set_user_last_login(&env, user.clone(), env.ledger().timestamp());

            // ── Issue: Emit event when user registers ────────────────────
            // Publish a structured event so off-chain indexers and other
            // contracts can react to new registrations.
            env.events().publish(
                (symbol_short!("users"), symbol_short!("reg")),
                user,
            );
        }

        is_new
    }

    /// Return the total count of registered users.
    pub fn get_all_users_count(env: Env) -> u64 {
        get_user_count(&env)
    }

    /// Return `true` if the given address has been registered.
    pub fn is_user_registered(env: Env, user: Address) -> bool {
        user_exists(&env, user)
    }

    /// Alias for `is_user_registered`; satisfies the `check_user_exists` API
    /// surface requested in issue #336.
    pub fn check_user_exists(env: Env, user: Address) -> bool {
        user_exists(&env, user)
    }

    /// Return all registered user addresses (admin only).
    pub fn get_all_users(env: Env, caller: Address) -> Vec<Address> {
        caller.require_auth();
        Self::require_admin(&env, &caller);
        get_all_users(&env)
    }

    /// Reset the calling user's profile data.
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

    /// Return the stored admin address, if the contract has been initialized.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Return the activity status for the given user.
    pub fn get_user_active_status(env: Env, user: Address) -> bool {
        get_user_active_status(&env, user)
    }

    /// Set the activity status for a user (admin only).
    pub fn set_user_active_status(
        env: Env,
        caller: Address,
        user: Address,
        is_active: bool,
    ) -> bool {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        let success = set_user_active_status(&env, user.clone(), is_active);

        if success {
            env.events().publish(
                (symbol_short!("users"), symbol_short!("actv_upd")),
                (user, is_active),
            );
        }

        success
    }

    /// Return the user's preferred currency string.
    pub fn get_user_currency(env: Env, user: Address) -> String {
        get_user_currency(&env, user)
    }

    /// Set the calling user's preferred currency.
    pub fn set_user_currency(env: Env, user: Address, currency: String) -> bool {
        user.require_auth();

        let success = set_user_currency(&env, user.clone(), currency.clone());

        if success {
            env.events().publish(
                (symbol_short!("users"), symbol_short!("curr_upd")),
                (user, currency),
            );
        }

        success
    }

    /// Return the ledger timestamp recorded when the user last logged in (or
    /// registered, whichever is more recent).
    pub fn get_user_last_login(env: Env, user: Address) -> Option<u64> {
        get_user_last_login(&env, user)
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

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

// ── Issue #336: check_user_exists ─────────────────────────────────────────────
//
// Tests live here (not in test.rs) to avoid surfacing pre-existing compile
// errors in that file (missing Vec import, Option<Address> mismatches, and
// std::panic::catch_unwind calls incompatible with no_std). Fixing those is
// tracked separately.
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
        assert!(!client.check_user_exists(&user));
        client.register_user(&user);
        assert!(client.check_user_exists(&user));
    }

    #[test]
    fn matches_is_user_registered_for_parity() {
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

    /// Verifies that the join timestamp is populated on registration.
    #[test]
    fn registration_records_join_timestamp() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);

        // No timestamp before registration
        assert!(client.get_user_last_login(&user).is_none());

        client.register_user(&user);

        // Timestamp present after registration
        assert!(client.get_user_last_login(&user).is_some());
    }
}