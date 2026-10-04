#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct DataEntry {
    pub key: Symbol,
    pub value: Bytes,
    pub ttl_ledgers: u32,
    pub created_at: u32,
}

const ENTRIES: Symbol = symbol_short!("ENTRIES");
const ADMIN: Symbol = symbol_short!("ADMIN");
const LIVE_LEDGERS: u32 = 500_000;

#[contract]
pub struct TtlManagement;

#[contractimpl]
impl TtlManagement {
    pub fn init(env: Env, admin: Address) {
        env.storage().instance().set(&ADMIN, &admin);
        let empty: Vec<Symbol> = Vec::new(&env);
        env.storage().instance().set(&ENTRIES, &empty);
    }

    fn only_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        caller.require_auth();
        assert_eq!(caller.clone(), admin, "TTL: caller is not admin");
    }

    pub fn write(env: Env, caller: Address, key: Symbol, value: Bytes, ttl_ledgers: u32) {
        Self::only_admin(&env, &caller);
        assert!(ttl_ledgers > 0, "TTL: ttl_ledgers must be positive");

        let entry = DataEntry {
            key: key.clone(),
            value,
            ttl_ledgers,
            created_at: env.ledger().sequence(),
        };

        env.storage().persistent().set(&key, &entry);
        env.storage()
            .persistent()
            .extend_ttl(&key, ttl_ledgers, ttl_ledgers + LIVE_LEDGERS);

        let mut entries: Vec<Symbol> = env.storage().instance().get(&ENTRIES).unwrap();
        entries.push_back(key);
        env.storage().instance().set(&ENTRIES, &entries);
    }

    pub fn read(env: &Env, key: Symbol) -> Option<DataEntry> {
        env.storage().persistent().get(&key)
    }

    pub fn remove(env: Env, caller: Address, key: Symbol) {
        Self::only_admin(&env, &caller);
        env.storage().persistent().remove(&key);
        let entries: Vec<Symbol> = env.storage().instance().get(&ENTRIES).unwrap();
        let mut filtered: Vec<Symbol> = Vec::new(&env);
        for k in entries.iter() {
            if k != key {
                filtered.push_back(k);
            }
        }
        env.storage().instance().set(&ENTRIES, &filtered);
    }

    pub fn extend_ttl(env: Env, caller: Address, key: Symbol, ttl_ledgers: u32) {
        Self::only_admin(&env, &caller);
        assert!(ttl_ledgers > 0, "TTL: ttl_ledgers must be positive");
        let mut entry: DataEntry = env.storage().persistent().get(&key).unwrap();
        entry.ttl_ledgers = ttl_ledgers;
        entry.created_at = env.ledger().sequence();
        env.storage().persistent().set(&key, &entry);
        env.storage()
            .persistent()
            .extend_ttl(&key, ttl_ledgers, ttl_ledgers + LIVE_LEDGERS);
    }

    pub fn entries(env: &Env) -> Vec<Symbol> {
        env.storage().instance().get(&ENTRIES).unwrap()
    }

    pub fn remaining_ttl(env: &Env, key: Symbol) -> Option<u32> {
        let entry: DataEntry = env.storage().persistent().get(&key)?;
        let elapsed = env.ledger().sequence().saturating_sub(entry.created_at);
        let total_ttl = entry.ttl_ledgers.saturating_add(LIVE_LEDGERS);
        Some(total_ttl.saturating_sub(elapsed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn create_env() -> (Env, Address) {
        let env = Env::default();
        let contract_id = env.register(TtlManagement, ());
        (env, contract_id)
    }

    #[test]
    fn test_write_and_read() {
        let (env, contract_id) = create_env();
        env.mock_all_auths();
        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            TtlManagement::init(env.clone(), admin.clone());

            let key = symbol_short!("DATA1");
            let value = Bytes::from_array(&env, &[1, 2, 3, 4]);
            TtlManagement::write(
                env.clone(),
                admin.clone(),
                key.clone(),
                value.clone(),
                100_000,
            );

            let entry = TtlManagement::read(&env, key.clone()).unwrap();
            assert_eq!(entry.value, value);
        });
    }

    #[test]
    fn test_remove() {
        let (env, contract_id) = create_env();
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            let key = symbol_short!("DATA1");
            TtlManagement::init(env.clone(), admin.clone());

            let value = Bytes::from_array(&env, &[1, 2, 3]);
            TtlManagement::write(env.clone(), admin.clone(), key.clone(), value, 100_000);

            // Change admin to allow a fresh auth frame for remove
            let admin2 = Address::generate(&env);
            env.storage().instance().set(&ADMIN, &admin2);
            TtlManagement::remove(env.clone(), admin2.clone(), key.clone());
            assert!(TtlManagement::read(&env, key).is_none());
        });
    }

    #[test]
    fn test_remaining_ttl_includes_live_ledgers_and_tracks_extension() {
        use soroban_sdk::testutils::Ledger as _;

        let (env, contract_id) = create_env();
        env.mock_all_auths();
        let client = TtlManagementClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.init(&admin);

        let key = symbol_short!("DATA_TTL");
        let value = Bytes::from_array(&env, &[42]);
        let write_ttl = 100_000u32;

        // Initial state: non-existent key returns None
        assert_eq!(client.remaining_ttl(&key), None);

        // Set ledger sequence to 10 and write entry
        env.ledger().set_sequence_number(10);
        client.write(&admin, &key, &value, &write_ttl);

        // Formula: ttl_ledgers + LIVE_LEDGERS - elapsed
        // At sequence 10, elapsed = 0 -> remaining = 100_000 + 500_000 = 600_000
        let expected_initial = write_ttl + LIVE_LEDGERS;
        assert_eq!(client.remaining_ttl(&key), Some(expected_initial));

        // Advance ledger sequence by 20 ledgers (to 30)
        env.ledger().set_sequence_number(30);
        let expected_after_advance = expected_initial - 20;
        assert_eq!(client.remaining_ttl(&key), Some(expected_after_advance));

        // Advance further by 30 ledgers (to 60)
        env.ledger().set_sequence_number(60);
        assert_eq!(client.remaining_ttl(&key), Some(expected_initial - 50));

        // Extend TTL to 200,000 ledgers at sequence 60
        let new_ttl = 200_000u32;
        client.extend_ttl(&admin, &key, &new_ttl);

        // Elapsed is reset relative to sequence 60, remaining is now new_ttl + LIVE_LEDGERS
        let expected_after_extend = new_ttl + LIVE_LEDGERS;
        assert_eq!(client.remaining_ttl(&key), Some(expected_after_extend));

        // Advance by 40 ledgers to 100
        env.ledger().set_sequence_number(100);
        assert_eq!(client.remaining_ttl(&key), Some(expected_after_extend - 40));
    }
}
