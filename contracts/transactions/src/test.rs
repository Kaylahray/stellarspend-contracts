use soroban_sdk::{
    testutils::Address as _, testutils::Bytes as _, Address, Env, Symbol, String,
};
use crate::{TransactionsContract, TransactionError, Transaction};

#[test]
fn test_initialize_and_get_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    // Test initialization
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    // Verify admin is set
    assert_eq!(TransactionsContract::get_admin(env.clone()), Some(admin.clone()));
    
    // Test duplicate initialization fails
    let admin2 = Address::generate(&env);
    let result = std::panic::catch_unwind(|| {
        TransactionsContract::initialize(env.clone(), admin2);
    });
    assert!(result.is_err());
}

#[test]
fn test_create_transaction() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 1000;
    let note = String::from_str(&env, "Test transaction");
    
    // Create transaction
    let tx_id = TransactionsContract::create_transaction(
        env.clone(),
        from.clone(),
        to.clone(),
        amount,
        note.clone(),
    );
    
    // Verify transaction was created
    let transaction = TransactionsContract::get_transaction(env.clone(), tx_id.clone()).unwrap();
    assert_eq!(transaction.id, tx_id);
    assert_eq!(transaction.from, from);
    assert_eq!(transaction.to, to);
    assert_eq!(transaction.amount, amount);
    assert_eq!(transaction.note, note);
    assert!(transaction.timestamp > 0);
}

#[test]
fn test_create_transaction_invalid_amount() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Invalid amount test");
    
    // Test zero amount fails
    let result = std::panic::catch_unwind(|| {
        TransactionsContract::create_transaction(
            env.clone(),
            from.clone(),
            to.clone(),
            0,
            note.clone(),
        );
    });
    assert!(result.is_err());
    
    // Test negative amount fails
    let result = std::panic::catch_unwind(|| {
        TransactionsContract::create_transaction(
            env.clone(),
            from.clone(),
            to.clone(),
            -100,
            note.clone(),
        );
    });
    assert!(result.is_err());
}

#[test]
fn test_update_transaction_note() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 1000;
    let original_note = String::from_str(&env, "Original note");
    let updated_note = String::from_str(&env, "Updated note");
    
    // Create transaction
    let tx_id = TransactionsContract::create_transaction(
        env.clone(),
        from.clone(),
        to.clone(),
        amount,
        original_note.clone(),
    );
    
    // Verify original note
    let transaction = TransactionsContract::get_transaction(env.clone(), tx_id.clone()).unwrap();
    assert_eq!(transaction.note, original_note);
    
    // Update note (this should work since the caller is the transaction owner)
    let success = TransactionsContract::update_transaction_note(
        env.clone(),
        tx_id.clone(),
        updated_note.clone(),
    );
    
    // Note: In a real implementation, we'd need proper auth handling
    // For this test, we'll verify the function exists and can be called
    // The actual authorization would be handled by the Soroban runtime
    
    // Verify note was updated
    let updated_transaction = TransactionsContract::get_transaction(env.clone(), tx_id.clone()).unwrap();
    assert_eq!(updated_transaction.note, updated_note);
}

#[test]
fn test_get_transaction_timestamp() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 1000;
    let note = String::from_str(&env, "Timestamp test");
    
    // Create transaction
    let tx_id = TransactionsContract::create_transaction(
        env.clone(),
        from.clone(),
        to.clone(),
        amount,
        note.clone(),
    );
    
    // Get timestamp
    let timestamp = TransactionsContract::get_transaction_timestamp(env.clone(), tx_id.clone());
    assert!(timestamp.is_some());
    assert!(timestamp.unwrap() > 0);
    
    // Test non-existent transaction
    let fake_id = Symbol::new(&env, "fake_id");
    let fake_timestamp = TransactionsContract::get_transaction_timestamp(env.clone(), fake_id);
    assert!(fake_timestamp.is_none());
}

#[test]
fn test_get_user_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    // Create transactions for user1
    let tx1_id = TransactionsContract::create_transaction(
        env.clone(),
        user1.clone(),
        recipient.clone(),
        1000,
        String::from_str(&env, "User1 transaction 1"),
    );
    
    let tx2_id = TransactionsContract::create_transaction(
        env.clone(),
        user1.clone(),
        recipient.clone(),
        2000,
        String::from_str(&env, "User1 transaction 2"),
    );
    
    // Create transaction for user2
    let tx3_id = TransactionsContract::create_transaction(
        env.clone(),
        user2.clone(),
        recipient.clone(),
        3000,
        String::from_str(&env, "User2 transaction"),
    );
    
    // Get user1's transactions
    let user1_txs = TransactionsContract::get_user_transactions(env.clone(), user1.clone());
    assert_eq!(user1_txs.len(), 2);
    
    // Get user2's transactions
    let user2_txs = TransactionsContract::get_user_transactions(env.clone(), user2.clone());
    assert_eq!(user2_txs.len(), 1);
    
    // Get transactions for non-existent user
    let non_existent_user = Address::generate(&env);
    let empty_txs = TransactionsContract::get_user_transactions(env.clone(), non_existent_user);
    assert_eq!(empty_txs.len(), 0);
}

#[test]
fn test_clear_user_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    // Create multiple transactions for the user
    let tx1_id = TransactionsContract::create_transaction(
        env.clone(),
        user.clone(),
        recipient.clone(),
        1000,
        String::from_str(&env, "Transaction 1"),
    );
    
    let tx2_id = TransactionsContract::create_transaction(
        env.clone(),
        user.clone(),
        recipient.clone(),
        2000,
        String::from_str(&env, "Transaction 2"),
    );
    
    // Verify transactions exist
    let user_txs = TransactionsContract::get_user_transactions(env.clone(), user.clone());
    assert_eq!(user_txs.len(), 2);
    
    // Clear user transactions
    let success = TransactionsContract::clear_user_transactions(env.clone(), user.clone());
    assert!(success);
    
    // Verify transactions are cleared
    let empty_txs = TransactionsContract::get_user_transactions(env.clone(), user.clone());
    assert_eq!(empty_txs.len(), 0);
    
    // Verify individual transactions are removed
    let tx1 = TransactionsContract::get_transaction(env.clone(), tx1_id);
    assert!(tx1.is_none());
    
    let tx2 = TransactionsContract::get_transaction(env.clone(), tx2_id);
    assert!(tx2.is_none());
}

#[test]
fn test_transaction_counter_increments() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    // Create multiple transactions
    let tx1_id = TransactionsContract::create_transaction(
        env.clone(),
        from.clone(),
        to.clone(),
        1000,
        String::from_str(&env, "Transaction 1"),
    );
    
    let tx2_id = TransactionsContract::create_transaction(
        env.clone(),
        from.clone(),
        to.clone(),
        2000,
        String::from_str(&env, "Transaction 2"),
    );
    
    let tx3_id = TransactionsContract::create_transaction(
        env.clone(),
        from.clone(),
        to.clone(),
        3000,
        String::from_str(&env, "Transaction 3"),
    );
    
    // Verify IDs are different and sequential
    assert_ne!(tx1_id, tx2_id);
    assert_ne!(tx2_id, tx3_id);
    assert_ne!(tx1_id, tx3_id);
    
    // Verify all transactions exist
    assert!(TransactionsContract::get_transaction(env.clone(), tx1_id).is_some());
    assert!(TransactionsContract::get_transaction(env.clone(), tx2_id).is_some());
    assert!(TransactionsContract::get_transaction(env.clone(), tx3_id).is_some());
}
#[test]
fn test_transaction_exists() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    
    TransactionsContract::initialize(env.clone(), admin.clone());
    
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 1000;
    let note = String::from_str(&env, "Existence test");
    
    // Create transaction
    let tx_id = TransactionsContract::create_transaction(
        env.clone(),
        from.clone(),
        to.clone(),
        amount,
        note,
    );
    
    // Verify existence
    assert!(TransactionsContract::transaction_exists(env.clone(), tx_id.clone()));
    
    // Test non-existent transaction
    let fake_id = Symbol::new(&env, "not_here");
    assert!(!TransactionsContract::transaction_exists(env.clone(), fake_id));
}
