//! Tests for the is_protocol_config_initialized read-only method.

use creator_keys::{CreatorKeysContract, CreatorKeysContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_is_protocol_config_initialized_returns_false_when_unset() {
    let env = Env::default();
    let contract_id = env.register(CreatorKeysContract, ());
    let client = CreatorKeysContractClient::new(&env, &contract_id);

    assert!(!client.is_protocol_config_initialized());
}

#[test]
fn test_is_protocol_config_initialized_returns_true_after_fee_config_set() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreatorKeysContract, ());
    let client = CreatorKeysContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.set_fee_config(&admin, &9000u32, &1000u32);

    assert!(client.is_protocol_config_initialized());
}

#[test]
fn test_is_protocol_config_initialized_is_read_only() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreatorKeysContract, ());
    let client = CreatorKeysContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.set_fee_config(&admin, &8000u32, &2000u32);

    let first = client.is_protocol_config_initialized();
    let second = client.is_protocol_config_initialized();

    assert!(first);
    assert_eq!(first, second);
}
