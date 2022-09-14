#![no_std]

use soroban_sdk::{contractimpl, Bytes, BytesN, Env, RawVal, Symbol, Vec};

pub struct Deployer;

#[contractimpl]
impl Deployer {
    pub fn deploy(
        env: Env,
        salt: Bytes,
        wasm: Bytes,
        init_fn: Symbol,
        init_args: Vec<RawVal>,
    ) -> BytesN<32> {
        let id = env.deployer().from_current_contract(salt).deploy(wasm);
        let _: RawVal = env.invoke_contract(&id, &init_fn, init_args);
        id
    }
}

mod test;
