use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env,
};

#[path = "../contracts/fees.rs"]
mod fees;

use fees::{FeeError, FeesContract, FeesContractClient};

fn setup_fee_contract() -> (Env, Address, FeesContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FeesContract, ());
    let client = FeesContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    // initialize with 500 bps (5%)
    client.initialize(&admin, &500u32);

    (env, admin, client)
}

#[test]
fn test_initialization_and_get() {
    let (env, admin, client) = setup_fee_contract();
    assert_eq!(client.get_percentage(), 500u32);
    assert_eq!(client.get_total_collected(), 0i128);
}

#[test]
fn test_set_percentage_unauthorized() {
    let (env, _admin, client) = setup_fee_contract();
    let other = Address::generate(&env);
    // should panic because other is not admin
    let result = std::panic::catch_unwind(|| {
        client.set_percentage(&other, &100u32);
    });
    assert!(result.is_err());
}

#[test]
fn test_calculate_and_deduct_fee() {
    let (env, admin, client) = setup_fee_contract();
    let payer = Address::generate(&env);
    let amount: i128 = 1_000;
    // fee = 1_000 * 500 / 10_000 = 50
    let fee = FeesContract::calculate_fee(env.clone(), amount);
    assert_eq!(fee, 50);

    // deduct fee via client
    let (net, charged) = client.deduct_fee(&payer, &amount);
    assert_eq!(charged, 50);
    assert_eq!(net, 950);

    // total collected should update
    assert_eq!(client.get_total_collected(), 50);

    // event emitted
    let events = env.events().all();
    assert!(events
        .iter()
        .any(|e| e.topics.0 == "fee" && e.topics.1 == "deducted"));
}

#[test]
fn test_total_collected_accumulates() {
    let (env, admin, client) = setup_fee_contract();
    let payer = Address::generate(&env);
    client.deduct_fee(&payer, &200);
    client.deduct_fee(&payer, &800);
    // 200*5% =10, 800*5%=40 => total 50
    assert_eq!(client.get_total_collected(), 50);
}

#[test]
fn test_invalid_amount_errors() {
    let (env, _admin, _client) = setup_fee_contract();
    // using contract impl directly to exercise panic
    let err = std::panic::catch_unwind(|| FeesContract::calculate_fee(env.clone(), 0));
    assert!(err.is_err());
}

#[test]
fn test_update_configuration_emits_event() {
    let (env, admin, client) = setup_fee_contract();
    client.set_percentage(&admin, &250u32); // 2.5%
    let events = env.events().all();
    assert!(events
        .iter()
        .any(|e| e.topics.0 == "fee" && e.topics.1 == "config_updated"));
    assert_eq!(client.get_percentage(), 250u32);
}

#[test]
fn test_user_fees_accrued_initialization() {
    let (env, _admin, client) = setup_fee_contract();
    let user = Address::generate(&env);
    // User with no fees should return 0
    assert_eq!(client.get_user_fees_accrued(&user), 0);
}

#[test]
fn test_user_fees_accrued_single_transaction() {
    let (env, _admin, client) = setup_fee_contract();
    let user = Address::generate(&env);
    let amount: i128 = 1_000;
    
    // fee = 1_000 * 500 / 10_000 = 50
    let (_net, fee) = client.deduct_fee(&user, &amount);
    assert_eq!(fee, 50);
    
    // User's accumulated fees should be 50
    assert_eq!(client.get_user_fees_accrued(&user), 50);
}

#[test]
fn test_user_fees_accrued_multiple_transactions() {
    let (env, _admin, client) = setup_fee_contract();
    let user = Address::generate(&env);
    
    // First transaction: 1_000, fee = 50
    let (_net1, fee1) = client.deduct_fee(&user, &1_000);
    assert_eq!(fee1, 50);
    assert_eq!(client.get_user_fees_accrued(&user), 50);
    
    // Second transaction: 800, fee = 40
    let (_net2, fee2) = client.deduct_fee(&user, &800);
    assert_eq!(fee2, 40);
    assert_eq!(client.get_user_fees_accrued(&user), 90);
    
    // Third transaction: 2_000, fee = 100
    let (_net3, fee3) = client.deduct_fee(&user, &2_000);
    assert_eq!(fee3, 100);
    assert_eq!(client.get_user_fees_accrued(&user), 190);
}

#[test]
fn test_user_fees_accrued_multiple_users() {
    let (env, _admin, client) = setup_fee_contract();
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    
    // User1 transactions
    client.deduct_fee(&user1, &1_000); // fee = 50
    client.deduct_fee(&user1, &2_000); // fee = 100
    
    // User2 transactions
    client.deduct_fee(&user2, &500); // fee = 25
    
    // User3 transactions
    client.deduct_fee(&user3, &10_000); // fee = 500
    client.deduct_fee(&user3, &200); // fee = 10
    
    // Verify each user's totals independently
    assert_eq!(client.get_user_fees_accrued(&user1), 150);
    assert_eq!(client.get_user_fees_accrued(&user2), 25);
    assert_eq!(client.get_user_fees_accrued(&user3), 510);
    
    // Total global fees should be 150 + 25 + 510 = 685
    assert_eq!(client.get_total_collected(), 685);
}

#[test]
fn test_user_fees_accrued_fee_percentage_change() {
    let (env, admin, client) = setup_fee_contract();
    let user = Address::generate(&env);
    
    // Initial fee percentage: 500 bps (5%)
    client.deduct_fee(&user, &1_000); // fee = 50
    assert_eq!(client.get_user_fees_accrued(&user), 50);
    
    // Change fee percentage to 1000 bps (10%)
    client.set_percentage(&admin, &1_000u32);
    
    // New transaction with higher fee
    client.deduct_fee(&user, &1_000); // fee = 100
    
    // User's accumulated fees should now be 50 + 100 = 150
    assert_eq!(client.get_user_fees_accrued(&user), 150);
    assert_eq!(client.get_total_collected(), 150);
}

#[test]
fn test_user_fees_accrued_large_amounts() {
    let (env, _admin, client) = setup_fee_contract();
    let user = Address::generate(&env);
    let large_amount: i128 = 100_000_000_000i128;
    
    // fee = 100_000_000_000 * 500 / 10_000 = 5_000_000_000
    let (_net, fee) = client.deduct_fee(&user, &large_amount);
    assert_eq!(fee, 5_000_000_000);
    assert_eq!(client.get_user_fees_accrued(&user), 5_000_000_000);
}

