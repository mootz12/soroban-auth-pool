#![cfg(test)]

use soroban_sdk::{BigInt, Env, testutils::Accounts, symbol};
use soroban_auth::{Identifier, Signature, testutils::ed25519};

mod helper;
use helper::{create_token_contract, create_pool_contract, generate_contract_id};

/// TODO: Test support for accounts
///       blocked by: https://github.com/stellar/rs-soroban-sdk/issues/741

#[test]
fn test_deposit_permit_happy_path() {
    let e = Env::default();
    let deposit_amount_i64 = 123456789;
    let deposit_amount = BigInt::from_i64(&e, deposit_amount_i64);

    // deploy token contract
    let token_admin = e.accounts().generate_and_create();
    let token_contract_id = generate_contract_id(&e);
    let token_client = create_token_contract(&e, &token_contract_id, &token_admin);

    // deploy and init auth pool
    let pool_contract_id = generate_contract_id(&e);
    let pool_id = Identifier::Contract(pool_contract_id.clone());
    let pool_client = create_pool_contract(&e, &pool_contract_id);
    pool_client.initialize(&token_contract_id);

    // setup env
    let (user1_id, user1_sign) = ed25519::generate(&e);
    token_client.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &deposit_amount,
    );
    assert_eq!(token_client.balance(&user1_id), deposit_amount);

    // deposit with permit
    let user1_token_nonce = token_client.nonce(&user1_id);
    let approval_sig = ed25519::sign(
        &e,
        &user1_sign,
        &token_contract_id,
        symbol!("approve"),
        (&user1_id, &user1_token_nonce, &pool_id, &deposit_amount),
    );
    pool_client.deposit_p(&approval_sig, &deposit_amount_i64);

    assert_eq!(token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token_client.balance(&pool_id), deposit_amount);
    assert_eq!(pool_client.collateral(&user1_id), deposit_amount_i64);
}

#[test]
#[should_panic(expected = "Failed ED25519 verification")]
fn test_deposit_permit_bad_signature() {
    let e = Env::default();
    let deposit_amount_i64 = 123456789;
    let deposit_amount = BigInt::from_i64(&e, deposit_amount_i64);

    // deploy token contract
    let token_admin = e.accounts().generate_and_create();
    let token_contract_id = generate_contract_id(&e);
    let token_client = create_token_contract(&e, &token_contract_id, &token_admin);

    // deploy and init auth pool
    let pool_contract_id = generate_contract_id(&e);
    let pool_id = Identifier::Contract(pool_contract_id.clone());
    let pool_client = create_pool_contract(&e, &pool_contract_id);
    pool_client.initialize(&token_contract_id);

    // setup env
    let (user1_id, _user1_sign) = ed25519::generate(&e);
    let (_evil_id, evil_sign) = ed25519::generate(&e);
    token_client.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &deposit_amount,
    );
    assert_eq!(token_client.balance(&user1_id), deposit_amount);

    // deposit with permit
    // - build approval signature
    let user1_token_nonce = token_client.nonce(&user1_id);
    let approval_sig = ed25519::sign(
        &e,
        &evil_sign,
        &token_contract_id,
        symbol!("approve"),
        (&user1_id, &user1_token_nonce, &pool_id, &deposit_amount),
    );

    // - call deposit with permit
    pool_client.deposit_p(&approval_sig, &deposit_amount_i64);
}
