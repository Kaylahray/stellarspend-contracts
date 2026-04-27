#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short,
    Address, Env, Vec,
};

// ── Error types ───────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TxError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
}

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Stores the contract admin address.
    Admin,
    /// Stores the global ordered list of transaction IDs (Vec<u64>).
    TxList,
    /// Stores the next auto-increment transaction ID counter.
    TxCounter,
    /// Stores the Transaction record for a given ID.
    Tx(u64),
    /// Stores the transaction count for a given user (sender).
    UserTxCount(Address),
}

// ── Data types ────────────────────────────────────────────────────────────────

/// A single on-chain transaction record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    pub id: u64,
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub timestamp: u64,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct TransactionContract;

#[contractimpl]
impl TransactionContract {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, TxError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TxCounter, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::TxList, &Vec::<u64>::new(&env));

        env.events().publish(
            (symbol_short!("txs"), symbol_short!("init")),
            admin,
        );
    }

    /// Record a new transaction between two parties.
    ///
    /// Emits a `("txs", "created")` event carrying the full Transaction
    /// struct so off-chain indexers can track every transfer without
    /// polling on-chain storage.
    ///
    /// Returns the assigned transaction ID.
    pub fn create_transaction(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> u64 {
        from.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, TxError::InvalidAmount);
        }

        // Assign an auto-incrementing ID.
        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TxCounter)
            .unwrap_or(0u64);

        let tx = Transaction {
            id,
            from: from.clone(),
            to: to.clone(),
            amount,
            timestamp: env.ledger().timestamp(),
        };

        // Persist the transaction record.
        env.storage().instance().set(&DataKey::Tx(id), &tx);

        // Append the ID to the global ordered list.
        let mut tx_list: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::TxList)
            .unwrap_or_else(|| Vec::new(&env));
        tx_list.push_back(id);
        env.storage().instance().set(&DataKey::TxList, &tx_list);

        // Advance the counter.
        env.storage()
            .instance()
            .set(&DataKey::TxCounter, &(id + 1));

        // Increment per-user transaction count for the sender.
        let user_count: i128 = env
            .storage()
            .instance()
            .get(&DataKey::UserTxCount(from.clone()))
            .unwrap_or(0i128);
        env.storage()
            .instance()
            .set(&DataKey::UserTxCount(from.clone()), &(user_count + 1));

        // Emit event.
        env.events().publish(
            (symbol_short!("txs"), symbol_short!("created")),
            tx.clone(),
        );

        id
    }

    // ── Issue: Fetch most recent transactions ─────────────────────────────────
    //
    // Returns up to `limit` of the most recently recorded transactions,
    // ordered newest-first. If fewer than `limit` transactions exist, all
    // available records are returned.
    //
    // Acceptance criteria: returns latest N transactions.
    pub fn get_recent_transactions(env: Env, limit: u32) -> Vec<Transaction> {
        let tx_list: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::TxList)
            .unwrap_or_else(|| Vec::new(&env));

        let total = tx_list.len();
        // How many records we will actually return (capped at total available).
        let count = if limit as u32 > total { total } else { limit as u32 };

        let mut result: Vec<Transaction> = Vec::new(&env);

        // Iterate from the tail of the list to get newest-first order.
        let start = total - count;
        let mut i = total;
        while i > start {
            i -= 1;
            let tx_id = tx_list.get(i).unwrap();
            if let Some(tx) = env.storage().instance().get(&DataKey::Tx(tx_id)) {
                result.push_back(tx);
            }
        }

        result
    }

    // ── Issue: Quick check for user activity ──────────────────────────────────
    //
    // Returns `true` if the given user has sent at least one transaction,
    // `false` otherwise. This is a cheap O(1) read against the per-user
    // counter maintained by `create_transaction`.
    //
    // Acceptance criteria: returns correct boolean state.
    pub fn has_user_transacted(env: Env, user: Address) -> bool {
        let count: i128 = env
            .storage()
            .instance()
            .get(&DataKey::UserTxCount(user))
            .unwrap_or(0i128);
        count > 0
    }

    /// Return the total number of transactions sent by a user.
    pub fn get_user_transaction_count(env: Env, user: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::UserTxCount(user))
            .unwrap_or(0i128)
    }

    /// Return the transaction record for a given ID, if it exists.
    pub fn get_transaction(env: Env, id: u64) -> Option<Transaction> {
        env.storage().instance().get(&DataKey::Tx(id))
    }

    /// Return the total number of recorded transactions.
    pub fn get_transaction_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TxCounter)
            .unwrap_or(0u64)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{TransactionContract, TransactionContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup<'a>() -> (Env, Address, TransactionContractClient<'a>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(TransactionContract, ());
        let client = TransactionContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    // ── get_recent_transactions ───────────────────────────────────────────────

    #[test]
    fn returns_empty_when_no_transactions() {
        let (_env, _admin, client) = setup();
        let recent = client.get_recent_transactions(&5);
        assert_eq!(recent.len(), 0);
    }

    #[test]
    fn returns_latest_n_transactions_newest_first() {
        let (env, _admin, client) = setup();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        // Create 5 transactions.
        for amount in 1i128..=5 {
            client.create_transaction(&alice, &bob, &amount);
        }

        // Ask for the latest 3.
        let recent = client.get_recent_transactions(&3);
        assert_eq!(recent.len(), 3);

        // Should be newest-first: amounts 5, 4, 3.
        assert_eq!(recent.get(0).unwrap().amount, 5);
        assert_eq!(recent.get(1).unwrap().amount, 4);
        assert_eq!(recent.get(2).unwrap().amount, 3);
    }

    #[test]
    fn clamps_limit_to_available_count() {
        let (env, _admin, client) = setup();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.create_transaction(&alice, &bob, &100);
        client.create_transaction(&alice, &bob, &200);

        // Requesting more than available should return all 2.
        let recent = client.get_recent_transactions(&50);
        assert_eq!(recent.len(), 2);
    }

    // ── has_user_transacted ───────────────────────────────────────────────────

    #[test]
    fn returns_false_for_user_with_no_transactions() {
        let (env, _admin, client) = setup();
        let stranger = Address::generate(&env);
        assert!(!client.has_user_transacted(&stranger));
    }

    #[test]
    fn returns_true_after_first_transaction() {
        let (env, _admin, client) = setup();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        assert!(!client.has_user_transacted(&alice));

        client.create_transaction(&alice, &bob, &42);

        assert!(client.has_user_transacted(&alice));
        // Recipient has no outgoing transactions — still false.
        assert!(!client.has_user_transacted(&bob));
    }

    #[test]
    fn transaction_count_increments_correctly() {
        let (env, _admin, client) = setup();
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.create_transaction(&alice, &bob, &10);
        client.create_transaction(&alice, &bob, &20);

        assert_eq!(client.get_user_transaction_count(&alice), 2);
        assert_eq!(client.get_user_transaction_count(&bob), 0);
    }
}