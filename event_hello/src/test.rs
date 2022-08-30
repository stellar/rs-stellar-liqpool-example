#![cfg(test)]

use super::*;
use soroban_sdk::{BytesN, Env};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = BytesN::from_array(&env, &[0; 32]);
    env.register_contract(&contract_id, EventContract);
    EventContractClient::new(&env, &contract_id).hello(&Symbol::from_str("SourBun"));
}
