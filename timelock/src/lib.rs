//! This contract demonstrates 'timelock' concept and implements a
//! greatly simplified Claimable Balance (similar to
//! https://developers.stellar.org/docs/glossary/claimable-balance).
//! The contract allows to deposit some amount of token and allow another
//! account(s) claim it before or after provided time point.
#![no_std]
#[cfg(feature = "testutils")]
extern crate std;

use soroban_auth::{
    check_auth, NonceAuth, {Identifier, Signature},
};
use soroban_sdk::{contractimpl, contracttype, BigInt, BytesN, Env, IntoVal, Symbol, Vec};

mod token {
    soroban_sdk::contractimport!(file = "../soroban_token_contract.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Init,
    Balance,
    Nonce(Identifier),
}

#[derive(Clone)]
#[contracttype]
pub enum TimeBoundKind {
    Before,
    After,
}

#[derive(Clone)]
#[contracttype]
pub struct TimeBound {
    pub kind: TimeBoundKind,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct ClaimableBalance {
    pub token: BytesN<32>,
    pub amount: BigInt,
    pub claimants: Vec<Identifier>,
    pub time_bound: TimeBound,
}

pub struct ClaimableBalanceContract;

// The 'timelock' part: check that provided timestamp is before/after
// the current ledger timestamp.
fn check_time_bound(env: &Env, time_bound: &TimeBound) -> bool {
    let ledger_timestamp = env.ledger().timestamp();

    match time_bound.kind {
        TimeBoundKind::Before => ledger_timestamp <= time_bound.timestamp,
        TimeBoundKind::After => ledger_timestamp >= time_bound.timestamp,
    }
}

// Contract usage pattern (pseudocode):
// 1. Depositor calls `token.approve(depositor_auth, claimable_balance_contract, 100)`
//    to allow contract to withdraw the needed amount of token.
// 2. Depositor calls `deposit(depositor_auth, token_id, 100, claimants, time bound)`. Contract
//    withdraws the provided token amount and stores it until one of the claimants
//    claims it.
// 3. Claimant calls `claim(claimant_auth)` and if time/auth conditons are passed
//    receives the balance.
#[contractimpl]
impl ClaimableBalanceContract {
    pub fn deposit(
        env: Env,
        from: Signature,
        token: BytesN<32>,
        amount: BigInt,
        claimants: Vec<Identifier>,
        time_bound: TimeBound,
    ) {
        if claimants.len() > 10 {
            panic!("too many claimants");
        }
        if is_initialized(&env) {
            panic!("contract has been already initialized");
        }

        let from_id = from.get_identifier(&env);
        // Authenticate depositor with nonce of zero, so that this may
        // be successfully called just once.
        check_auth(
            &env,
            &NonceForSignature(from),
            BigInt::zero(&env),
            Symbol::from_str("deposit"),
            (&from_id, &token, &amount, &claimants, &time_bound).into_val(&env),
        );
        // Transfer token to this contract address.
        transfer_from(&env, &token, &from_id, &get_contract_id(&env), &amount);
        // Store all the necessary balance to allow one of the claimants to claim it.
        env.contract_data().set(
            DataKey::Balance,
            ClaimableBalance {
                token,
                amount,
                time_bound,
                claimants,
            },
        );
        env.contract_data().set(DataKey::Init, ());
    }

    pub fn claim(env: Env, claimant: Signature) {
        let claimable_balance: ClaimableBalance =
            env.contract_data().get_unchecked(DataKey::Balance).unwrap();

        if !check_time_bound(&env, &claimable_balance.time_bound) {
            panic!("time predicate is not fulfilled");
        }

        let claimant_id = claimant.get_identifier(&env);
        let claimants = &claimable_balance.claimants;
        if !claimants.contains(&claimant_id) {
            panic!("claimant is not allowed to claim this balance");
        }
        // Authenticate claimant with nonce of zero, so that this may be
        // successfully called just once.
        // For simplicity, depositor can't be the claimant.
        check_auth(
            &env,
            &NonceForSignature(claimant),
            BigInt::zero(&env),
            Symbol::from_str("claim"),
            (&claimant_id,).into_val(&env),
        );
        // Transfer the stored amount of token to claimant after passing
        // all the checks.
        transfer_to(
            &env,
            &claimable_balance.token,
            &claimant_id,
            &claimable_balance.amount,
        );
        // Cleanup unnecessary balance entry.
        env.contract_data().remove(DataKey::Balance);
    }
}

fn is_initialized(env: &Env) -> bool {
    env.contract_data().has(DataKey::Init)
}

fn get_contract_id(e: &Env) -> Identifier {
    Identifier::Contract(e.get_current_contract().into())
}

fn transfer_from(
    e: &Env,
    token_id: &BytesN<32>,
    from: &Identifier,
    to: &Identifier,
    amount: &BigInt,
) {
    let client = token::ContractClient::new(&e, token_id);
    client.xfer_from(&Signature::Contract, &BigInt::zero(&e), &from, &to, &amount);
}

fn transfer_to(e: &Env, token_id: &BytesN<32>, to: &Identifier, amount: &BigInt) {
    let client = token::ContractClient::new(&e, token_id);
    client.xfer(&Signature::Contract, &BigInt::zero(&e), to, amount);
}

struct NonceForSignature(Signature);

impl NonceAuth for NonceForSignature {
    fn read_nonce(e: &Env, id: Identifier) -> BigInt {
        let key = DataKey::Nonce(id);
        if let Some(nonce) = e.contract_data().get(key) {
            nonce.unwrap()
        } else {
            BigInt::zero(e)
        }
    }

    fn read_and_increment_nonce(&self, e: &Env, id: Identifier) -> BigInt {
        let key = DataKey::Nonce(id.clone());
        let nonce = Self::read_nonce(e, id);
        e.contract_data()
            .set(key, nonce.clone() + BigInt::from_u32(e, 1));
        nonce
    }

    fn signature(&self) -> &Signature {
        &self.0
    }
}

mod test;
