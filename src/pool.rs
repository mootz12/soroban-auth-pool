use soroban_auth::{Identifier, Signature, verify};
use soroban_sdk::{contractimpl, contracttype, BigInt, Env, BytesN, symbol};

use crate::{accounting::{get_collateral, set_collateral, get_liabilities, set_liabilities}, auth::{get_nonce, verify_and_consume_nonce}};

// ****** Contract Storage *****

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Token, // address of the token the pool operates with
    Nonce(Identifier),
    Liability(Identifier), // any tokens owed to the pool
    Collateral(Identifier) // credit for tokens from the pool
}

// ****** Contract *****

/// A pool of tokens.
/// 
/// Allows users to deposit the token and borrow on-behalf-of the depositor
/// from a separate address.
/// 
/// This is a completely contrived and unsafe example to showcase interesting auth mechanics in Soroban.
pub struct Pool;

pub trait PoolTrait {
    /// Initialize the pool with a token
    fn initialize(e: Env, token: BytesN<32>);

    /// The sender deposits tokens into the pool and is accredited
    /// the deposit amount as collateral.
    fn deposit(e: Env, amount: i64);

    /// The sender deposits tokens into the pool and is accredited
    /// the deposit amount as collateral.
    /// 
    /// Showcase permit-style auth technique with native token contract
    fn deposit_p(e: Env, token_approval_sig: Signature, amount: i64);

    /// The sender borrows tokens on-behalf-of another address that provides
    /// permission via a signed message that is valid until expiration
    /// 
    /// Signature(owner: Identifier, nonce: i64, receiver: Identifier, amount: i64, expiration: u64)
    /// 
    /// Showcase custom auth usage to implement "on-behalf-of"
    fn borrow_obo(e: Env, sig: Signature, amount: i64, expiration: u64);

    // ***** View *****

    /// Get the token for the pool
    fn get_token(e: Env) -> BytesN<32>;

    /// Get the current collateral a user has
    fn collateral(e: Env, id: Identifier) -> i64;

    /// Get the current liability a user has
    fn liability(e: Env, id: Identifier) -> i64;

    /// Fetch the current nonce for the identifier
    fn nonce(e: Env, id: Identifier) -> i64;
}

#[contractimpl]
impl PoolTrait for Pool {
    fn initialize(e: Env, token: BytesN<32>) {
        if e.data().has(DataKey::Token) {
            panic!("pool already initialized");
        }

        e.data().set(DataKey::Token, token);
    }

    /// Requires approval for `transfer_from` before running
    fn deposit(e: Env, amount: i64) {
        let sender = e.invoker();
        let sender_id = Identifier::from(sender);
        let token_client = get_token_client(&e);

        token_client.xfer_from(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &sender_id,
            &get_contract_id(&e),
            &BigInt::from_i64(&e, amount)
        );

        let cur_collateral = get_collateral(&e, sender_id.clone());
        set_collateral(&e, sender_id, cur_collateral + amount);
    }

    /// Runs without needing any approval logic (not even a permit!)
    fn deposit_p(e: Env, token_approval_sig: Signature, amount: i64) {
        let sig_id = token_approval_sig.identifier(&e);
        let token_client = get_token_client(&e);
        let amount_bi = &BigInt::from_i64(&e, amount);

        // run the approval
        let sender_nonce = token_client.nonce(&sig_id);
        token_client.approve(&token_approval_sig, &sender_nonce, &get_contract_id(&e), &amount_bi);

        // now the pool has the appropriate permissions to run `transfer_from`
        token_client.xfer_from(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &sig_id,
            &get_contract_id(&e),
            &BigInt::from_i64(&e, amount)
        );

        let cur_collateral = get_collateral(&e, sig_id.clone());
        set_collateral(&e, sig_id, cur_collateral + amount);
    }

    /// A signature gives permission to the sender to borrow funds from the signer's collateral balance!
    fn borrow_obo(e: Env, sig: Signature, amount: i64, expiration: u64) {
        // verify signature is not expired
        if expiration < e.ledger().timestamp() {
            panic!("expired signature")
        }
        
        // Verify that the signature signs and authorizes this invocation.
        let signer_id = sig.identifier(&e);
        let sender_id = Identifier::from(e.invoker());
        let nonce = get_nonce(&e, &signer_id);

        // by including the `sender` in the signature alongside the `contract_id` and function `symbol`
        // the signer can be ensured nobody other than the sender can execute against this signature
        verify(&e, &sig, symbol!("borrow_obo"), (&signer_id, &nonce, &sender_id, &amount, &expiration));
        verify_and_consume_nonce(&e, &sig, &nonce);

        // check collateral and liability balances
        let signer_collateral = get_collateral(&e, signer_id.clone());
        let signer_liability = get_liabilities(&e, signer_id.clone());
        if signer_collateral < (signer_liability + amount) {
            panic!("not enough collateral")
        }

        // looks good - execute token distribution
        set_liabilities(&e, signer_id, signer_liability + amount);

        let token_client = get_token_client(&e);
        token_client.xfer(&Signature::Invoker, &BigInt::zero(&e), &sender_id, &BigInt::from_i64(&e, amount));
    }

    // ***** View *****

    fn get_token(e: Env) -> BytesN<32> {
        get_token_id(&e)
    }

    fn collateral(e: Env, id: Identifier) -> i64 {
        get_collateral(&e, id)
    }

    fn liability(e: Env, id: Identifier) -> i64 {
        get_liabilities(&e, id)
    }

    fn nonce(e: Env, id: Identifier) -> i64 {
        get_nonce(&e, &id)
    }
}

// ****** Helpers *****

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract().into())
}

fn get_token_id(e: &Env) -> BytesN<32> {
    let key = DataKey::Token;
    e.data().get::<DataKey, BytesN<32>>(key).unwrap().unwrap()
}

 fn get_token_client(e: &Env) -> crate::token::Client {
    let id = get_token_id(e);
    crate::token::Client::new(e, id)
 }

