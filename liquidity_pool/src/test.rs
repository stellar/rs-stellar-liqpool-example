#![cfg(test)]

use crate::testutils::{register_test_contract as register_liqpool, LiquidityPool};
use ed25519_dalek::Keypair;
use rand::{thread_rng, RngCore};
use soroban_auth::Identifier;
use soroban_sdk::{BigInt, BytesN, Env};
use soroban_token_contract::testutils::{to_ed25519, Token};

fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    id
}

fn generate_sorted_contract_ids() -> ([u8; 32], [u8; 32]) {
    let a = generate_contract_id();
    let b = generate_contract_id();
    if a < b {
        (a, b)
    } else if a == b {
        generate_sorted_contract_ids()
    } else {
        (b, a)
    }
}

fn generate_keypair() -> Keypair {
    Keypair::generate(&mut thread_rng())
}

fn create_token_contract(e: &Env, id: &[u8; 32], admin: &Keypair) -> Token {
    #[cfg(not(all(any(test, feature = "testutils"), not(feature = "token-wasm"))))]
    {
        soroban_token_contract::testutils::register_test_contract(&e, id);
    }
    #[cfg(all(any(test, feature = "testutils"), not(feature = "token-wasm")))]
    {
        let contract_id = BytesN::from_array(e, &id);
        e.register_contract_wasm(&contract_id, crate::token_contract::WASM);
    }
    let token = Token::new(e, id);
    // decimals, name, symbol don't matter in tests
    token.initialize(&to_ed25519(&e, admin), 7, "name", "symbol");
    token
}

fn create_liqpool_contract(
    e: &Env,
    token_a: &[u8; 32],
    token_b: &[u8; 32],
) -> ([u8; 32], LiquidityPool) {
    let id = generate_contract_id();
    register_liqpool(&e, &id);
    let liqpool = LiquidityPool::new(e, &id);
    liqpool.initialize(token_a, token_b);
    (id, liqpool)
}

#[test]
fn test() {
    let e: Env = Default::default();

    let admin1 = generate_keypair();
    let admin2 = generate_keypair();
    let user1 = generate_keypair();
    let user1_id = to_ed25519(&e, &user1);

    let (contract1, contract2) = generate_sorted_contract_ids();
    let token1 = create_token_contract(&e, &contract1, &admin1);
    let token2 = create_token_contract(&e, &contract2, &admin2);
    let (contract_pool, liqpool) = create_liqpool_contract(&e, &contract1, &contract2);
    let pool_id = Identifier::Contract(BytesN::from_array(&e, &contract_pool));
    let contract_share: [u8; 32] = liqpool.share_id().into();
    let token_share = Token::new(&e, &contract_share);

    token1.mint(&admin1, &user1_id, &BigInt::from_u32(&e, 1000));
    assert_eq!(token1.balance(&user1_id), BigInt::from_u32(&e, 1000));
    token2.mint(&admin2, &user1_id, &BigInt::from_u32(&e, 1000));
    assert_eq!(token2.balance(&user1_id), BigInt::from_u32(&e, 1000));

    token1.xfer(&user1, &pool_id, &BigInt::from_u32(&e, 100));
    assert_eq!(token1.balance(&user1_id), BigInt::from_u32(&e, 900));
    assert_eq!(token1.balance(&pool_id), BigInt::from_u32(&e, 100));
    token2.xfer(&user1, &pool_id, &BigInt::from_u32(&e, 100));
    assert_eq!(token2.balance(&user1_id), BigInt::from_u32(&e, 900));
    assert_eq!(token2.balance(&pool_id), BigInt::from_u32(&e, 100));
    liqpool.deposit(&user1_id);
    assert_eq!(token_share.balance(&user1_id), BigInt::from_u32(&e, 100));
    assert_eq!(token_share.balance(&pool_id), BigInt::zero(&e));

    token1.xfer(&user1, &pool_id, &BigInt::from_u32(&e, 100));
    assert_eq!(token1.balance(&user1_id), BigInt::from_u32(&e, 800));
    assert_eq!(token1.balance(&pool_id), BigInt::from_u32(&e, 200));
    liqpool.swap(&user1_id, &BigInt::zero(&e), &BigInt::from_u32(&e, 49));
    assert_eq!(token1.balance(&user1_id), BigInt::from_u32(&e, 800));
    assert_eq!(token1.balance(&pool_id), BigInt::from_u32(&e, 200));
    assert_eq!(token2.balance(&user1_id), BigInt::from_u32(&e, 949));
    assert_eq!(token2.balance(&pool_id), BigInt::from_u32(&e, 51));

    token_share.xfer(&user1, &pool_id, &BigInt::from_u32(&e, 100));
    assert_eq!(token_share.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token_share.balance(&pool_id), BigInt::from_u32(&e, 100));
    liqpool.withdraw(&user1_id);
    assert_eq!(token1.balance(&user1_id), BigInt::from_u32(&e, 1000));
    assert_eq!(token2.balance(&user1_id), BigInt::from_u32(&e, 1000));
    assert_eq!(token_share.balance(&user1_id), BigInt::zero(&e));
    assert_eq!(token1.balance(&pool_id), BigInt::zero(&e));
    assert_eq!(token2.balance(&pool_id), BigInt::zero(&e));
    assert_eq!(token_share.balance(&pool_id), BigInt::zero(&e));
}
