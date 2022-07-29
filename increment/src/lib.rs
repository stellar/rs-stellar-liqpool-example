#![no_std]
use soroban_sdk::{contractimpl, Env, Symbol};

const COUNTER: Symbol = Symbol::from_str("COUNTER");

pub struct IncrementContract;

#[contractimpl(export_if = "export")]
impl IncrementContract {
    /// Increment increments an internal counter, and returns the value.
    pub fn increment(env: Env) -> u32 {
        // Get the current count.
        let mut count: u32 = env
            .contract_data()
            .get(COUNTER)
            .unwrap_or(Ok(0)) // If no value set, assume 0.
            .unwrap(); // Panic if the value of COUNTER is not u32.

        // Increment the count.
        count += 1;

        // Save the count.
        env.contract_data().set(COUNTER, count);

        // Return the count to the caller.
        count
    }
}

mod test;
