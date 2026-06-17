//! Tests for creator storage TTL initialization and renewal.

use creator_keys::{
    config::CREATOR_TTL_LEDGERS, constants, ContractError, CreatorKeysContract,
    CreatorKeysContractClient,
};
use soroban_sdk::{
    testutils::{storage::Persistent, Address as _, Ledger},
    Address, Env, String,
};

fn setup(env: &Env) -> (CreatorKeysContractClient<'_>, Address, Address, Address) {
    let contract_id = env.register(CreatorKeysContract, ());
    let client = CreatorKeysContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let creator = Address::generate(env);
    let trader = Address::generate(env);

    client.set_key_price(&admin, &100_i128);
    client.set_fee_config(&admin, &9_000_u32, &1_000_u32);
    client.register_creator(&creator, &String::from_str(env, "alice"));

    (client, contract_id, creator, trader)
}

fn creator_ttl(env: &Env, contract_id: &Address, creator: &Address) -> u32 {
    let key = constants::storage::creator(creator);
    env.as_contract(contract_id, || env.storage().persistent().get_ttl(&key))
}

fn holder_ttl(env: &Env, contract_id: &Address, creator: &Address, holder: &Address) -> u32 {
    let key = constants::storage::key_balance(creator, holder);
    env.as_contract(contract_id, || env.storage().persistent().get_ttl(&key))
}

fn fee_config_ttl(env: &Env, contract_id: &Address) -> u32 {
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .get_ttl(&constants::storage::FEE_CONFIG)
    })
}

#[test]
fn test_register_creator_sets_initial_ttl() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, creator, _trader) = setup(&env);

    assert_eq!(
        client.get_creator_ttl_remaining(&creator),
        CREATOR_TTL_LEDGERS
    );
    assert_eq!(
        creator_ttl(&env, &contract_id, &creator),
        CREATOR_TTL_LEDGERS
    );
}

#[test]
fn test_successful_buy_extends_creator_storage_ttls() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, creator, buyer) = setup(&env);

    env.ledger().set_sequence_number(100);
    client.buy_key(&creator, &buyer, &100_i128);

    assert_eq!(
        creator_ttl(&env, &contract_id, &creator),
        CREATOR_TTL_LEDGERS
    );
    assert_eq!(
        client.get_creator_ttl_remaining(&creator),
        CREATOR_TTL_LEDGERS
    );
    assert_eq!(
        holder_ttl(&env, &contract_id, &creator, &buyer),
        CREATOR_TTL_LEDGERS
    );
    assert_eq!(fee_config_ttl(&env, &contract_id), CREATOR_TTL_LEDGERS);
}

#[test]
fn test_successful_sell_extends_creator_storage_ttls() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, creator, seller) = setup(&env);
    client.buy_key(&creator, &seller, &100_i128);

    env.ledger().set_sequence_number(200);
    client.sell_key(&creator, &seller);

    assert_eq!(
        creator_ttl(&env, &contract_id, &creator),
        CREATOR_TTL_LEDGERS
    );
    assert_eq!(
        client.get_creator_ttl_remaining(&creator),
        CREATOR_TTL_LEDGERS
    );
    assert_eq!(
        holder_ttl(&env, &contract_id, &creator, &seller),
        CREATOR_TTL_LEDGERS
    );
    assert_eq!(fee_config_ttl(&env, &contract_id), CREATOR_TTL_LEDGERS);
}

#[test]
fn test_failed_buy_does_not_extend_ttls() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, creator, buyer) = setup(&env);

    env.ledger().set_sequence_number(100);
    let result = client.try_buy_key(&creator, &buyer, &99_i128);
    assert_eq!(result, Err(Ok(ContractError::InsufficientPayment)));

    assert_eq!(
        creator_ttl(&env, &contract_id, &creator),
        CREATOR_TTL_LEDGERS - 100
    );
    assert_eq!(
        client.get_creator_ttl_remaining(&creator),
        CREATOR_TTL_LEDGERS - 100
    );
    assert_eq!(
        fee_config_ttl(&env, &contract_id),
        CREATOR_TTL_LEDGERS - 100
    );
}

#[test]
fn test_failed_sell_does_not_extend_ttls() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, contract_id, creator, seller) = setup(&env);
    client.buy_key(&creator, &seller, &100_i128);

    env.ledger().set_sequence_number(100);
    let result = client.try_sell_key(&creator, &Address::generate(&env));
    assert_eq!(result, Err(Ok(ContractError::InsufficientBalance)));

    assert_eq!(
        creator_ttl(&env, &contract_id, &creator),
        CREATOR_TTL_LEDGERS - 100
    );
    assert_eq!(
        client.get_creator_ttl_remaining(&creator),
        CREATOR_TTL_LEDGERS - 100
    );
    assert_eq!(
        holder_ttl(&env, &contract_id, &creator, &seller),
        CREATOR_TTL_LEDGERS - 100
    );
    assert_eq!(
        fee_config_ttl(&env, &contract_id),
        CREATOR_TTL_LEDGERS - 100
    );
}
