#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Symbol, String, Vec,
};

mod storage;

pub use storage::{
    create_transaction, get_transaction, get_transaction_timestamp, get_user_transactions,
    clear_user_transactions, transaction_exists, Transaction,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TransactionError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    TransactionNotFound = 4,
    InvalidAmount = 5,
    InvalidId = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
}

#[contract]
pub struct TransactionsContract;

#[contractimpl]
impl TransactionsContract {
    /// Initialize the transactions contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, TransactionError::AlreadyInitialized);
        }
        
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        env.events().publish(
            (symbol_short!("tx"), symbol_short!("init")),
            admin,
        );
    }
    
    /// Create a new transaction
    pub fn create_transaction(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        note: String,
    ) -> Symbol {
        from.require_auth();
        
        if amount <= 0 {
            panic_with_error!(&env, TransactionError::InvalidAmount);
        }
        
        let transaction = create_transaction(&env, from.clone(), to, amount, note);
        
        env.events().publish(
            (symbol_short!("tx"), symbol_short!("created")),
            (transaction.id.clone(), transaction.from.clone(), transaction.to.clone(), transaction.amount),
        );
        
        transaction.id
    }
    
    /// Update the note attached to a transaction (only transaction owner can update)
    pub fn update_transaction_note(env: Env, id: Symbol, caller: Address, note: String) -> bool {
        caller.require_auth();
        
        if !transaction_exists(&env, id.clone()) {
            panic_with_error!(&env, TransactionError::TransactionNotFound);
        }
        
        let success = storage::update_transaction_note(&env, id.clone(), caller, note);
        
        if success {
            env.events().publish(
                (symbol_short!("tx"), symbol_short!("note_upd")),
                id,
            );
        }
        
        success
    }
    
    /// Get the timestamp of a transaction
    pub fn get_transaction_timestamp(env: Env, id: Symbol) -> Option<u64> {
        get_transaction_timestamp(&env, id)
    }
    
    /// Get a transaction by ID
    pub fn get_transaction(env: Env, id: Symbol) -> Option<Transaction> {
        get_transaction(&env, id)
    }
    
    /// Get all transactions for a user
    pub fn get_user_transactions(env: Env, user: Address) -> Vec<Transaction> {
        get_user_transactions(&env, user)
    }
    
    /// Clear all transactions for a user (only user can perform this action)
    pub fn clear_user_transactions(env: Env, user: Address) -> bool {
        user.require_auth();
        
        let success = clear_user_transactions(&env, user.clone());
        
        if success {
            env.events().publish(
                (symbol_short!("tx"), symbol_short!("cleared")),
                user,
            );
        }
        
        success
    }
    
    /// Get the admin address
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Check if a transaction exists
    pub fn transaction_exists(env: Env, id: Symbol) -> bool {
        transaction_exists(&env, id)
    }

    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, TransactionError::NotInitialized));
        if caller != &admin {
            panic_with_error!(env, TransactionError::Unauthorized);
        }
    }
}
