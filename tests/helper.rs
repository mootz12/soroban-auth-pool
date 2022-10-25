use soroban_auth_pool::{token, pool::{Pool, PoolClient}};

use rand::{thread_rng, RngCore};
use soroban_sdk::{BytesN, Env, AccountId, IntoVal};
use soroban_auth::Identifier;

pub fn generate_contract_id(e: &Env) -> BytesN<32> {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    BytesN::from_array(e, &id)
}

pub fn create_token_contract(e: &Env, contract_id: &BytesN<32>, admin: &AccountId) -> token::Client {
    e.register_contract_token(contract_id);

    let token = token::Client::new(e, contract_id);
    token.init(
        &Identifier::Account(admin.clone()),
        &token::TokenMetadata {
            name: "unit".into_val(e),
            symbol: "test".into_val(e),
            decimals: 7,
        },
    );
    token
}

pub fn create_pool_contract(e: &Env, contract_id: &BytesN<32>) -> PoolClient {
    e.register_contract(contract_id, Pool {});
    return PoolClient::new(e, contract_id);
}

