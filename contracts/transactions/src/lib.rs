#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Symbol, String, Vec,
};

mod storage;

pub use storage::{
    create_transaction, get_transaction, get_transaction_timestamp, get_user_transactions,
    clear_user_transactions, transaction_exists, get_last_transaction, get_total_transactions_count, 
    update_transaction_status, is_transaction_owner, Transaction, TransactionStatus,
};

#[cfg(test)]
mod test;

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
    TransactionLimitReached = 7,
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
        tags: Vec<String>,
    ) -> Symbol {
        from.require_auth();
        
        if amount <= 0 {
            panic_with_error!(&env, TransactionError::InvalidAmount);
        }
        
        let transaction = create_transaction(&env, from.clone(), to, amount, note, tags);
        
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
                id.clone(),
            );
        }
        
        success
    }

    /// Update the amount for a transaction (only transaction owner can update)
    pub fn update_transaction_amount(env: Env, id: Symbol, caller: Address, amount: i128) -> bool {
        caller.require_auth();
        
        if amount <= 0 {
            panic_with_error!(&env, TransactionError::InvalidAmount);
        }
        
        if !transaction_exists(&env, id.clone()) {
            panic_with_error!(&env, TransactionError::TransactionNotFound);
        }
        
        let success = storage::update_transaction_amount(&env, id.clone(), caller, amount);
        
        if success {
            env.events().publish(
                (symbol_short!("tx"), symbol_short!("amount_up")),
                id.clone(),
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
    
    /// Get the last (most recent) transaction for a user
    pub fn get_last_transaction(env: Env, user: Address) -> Option<Transaction> {
        get_last_transaction(&env, user)
    }
    
    /// Get the total number of transactions recorded in the contract
    pub fn get_total_transactions_count(env: Env) -> u64 {
        get_total_transactions_count(&env)
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

    /// Update the status of a transaction (only owner/admin allowed)
    pub fn update_transaction_status(env: Env, id: Symbol, caller: Address, status: TransactionStatus) -> bool {
        caller.require_auth();
        
        if !transaction_exists(&env, id.clone()) {
            panic_with_error!(&env, TransactionError::TransactionNotFound);
        }
        
        let success = storage::update_transaction_status(&env, id.clone(), caller, status);
        
        if success {
            env.events().publish(
                (symbol_short!("tx"), symbol_short!("status_upd")),
                (id.clone(), status),
            );
        }
        
        success
    }

    /// Check if a user is the owner of a transaction
    pub fn is_transaction_owner(env: Env, id: Symbol, user: Address) -> bool {
        is_transaction_owner(&env, id, user)
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
