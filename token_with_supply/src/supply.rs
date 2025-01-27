use soroban_sdk::Env;
use crate::storage_types::DataKey;
pub fn read_supply(e: &Env) -> i128 {
    let key = DataKey::Supply;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_supply(e: &Env, amount: &i128) {
    let key = DataKey::Supply;
    e.storage().instance().set(&key, amount);
}

pub fn decrement_supply(e: &Env, amount: &i128) {
    let key = DataKey::Supply;
    let current_supply: i128 = e.storage().instance().get(&key).unwrap();
    let new_supply = current_supply - amount;
    
    e.storage().instance().set(&key, &new_supply);
}

pub fn increment_supply(e: &Env, amount: &i128) {
    let key = DataKey::Supply;
    let current_supply: i128 = e.storage().instance().get(&key).unwrap();
    let new_supply = current_supply + amount;
    
    e.storage().instance().set(&key, &new_supply);
}