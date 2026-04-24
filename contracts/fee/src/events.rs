use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol};

use crate::storage::{DEFAULT_FEE_BPS, DEFAULT_MIN_FEE};
use crate::utils::format_amount;

pub struct TierEvents;

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct FeeResetEventData {
    pub admin: Address,
    pub fee_bps: u32,
    pub min_fee: i128,
    pub formatted_min_fee: String,
}

pub struct ConfigEvents;

impl ConfigEvents {
    /// Emitted when an admin resets fee configuration to defaults.
    pub fn fee_reset(env: &Env, admin: &Address) {
        let topics = (symbol_short!("fee"), symbol_short!("reset"));
        env.events().publish(
            topics,
            FeeResetEventData {
                admin: admin.clone(),
                fee_bps: DEFAULT_FEE_BPS,
                min_fee: DEFAULT_MIN_FEE,
                formatted_min_fee: format_amount(env, DEFAULT_MIN_FEE),
            },
        );
    }
}
