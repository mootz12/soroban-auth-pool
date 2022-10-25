#![cfg(test)]

use soroban_sdk::{BigInt, Env, testutils::Accounts};
use soroban_auth::{Identifier, Signature};

mod helper;
use helper::{create_token_contract, create_pool_contract, generate_contract_id};

#[test]
fn test_deposit_happy_path() {
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
    let user1_acct = e.accounts().generate_and_create();
    let user1_id = Identifier::Account(user1_acct.clone());
    token_client.with_source_account(&token_admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&e),
        &user1_id,
        &deposit_amount,
    );
    assert_eq!(token_client.balance(&user1_id), deposit_amount);

    token_client.with_source_account(&user1_acct).approve(
        &Signature::Invoker, 
        &BigInt::zero(&e), 
        &pool_id, 
        &deposit_amount
    );

    // deposit
    pool_client.with_source_account(&user1_acct).deposit(&deposit_amount_i64);

    assert_eq!(token_client.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token_client.balance(&pool_id), deposit_amount);
    assert_eq!(pool_client.collateral(&user1_id), deposit_amount_i64);
}
