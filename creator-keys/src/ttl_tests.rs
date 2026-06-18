use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

fn setup_registered_creator() -> (Env, CreatorKeysContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CreatorKeysContract, ());
    let client = CreatorKeysContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_key_price(&admin, &100);
    client.set_fee_config(&admin, &9000, &1000);

    let creator = Address::generate(&env);
    let handle = String::from_str(&env, "alice");
    client.register_creator(&creator, &handle);

    let holder = Address::generate(&env);
    (env, client, creator, holder)
}

fn advance_near_ttl_threshold(env: &Env) {
    env.ledger().with_mut(|li| {
        li.sequence_number += config::CREATOR_TTL_LEDGERS - config::CREATOR_TTL_THRESHOLD_LEDGERS + 10;
    });
}

#[test]
fn register_creator_sets_initial_creator_ttl() {
    let (_env, client, creator, _holder) = setup_registered_creator();

    let ttl = client.get_creator_ttl_remaining(&creator);

    assert!(ttl > 0);
}

#[test]
fn successful_buy_extends_creator_ttl() {
    let (env, client, creator, holder) = setup_registered_creator();
    advance_near_ttl_threshold(&env);

    let before_buy = client.get_creator_ttl_remaining(&creator);
    client.buy_key(&creator, &holder, &100);
    let after_buy = client.get_creator_ttl_remaining(&creator);

    assert!(before_buy < config::CREATOR_TTL_THRESHOLD_LEDGERS);
    assert!(after_buy > before_buy);
}

#[test]
fn successful_sell_extends_creator_ttl() {
    let (env, client, creator, holder) = setup_registered_creator();
    client.buy_key(&creator, &holder, &100);
    advance_near_ttl_threshold(&env);

    let before_sell = client.get_creator_ttl_remaining(&creator);
    client.sell_key(&creator, &holder);
    let after_sell = client.get_creator_ttl_remaining(&creator);

    assert!(before_sell < config::CREATOR_TTL_THRESHOLD_LEDGERS);
    assert!(after_sell > before_sell);
}

#[test]
fn failed_buy_does_not_extend_creator_ttl() {
    let (env, client, creator, holder) = setup_registered_creator();
    advance_near_ttl_threshold(&env);

    let before_failed_buy = client.get_creator_ttl_remaining(&creator);
    let result = client.try_buy_key(&creator, &holder, &99);
    let after_failed_buy = client.get_creator_ttl_remaining(&creator);

    assert_eq!(result, Err(Ok(ContractError::InsufficientPayment)));
    assert_eq!(after_failed_buy, before_failed_buy);
}

#[test]
fn failed_sell_does_not_extend_creator_ttl() {
    let (env, client, creator, holder) = setup_registered_creator();
    advance_near_ttl_threshold(&env);

    let before_failed_sell = client.get_creator_ttl_remaining(&creator);
    let result = client.try_sell_key(&creator, &holder);
    let after_failed_sell = client.get_creator_ttl_remaining(&creator);

    assert_eq!(result, Err(Ok(ContractError::InsufficientBalance)));
    assert_eq!(after_failed_sell, before_failed_sell);
}
