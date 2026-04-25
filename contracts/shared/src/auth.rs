use soroban_sdk::{Address, Env};

// ========== ADMIN ROLE CHECK (Issue #337) ==========

/// Check if the given address is the admin
pub fn is_admin(env: &Env, caller: Address) -> bool {
    let admin_key = "admin";
    let admin: Address = env.storage().instance().get(&admin_key).unwrap();
    caller == admin
}

/// Require that the caller is admin, otherwise panic
pub fn require_admin(env: &Env, caller: &Address) {
    assert!(is_admin(env, caller.clone()), "not authorized: admin only");
}