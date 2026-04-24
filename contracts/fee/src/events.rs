use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub struct TierEvents;

pub struct FeeConfigEvents;

impl TierEvents {
    /// Emitted when an admin assigns a tier to a user.
    pub fn tier_set(env: &Env, admin: &Address, user: &Address, tier: &Symbol) {
        let topics = (symbol_short!("tier"), symbol_short!("set"));
        env.events()
            .publish(topics, (admin.clone(), user.clone(), tier.clone()));
    }

    /// Emitted when an admin removes a tier from a user.
    pub fn tier_removed(env: &Env, admin: &Address, user: &Address) {
        let topics = (symbol_short!("tier"), symbol_short!("removed"));
        env.events().publish(topics, (admin.clone(), user.clone()));
    }

    /// Emitted when an admin resets fee configuration to defaults.
    pub fn fee_config_reset(env: &Env, admin: &Address) {
        let topics = (symbol_short!("fee"), symbol_short!("reset"));
        env.events().publish(topics, admin.clone());
    }
}

impl FeeConfigEvents {
    /// Emitted when fee configuration is updated.
    pub fn fee_config_updated(
        env: &Env,
        admin: &Address,
        fee_bps: Option<u32>,
        min_fee: Option<i128>,
        max_fee: Option<i128>,
    ) {
        let topics = (symbol_short!("fee"), symbol_short!("config_up"));
        env.events().publish(
            topics,
            (
                admin.clone(),
                fee_bps.unwrap_or(0),
                min_fee.unwrap_or(0),
                max_fee.unwrap_or(0),
            ),
        );
    }
}
