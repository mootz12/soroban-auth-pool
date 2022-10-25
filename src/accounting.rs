use crate::pool::DataKey;
use soroban_auth::Identifier;
use soroban_sdk::{Env};

pub fn get_collateral(e: &Env, id: Identifier) -> i64 {
    let key = DataKey::Collateral(id);
    if let Some(balance) = e.data().get::<DataKey, i64>(key) {
        balance.unwrap()
    } else {
        0
    }
}

pub fn set_collateral(e: &Env, id: Identifier, amount: i64) {
    let key = DataKey::Collateral(id);
    e.data().set::<DataKey, i64>(key, amount);
}

pub fn get_liabilities(e: &Env, id: Identifier) -> i64 {
    let key = DataKey::Liability(id);
    if let Some(balance) = e.data().get::<DataKey, i64>(key) {
        balance.unwrap()
    } else {
        0
    }
}

pub fn set_liabilities(e: &Env, id: Identifier, amount: i64) {
    let key = DataKey::Liability(id);
    e.data().set::<DataKey, i64>(key, amount);
}