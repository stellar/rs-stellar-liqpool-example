#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{token, vec, Address, Env, IntoVal, Symbol};
use token::Client as TokenClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    TokenClient::new(e, &e.register_stellar_asset_contract(admin.clone()))
}

fn create_claimable_balance_contract<'a>(e: &Env) -> ClaimableBalanceContractClient<'a> {
    ClaimableBalanceContractClient::new(e, &e.register_contract(None, ClaimableBalanceContract {}))
}

struct ClaimableBalanceTest<'a> {
    env: Env,
    deposit_address: Address,
    claim_addresses: [Address; 3],
    token: TokenClient<'a>,
    contract: ClaimableBalanceContractClient<'a>,
}

impl<'a> ClaimableBalanceTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        env.ledger().set(LedgerInfo {
            timestamp: 12345,
            protocol_version: 1,
            sequence_number: 10,
            network_id: Default::default(),
            base_reserve: 10,
        });

        let deposit_address = Address::random(&env);

        let claim_addresses = [
            Address::random(&env),
            Address::random(&env),
            Address::random(&env),
        ];

        let token_admin = Address::random(&env);

        let token = create_token_contract(&env, &token_admin);
        token.mint(&deposit_address, &1000);

        let contract = create_claimable_balance_contract(&env);
        ClaimableBalanceTest {
            env,
            deposit_address,
            claim_addresses,
            token,
            contract,
        }
    }
}

#[test]
fn test_deposit_and_claim() {
    let test = ClaimableBalanceTest::setup();
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &800,
        &vec![
            &test.env,
            test.claim_addresses[0].clone(),
            test.claim_addresses[1].clone(),
        ],
        &TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );

    assert_eq!(
        test.env.auths(),
        [
            (
                test.deposit_address.clone(),
                test.contract.address.clone(),
                Symbol::short("deposit"),
                (
                    test.deposit_address.clone(),
                    test.token.address.clone(),
                    800_i128,
                    vec![
                        &test.env,
                        test.claim_addresses[0].clone(),
                        test.claim_addresses[1].clone()
                    ],
                    TimeBound {
                        kind: TimeBoundKind::Before,
                        timestamp: 12346,
                    },
                )
                    .into_val(&test.env),
            ),
            (
                test.deposit_address.clone(),
                test.token.address.clone(),
                Symbol::short("transfer"),
                (
                    test.deposit_address.clone(),
                    &test.contract.address,
                    800_i128,
                )
                    .into_val(&test.env),
            ),
        ]
    );

    assert_eq!(test.token.balance(&test.deposit_address), 200);
    assert_eq!(test.token.balance(&test.contract.address), 800);
    assert_eq!(test.token.balance(&test.claim_addresses[1]), 0);

    test.contract.claim(&test.claim_addresses[1]);
    assert_eq!(
        test.env.auths(),
        [(
            test.claim_addresses[1].clone(),
            test.contract.address.clone(),
            Symbol::short("claim"),
            (test.claim_addresses[1].clone(),).into_val(&test.env),
        )]
    );

    assert_eq!(test.token.balance(&test.deposit_address), 200);
    assert_eq!(test.token.balance(&test.contract.address), 0);
    assert_eq!(test.token.balance(&test.claim_addresses[1]), 800);
}

#[test]
#[should_panic(expected = "contract has been already initialized")]
fn test_double_deposit_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &1,
        &vec![&test.env, test.claim_addresses[0].clone()],
        &TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &1,
        &vec![&test.env, test.claim_addresses[0].clone()],
        &TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );
}

#[test]
#[should_panic(expected = "claimant is not allowed to claim this balance")]
fn test_unauthorized_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &800,
        &vec![
            &test.env,
            test.claim_addresses[0].clone(),
            test.claim_addresses[1].clone(),
        ],
        &TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );

    test.contract.claim(&test.claim_addresses[2]);
}

#[test]
#[should_panic(expected = "time predicate is not fulfilled")]
fn test_out_of_time_bound_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &800,
        &vec![&test.env, test.claim_addresses[0].clone()],
        &TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 12346,
        },
    );

    test.contract.claim(&test.claim_addresses[0]);
}

#[test]
#[should_panic(expected = "HostStorageError")]
fn test_double_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &800,
        &vec![&test.env, test.claim_addresses[0].clone()],
        &TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );

    test.contract.claim(&test.claim_addresses[0]);
    assert_eq!(test.token.balance(&test.claim_addresses[0]), 800);
    test.contract.claim(&test.claim_addresses[0]);
}

#[test]
#[should_panic(expected = "contract has been already initialized")]
fn test_deposit_after_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &800,
        &vec![&test.env, test.claim_addresses[0].clone()],
        &TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 12344,
        },
    );

    test.contract.claim(&test.claim_addresses[0]);
    assert_eq!(test.token.balance(&test.claim_addresses[0]), 800);
    test.contract.deposit(
        &test.deposit_address,
        &test.token.address,
        &200,
        &vec![&test.env, test.claim_addresses[0].clone()],
        &TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 12344,
        },
    );
}
