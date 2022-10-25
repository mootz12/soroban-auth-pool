#![cfg(test)]

use soroban_sdk::{BigInt, Env, testutils::{Accounts, Ledger, LedgerInfo}, symbol};
use soroban_auth::{Identifier, Signature, testutils::ed25519};

mod helper;
use helper::{create_token_contract, create_pool_contract, generate_contract_id};

/// TODO: Test support for accounts
///       blocked by: https://github.com/stellar/rs-soroban-sdk/issues/741

#[test]
fn test_borrow_on_behalf_of_happy_path() {
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

    let user2_acct = e.accounts().generate_and_create();
    let user2_id = Identifier::Account(user2_acct.clone());

    // deposit with permit
    let user1_token_nonce = token_client.nonce(&user1_id);
    let approval_sig = ed25519::sign(
        &e,
        &user1_sign,
        &token_contract_id,
        symbol!("approve"),
        (&user1_id, &user1_token_nonce, &pool_id, &deposit_amount),
    );
    pool_client.deposit_p(&approval_sig, &deposit_amount.to_i64());
    assert_eq!(token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token_client.balance(&pool_id), deposit_amount);
    assert_eq!(pool_client.collateral(&user1_id), deposit_amount.to_i64());
    println!("deposit with permit succesful");

    // borrow on behalf of
    let signer_nonce = pool_client.nonce(&user1_id);
    let expiration = e.ledger().timestamp() + 100;
    let sig = ed25519::sign(
        &e,
        &user1_sign,
        &pool_contract_id,
        symbol!("borrow_obo"),
        (&user1_id, &signer_nonce, &user2_id, &deposit_amount_i64, &expiration),
    );
    pool_client.with_source_account(&user2_acct).borrow_obo(&sig, &deposit_amount_i64, &expiration);

    assert_eq!(token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token_client.balance(&user2_id), deposit_amount);
    assert_eq!(token_client.balance(&pool_id), BigInt::zero(&e));
    assert_eq!(pool_client.liability(&user1_id), deposit_amount_i64);
    assert_eq!(pool_client.liability(&user2_id), 0);
}

#[test]
#[should_panic(expected = "expired signature")] 
fn test_borrow_on_behalf_of_invalid_expiration() {
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

    let user2_acct = e.accounts().generate_and_create();
    let user2_id = Identifier::Account(user2_acct.clone());

    // deposit with permit
    let user1_token_nonce = token_client.nonce(&user1_id);
    let approval_sig = ed25519::sign(
        &e,
        &user1_sign,
        &token_contract_id,
        symbol!("approve"),
        (&user1_id, &user1_token_nonce, &pool_id, &deposit_amount),
    );
    pool_client.deposit_p(&approval_sig, &deposit_amount.to_i64());
    assert_eq!(token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token_client.balance(&pool_id), deposit_amount);
    assert_eq!(pool_client.collateral(&user1_id), deposit_amount.to_i64());
    println!("deposit with permit succesful");

    // borrow on behalf of
    let signer_nonce = pool_client.nonce(&user1_id);
    e.ledger().set(LedgerInfo {
        timestamp: 12345,
        protocol_version: 1,
        sequence_number: 10,
        network_passphrase: Default::default(),
        base_reserve: 10,
    });
    let expiration = e.ledger().timestamp() - 100;
    let sig = ed25519::sign(
        &e,
        &user1_sign,
        &pool_contract_id,
        symbol!("borrow_obo"),
        (&user1_id, &signer_nonce, &user2_id, &deposit_amount_i64, &expiration),
    );
    pool_client.with_source_account(&user2_acct).borrow_obo(&sig, &deposit_amount_i64, &expiration);
}

#[test]
#[should_panic(expected = "not enough collateral")] 
fn test_borrow_on_behalf_of_too_large() {
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

    let user2_acct = e.accounts().generate_and_create();
    let user2_id = Identifier::Account(user2_acct.clone());

    // deposit with permit
    let user1_token_nonce = token_client.nonce(&user1_id);
    let approval_sig = ed25519::sign(
        &e,
        &user1_sign,
        &token_contract_id,
        symbol!("approve"),
        (&user1_id, &user1_token_nonce, &pool_id, &deposit_amount),
    );
    pool_client.deposit_p(&approval_sig, &deposit_amount.to_i64());
    assert_eq!(token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token_client.balance(&pool_id), deposit_amount);
    assert_eq!(pool_client.collateral(&user1_id), deposit_amount.to_i64());
    println!("deposit with permit succesful");

    // borrow on behalf of
    let signer_nonce = pool_client.nonce(&user1_id);
    let expiration = e.ledger().timestamp() + 100;
    let sig = ed25519::sign(
        &e,
        &user1_sign,
        &pool_contract_id,
        symbol!("borrow_obo"),
        (&user1_id, &signer_nonce, &user2_id, &(deposit_amount_i64 + 1), &expiration),
    );
    pool_client.with_source_account(&user2_acct).borrow_obo(&sig, &(deposit_amount_i64 + 1), &expiration);
}
