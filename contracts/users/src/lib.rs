#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Vec,
};

mod storage;

pub use storage::{add_user, get_user_count, user_exists, get_all_users};

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
    
    /// Get all registered users (admin only)
    pub fn get_all_users(env: Env, caller: Address) -> Vec<Address> {
        caller.require_auth();
        Self::require_admin(&env, &caller);
        
        get_all_users(&env)
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
