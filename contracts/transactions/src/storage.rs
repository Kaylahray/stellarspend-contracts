use soroban_sdk::{contracttype, Address, Env, Vec, Symbol, String, panic_with_error};
use crate::TransactionError;

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum TransactionStatus {
    Pending = 0,
    Completed = 1,
    Failed = 2,
    Cancelled = 3,
}

pub const MAX_TRANSACTIONS_PER_USER: u32 = 1000;

#[derive(Clone)]
#[contracttype]
pub struct Transaction {
    pub id: Symbol,
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub note: String,
    pub memo: String,
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub status: TransactionStatus,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Transaction by ID
    Transaction(Symbol),
    /// User's transaction list (user address -> vector of transaction IDs)
    UserTransactions(Address),
    /// Transaction counter for generating unique IDs
    TransactionCounter,
}

/// Create a new transaction
pub fn create_transaction(
    env: &Env,
    from: Address,
    to: Address,
    amount: i128,
    note: String,
    memo: String,
    tags: Vec<String>,
) -> Transaction {
    let mut counter: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::TransactionCounter)
        .unwrap_or(0);
    
    counter += 1;
    // Create a unique symbol using the counter as a string
    let counter_str = if counter == 1 {
        "1"
    } else if counter == 2 {
        "2"
    } else if counter == 3 {
        "3"
    } else if counter == 4 {
        "4"
    } else if counter == 5 {
        "5"
    } else if counter == 6 {
        "6"
    } else if counter == 7 {
        "7"
    } else if counter == 8 {
        "8"
    } else if counter == 9 {
        "9"
    } else if counter == 10 {
        "10"
    } else {
        "tx" // fallback for larger numbers
    };
    let tx_id = Symbol::new(&env, counter_str);
    
    let mut user_txs: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&DataKey::UserTransactions(from.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    if user_txs.len() >= MAX_TRANSACTIONS_PER_USER {
        panic_with_error!(env, TransactionError::TransactionLimitReached);
    }
    
    let transaction = Transaction {
        id: tx_id.clone(),
        from: from.clone(),
        to,
        amount,
        note: note.clone(),
        memo: memo.clone(),
        tags: tags.clone(),
        timestamp: env.ledger().timestamp(),
        status: TransactionStatus::Pending,
    };
    
    // Store the transaction
    env.storage()
        .persistent()
        .set(&DataKey::Transaction(tx_id.clone()), &transaction);
    
    // Update transaction counter
    env.storage()
        .persistent()
        .set(&DataKey::TransactionCounter, &counter);
    
    // Add to user's transaction list
    user_txs.push_back(tx_id.clone());
    env.storage()
        .persistent()
        .set(&DataKey::UserTransactions(from), &user_txs);
    
    transaction
}

/// Update the amount for a transaction (only transaction owner can update)
pub fn update_transaction_amount(env: &Env, id: Symbol, caller: Address, new_amount: i128) -> bool {
    let mut transaction: Transaction = match env
        .storage()
        .persistent()
        .get(&DataKey::Transaction(id.clone())) {
        Some(tx) => tx,
        None => return false,
    };
    
    if transaction.from != caller {
        return false;
    }
    
    if new_amount <= 0 {
        return false;
    }
    
    transaction.amount = new_amount;
    env.storage()
        .persistent()
        .set(&DataKey::Transaction(id), &transaction);
    
    true
}

/// Get a transaction by ID
pub fn get_transaction(env: &Env, id: Symbol) -> Option<Transaction> {
    env.storage().persistent().get(&DataKey::Transaction(id))
}

/// Update transaction note (only transaction owner can update)
pub fn update_transaction_note(env: &Env, id: Symbol, caller: Address, new_note: String) -> bool {
    let mut transaction: Transaction = match env
        .storage()
        .persistent()
        .get(&DataKey::Transaction(id.clone())) {
        Some(tx) => tx,
        None => return false,
    };
    
    // Verify caller is the transaction owner
    if transaction.from != caller {
        return false;
    }
    
    // Update the note
    transaction.note = new_note.clone();
    
    // Store updated transaction
    env.storage()
        .persistent()
        .set(&DataKey::Transaction(id), &transaction);
    
    true
}

/// Update transaction status (only admin or transaction owner can update)
pub fn update_transaction_status(env: &Env, id: Symbol, caller: Address, new_status: TransactionStatus) -> bool {
    let mut transaction: Transaction = match env
        .storage()
        .persistent()
        .get(&DataKey::Transaction(id.clone())) {
        Some(tx) => tx,
        None => return false,
    };
    
    // Verify caller is the transaction owner or admin
    if transaction.from != caller {
        // Check if caller is admin
        let admin: Option<Address> = env.storage().instance().get(&crate::DataKey::Admin);
        if admin.is_none() || admin.unwrap() != caller {
            return false;
        }
    }
    
    // Update the status
    transaction.status = new_status;
    
    // Store updated transaction
    env.storage()
        .persistent()
        .set(&DataKey::Transaction(id), &transaction);
    
    true
}

/// Get transaction timestamp
pub fn get_transaction_timestamp(env: &Env, id: Symbol) -> Option<u64> {
    get_transaction(env, id).map(|tx| tx.timestamp)
}

/// Get all transactions for a user
pub fn get_user_transactions(env: &Env, user: Address) -> Vec<Transaction> {
    let tx_ids: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&DataKey::UserTransactions(user.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    let mut transactions = Vec::new(env);
    for tx_id in tx_ids.iter() {
        if let Some(tx) = get_transaction(env, tx_id) {
            transactions.push_back(tx);
        }
    }
    
    transactions
}

/// Clear all transactions for a user (only user can perform this)
pub fn clear_user_transactions(env: &Env, user: Address) -> bool {
    let tx_ids: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&DataKey::UserTransactions(user.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    // Remove all transactions
    for tx_id in tx_ids.iter() {
        env.storage()
            .persistent()
            .remove(&DataKey::Transaction(tx_id));
    }
    
    // Clear user's transaction list
    env.storage()
        .persistent()
        .remove(&DataKey::UserTransactions(user));
    
    true
}

/// Get the last (most recent) transaction for a user
pub fn get_last_transaction(env: &Env, user: Address) -> Option<Transaction> {
    let transactions = get_user_transactions(env, user);
    
    if transactions.is_empty() {
        return None;
    }
    
    // Find the transaction with the latest timestamp
    let mut latest_tx = None;
    let mut latest_timestamp = 0u64;
    
    for tx in transactions.iter() {
        if tx.timestamp > latest_timestamp {
            latest_timestamp = tx.timestamp;
            latest_tx = Some(tx.clone());
        }
    }
    
    latest_tx
}

/// Get the total number of transactions recorded in the contract
pub fn get_total_transactions_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::TransactionCounter)
        .unwrap_or(0)
}

/// Check if a transaction exists
pub fn transaction_exists(env: &Env, id: Symbol) -> bool {
    env.storage().persistent().has(&DataKey::Transaction(id))
}

/// Check if a user is the owner of a transaction
pub fn is_transaction_owner(env: &Env, id: Symbol, user: Address) -> bool {
    if let Some(transaction) = get_transaction(env, id) {
        transaction.from == user
    } else {
        false
    }
}

/// Get transaction memo
pub fn get_transaction_memo(env: &Env, id: Symbol) -> Option<String> {
    get_transaction(env, id).map(|tx| tx.memo)
}
