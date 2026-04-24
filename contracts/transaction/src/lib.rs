use soroban_sdk::{contractimpl, Env, Address, Symbol};

pub struct TransactionContract;

#[contractimpl]
impl TransactionContract {
    /// Create a transaction and emit an event
    pub fn create_transaction(env: Env, from: Address, to: Address, amount: i128) {
        // Store transaction data (simplified example)
        let tx_key = format!("tx:{}:{}", from, to);
        env.storage().set(&tx_key, &amount);

        // Emit event for off-chain listeners
        env.events().publish(
            (Symbol::short("transaction_created"),),
            (from, to, amount),
        );
    }
}

use soroban_sdk::{contractimpl, Env, Address, Symbol};

pub struct TransactionContract;

#[contractimpl]
impl TransactionContract {
    /// Create a transaction, emit event, and increment user transaction count
    pub fn create_transaction(env: Env, from: Address, to: Address, amount: i128) {
        // Store transaction data (simplified example)
        let tx_key = format!("tx:{}:{}", from, to);
        env.storage().set(&tx_key, &amount);

        // Increment transaction count for sender
        let count_key = format!("count:{}", from);
        let current_count: i128 = env.storage().get(&count_key).unwrap_or(0);
        env.storage().set(&count_key, &(current_count + 1));

        // Emit event for off-chain listeners
        env.events().publish(
            (Symbol::short("transaction_created"),),
            (from, to, amount),
        );
    }

    /// Get the transaction count for a given user
    pub fn get_user_transaction_count(env: Env, user: Address) -> i128 {
        let count_key = format!("count:{}", user);
        env.storage().get(&count_key).unwrap_or(0)
    }
}
