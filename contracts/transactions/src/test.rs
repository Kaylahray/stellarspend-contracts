use soroban_sdk::{
    testutils::Address as _, Address, Env, Symbol, String, Vec,
};
use crate::{TransactionsContract, TransactionsContractClient, TransactionError, Transaction, TransactionStatus};

#[test]
fn test_initialize_and_get_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    assert_eq!(client.get_admin(), Some(admin.clone()));
}

#[test]
#[should_panic]
fn test_initialize_duplicate_fails() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    let admin2 = Address::generate(&env);
    client.initialize(&admin2);
}

#[test]
fn test_create_transaction() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Test transaction");
    let memo = String::from_str(&env, "Payment memo");
    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "groceries"));
    tags.push_back(String::from_str(&env, "monthly"));

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &memo, &tags);

    let transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(transaction.id, tx_id);
    assert_eq!(transaction.from, from);
    assert_eq!(transaction.to, to);
    assert_eq!(transaction.amount, amount);
    assert_eq!(transaction.note, note);
    assert_eq!(transaction.memo, memo);
    assert_eq!(transaction.tags.len(), 2);
    assert_eq!(transaction.tags.get(0), Some(String::from_str(&env, "groceries")));
    assert_eq!(transaction.tags.get(1), Some(String::from_str(&env, "monthly")));
    assert!(transaction.timestamp > 0);
    assert_eq!(transaction.status, crate::TransactionStatus::Completed);
}

#[test]
#[should_panic]
fn test_create_transaction_invalid_amount_zero() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Invalid amount test");
    let zero_amount: i128 = 0;

    client.create_transaction(&from, &to, &zero_amount, &note, &Vec::new(&env));
}

#[test]
#[should_panic]
fn test_create_transaction_invalid_amount_negative() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Invalid amount test");
    let negative_amount: i128 = -100;

    client.create_transaction(&from, &to, &negative_amount, &note, &Vec::new(&env));
}

#[test]
fn test_update_transaction_note() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let original_note = String::from_str(&env, "Original note");
    let updated_note = String::from_str(&env, "Updated note");

    let tx_id = client.create_transaction(&from, &to, &amount, &original_note, &Vec::new(&env));

    let transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(transaction.note, original_note);

    let success = client.update_transaction_note(&tx_id, &from, &updated_note);
    assert!(success);

    let updated_transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(updated_transaction.note, updated_note);
}

#[test]
fn test_update_transaction_amount() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let updated_amount: i128 = 1500;
    let note = String::from_str(&env, "Amount update");
    let tags = Vec::new(&env);

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &tags);

    let success = client.update_transaction_amount(&tx_id, &from, &updated_amount);
    assert!(success);

    let transaction = client.get_transaction(&tx_id).unwrap();
    assert_eq!(transaction.amount, updated_amount);
}

#[test]
#[should_panic]
fn test_transaction_limit_per_user() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Limit test");
    let one: i128 = 1;

    for _ in 0..1000 {
        let tags = Vec::new(&env);
        client.create_transaction(&from, &to, &one, &note, &tags);
    }

    client.create_transaction(&from, &to, &one, &note, &Vec::new(&env));
}

#[test]
fn test_get_transaction_timestamp() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Timestamp test");

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &Vec::new(&env));

    let timestamp = client.get_transaction_timestamp(&tx_id);
    assert!(timestamp.is_some());
    assert!(timestamp.unwrap() > 0);

    let fake_id = Symbol::new(&env, "fake_id");
    let fake_timestamp = client.get_transaction_timestamp(&fake_id);
    assert!(fake_timestamp.is_none());
}

#[test]
fn test_get_user_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    let tx1_id = client.create_transaction(&user1, &recipient, &1000, &String::from_str(&env, "User1 transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&user1, &recipient, &2000, &String::from_str(&env, "User1 transaction 2"), &Vec::new(&env));
    let tx3_id = client.create_transaction(&user2, &recipient, &3000, &String::from_str(&env, "User2 transaction"), &Vec::new(&env));

    let user1_txs = client.get_user_transactions(&user1);
    assert_eq!(user1_txs.len(), 2);

    let user2_txs = client.get_user_transactions(&user2);
    assert_eq!(user2_txs.len(), 1);

    let non_existent_user = Address::generate(&env);
    let empty_txs = client.get_user_transactions(&non_existent_user);
    assert_eq!(empty_txs.len(), 0);
}

#[test]
fn test_clear_user_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    let tx1_id = client.create_transaction(&user, &recipient, &1000, &String::from_str(&env, "Transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&user, &recipient, &2000, &String::from_str(&env, "Transaction 2"), &Vec::new(&env));

    let user_txs = client.get_user_transactions(&user);
    assert_eq!(user_txs.len(), 2);

    let success = client.clear_user_transactions(&user);
    assert!(success);

    let empty_txs = client.get_user_transactions(&user);
    assert_eq!(empty_txs.len(), 0);

    let tx1 = client.get_transaction(&tx1_id);
    assert!(tx1.is_none());
    let tx2 = client.get_transaction(&tx2_id);
    assert!(tx2.is_none());
}

#[test]
fn test_transaction_counter_increments() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let tx1_id = client.create_transaction(&from, &to, &1000, &String::from_str(&env, "Transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&from, &to, &2000, &String::from_str(&env, "Transaction 2"), &Vec::new(&env));
    let tx3_id = client.create_transaction(&from, &to, &3000, &String::from_str(&env, "Transaction 3"), &Vec::new(&env));

    assert_ne!(tx1_id, tx2_id);
    assert_ne!(tx2_id, tx3_id);
    assert_ne!(tx1_id, tx3_id);

    assert!(client.get_transaction(&tx1_id).is_some());
    assert!(client.get_transaction(&tx2_id).is_some());
    assert!(client.get_transaction(&tx3_id).is_some());
}

#[test]
fn test_transaction_exists() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Existence test");

    let memo = String::from_str(&env, "Existence test memo");
    let tx_id = client.create_transaction(&from, &to, &amount, &note, &memo, &Vec::new(&env));

    assert!(client.transaction_exists(&tx_id));

    let fake_id = Symbol::new(&env, "not_here");
    assert!(!client.transaction_exists(&fake_id));
}

#[test]
fn test_get_transaction_memo() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount: i128 = 1000;
    let note = String::from_str(&env, "Test transaction");
    let memo = String::from_str(&env, "Important payment memo");
    let tags = Vec::new(&env);

    let tx_id = client.create_transaction(&from, &to, &amount, &note, &memo, &tags);

    // Test get_transaction_memo function
    let retrieved_memo = client.get_transaction_memo(&tx_id).unwrap();
    assert_eq!(retrieved_memo, memo);
}

#[test]
fn test_get_transaction_memo_nonexistent() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let fake_id = Symbol::new(&env, "not_here");
    
    // Test get_transaction_memo for non-existent transaction
    let memo = client.get_transaction_memo(&fake_id);
    assert!(memo.is_none());
}

#[test]
fn test_get_all_transactions() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Create some transactions
    let tx1_id = client.create_transaction(&user1, &recipient, &1000, &String::from_str(&env, "Transaction 1"), &Vec::new(&env));
    let tx2_id = client.create_transaction(&user2, &recipient, &2000, &String::from_str(&env, "Transaction 2"), &Vec::new(&env));
    let tx3_id = client.create_transaction(&user1, &recipient, &3000, &String::from_str(&env, "Transaction 3"), &Vec::new(&env));

    let all_txs = client.get_all_transactions();
    assert_eq!(all_txs.len(), 3);

    // Check that all transactions are present
    let ids: Vec<Symbol> = all_txs.iter().map(|tx| tx.id.clone()).collect();
    assert!(ids.contains(&tx1_id));
    assert!(ids.contains(&tx2_id));
    assert!(ids.contains(&tx3_id));
}

#[test]
fn test_get_all_transactions_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(TransactionsContract, ());
    let client = TransactionsContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    let all_txs = client.get_all_transactions();
    assert_eq!(all_txs.len(), 0);
}
