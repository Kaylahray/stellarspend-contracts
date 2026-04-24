use soroban_sdk::{contracttype, Address, Env, Map, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Set of all unique users who have interacted with the contract
    Users,
    /// Count of unique users
    UserCount,
}

/// Add a user to the set of unique users if not already present
pub fn add_user(env: &Env, user: Address) -> bool {
    let mut users: Map<Address, bool> = env
        .storage()
        .persistent()
        .get(&DataKey::Users)
        .unwrap_or_else(|| Map::new(env));
    
    // If user already exists, return false
    if users.contains_key(user.clone()) {
        return false;
    }
    
    // Add the user
    users.set(user.clone(), true);
    
    // Update storage
    env.storage()
        .persistent()
        .set(&DataKey::Users, &users);
    
    // Update count
    let mut count: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::UserCount)
        .unwrap_or(0);
    count += 1;
    env.storage()
        .persistent()
        .set(&DataKey::UserCount, &count);
    
    true
}

/// Get the total count of unique users
pub fn get_user_count(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::UserCount)
        .unwrap_or(0)
}

/// Check if a user exists in the set
pub fn user_exists(env: &Env, user: Address) -> bool {
    let users: Map<Address, bool> = env
        .storage()
        .persistent()
        .get(&DataKey::Users)
        .unwrap_or_else(|| Map::new(env));
    
    users.contains_key(user)
}

/// Get all unique users (for admin purposes)
pub fn get_all_users(env: &Env) -> Vec<Address> {
    let users: Map<Address, bool> = env
        .storage()
        .persistent()
        .get(&DataKey::Users)
        .unwrap_or_else(|| Map::new(env));
    
    let mut result = Vec::new(env);
    for (user, _) in users.iter() {
        result.push_back(user);
    }
    result
}
