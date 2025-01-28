#![cfg(test)]

use super::{DecrementContract, DecrementContractClient};
use soroban_sdk::{testutils::Logs, Env};

extern crate std;

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, DecrementContract);
    let client = DecrementContractClient::new(&env, &contract_id);

    assert_eq!(client.decrement(), -2);
    assert_eq!(client.decrement(), -4);
    assert_eq!(client.decrement(), -6);

    std::println!("{}", env.logs().all().join("\n"));
}
