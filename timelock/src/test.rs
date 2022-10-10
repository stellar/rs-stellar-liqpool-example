#![cfg(test)]

use super::*;
use rand::{thread_rng, RngCore};
use soroban_sdk::testutils::{Accounts, Ledger, LedgerInfo};
use soroban_sdk::{vec, AccountId, Env, IntoVal};
use token::{Client as TokenClient, TokenMetadata};

fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    id
}

fn create_token_contract(e: &Env, admin: &AccountId) -> (BytesN<32>, TokenClient) {
    let id = e.register_contract_token(None);
    let token = TokenClient::new(e, &id);
    // decimals, name, symbol don't matter in tests
    token.init(
        &Identifier::Account(admin.clone()),
        &TokenMetadata {
            name: "name".into_val(e),
            symbol: "symbol".into_val(e),
            decimals: 7,
        },
    );
    (id, token)
}

fn create_claimable_balance_contract(e: &Env) -> ClaimableBalanceContractClient {
    let contract_id = BytesN::from_array(e, &generate_contract_id());
    e.register_contract(&contract_id, ClaimableBalanceContract {});

    ClaimableBalanceContractClient::new(e, contract_id)
}

struct ClaimableBalanceTest {
    env: Env,
    deposit_user: AccountId,
    claim_users: [AccountId; 3],
    token: TokenClient,
    token_id: BytesN<32>,
    contract: ClaimableBalanceContractClient,
    contract_id: Identifier,
}

impl ClaimableBalanceTest {
    fn setup() -> Self {
        let env: Env = Default::default();
        env.ledger().set(LedgerInfo {
            timestamp: 12345,
            protocol_version: 1,
            sequence_number: 10,
            network_passphrase: Default::default(),
            base_reserve: 10,
        });

        let deposit_user = env.accounts().generate();
        let deposit_user_id = Identifier::Account(deposit_user.clone());

        let claim_users = [
            env.accounts().generate(),
            env.accounts().generate(),
            env.accounts().generate(),
        ];

        let token_admin = env.accounts().generate();

        let (token_id, token) = create_token_contract(&env, &token_admin);
        token.with_source_account(&token_admin).mint(
            &Signature::Invoker,
            &BigInt::zero(&env),
            &deposit_user_id,
            &BigInt::from_u32(&env, 1000),
        );

        let contract = create_claimable_balance_contract(&env);
        let contract_id = Identifier::Contract(contract.contract_id.clone());
        ClaimableBalanceTest {
            env,
            deposit_user,
            claim_users,
            token,
            token_id,
            contract,
            contract_id: contract_id,
        }
    }

    fn approve_deposit(&self, amount: u32) {
        self.token.with_source_account(&self.deposit_user).approve(
            &Signature::Invoker,
            &BigInt::zero(&self.env),
            &Identifier::Contract(self.contract.contract_id.clone()),
            &BigInt::from_u32(&self.env, amount),
        );
    }

    fn deposit(&self, amount: u32, claimants: &Vec<Identifier>, time_bound: TimeBound) {
        self.call_deposit(
            &self.deposit_user,
            &self.token_id,
            &BigInt::from_u32(&self.env, amount),
            claimants,
            &time_bound,
        );
    }

    fn claim(&self, claim_user: &AccountId) {
        self.call_claim(claim_user);
    }

    fn account_id_to_identifier(&self, account_id: &AccountId) -> Identifier {
        Identifier::Account(account_id.clone())
    }

    fn call_deposit(
        &self,
        account_id: &AccountId,
        token: &BytesN<32>,
        amount: &BigInt,
        claimants: &Vec<Identifier>,
        time_bound: &TimeBound,
    ) {
        self.contract
            .with_source_account(account_id)
            .deposit(token, amount, claimants, time_bound);
    }

    fn call_claim(&self, account_id: &AccountId) {
        self.contract.with_source_account(account_id).claim();
    }
}

#[test]
fn test_deposit_and_claim() {
    let test = ClaimableBalanceTest::setup();
    test.approve_deposit(800);
    test.deposit(
        800,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
            test.account_id_to_identifier(&test.claim_users[1]),
        ],
        TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );

    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.deposit_user)),
        BigInt::from_u32(&test.env, 200)
    );
    assert_eq!(
        test.token.balance(&test.contract_id),
        BigInt::from_u32(&test.env, 800)
    );
    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.claim_users[1])),
        BigInt::from_u32(&test.env, 0)
    );

    test.claim(&test.claim_users[1]);
    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.deposit_user)),
        BigInt::from_u32(&test.env, 200)
    );
    assert_eq!(
        test.token.balance(&test.contract_id),
        BigInt::from_u32(&test.env, 0)
    );
    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.claim_users[1])),
        BigInt::from_u32(&test.env, 800)
    );
}

#[test]
#[should_panic]
fn test_deposit_above_allowance_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.approve_deposit(800);
    test.deposit(
        801,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
        ],
        TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );
}

#[test]
#[should_panic(expected = "contract has been already initialized")]
fn test_double_deposit_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.approve_deposit(800);
    test.deposit(
        1,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
        ],
        TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );
    test.deposit(
        1,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
        ],
        TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );
}

#[test]
#[should_panic(expected = "claimant is not allowed to claim this balance")]
fn test_unauthorized_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.approve_deposit(800);
    test.deposit(
        800,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
            test.account_id_to_identifier(&test.claim_users[1]),
        ],
        TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );

    test.claim(&test.claim_users[2]);
}

#[test]
#[should_panic(expected = "time predicate is not fulfilled")]
fn test_out_of_time_bound_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.approve_deposit(800);
    test.deposit(
        800,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
        ],
        TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 12346,
        },
    );

    test.claim(&test.claim_users[0]);
}

#[test]
#[should_panic(expected = "HostStorageError")]
fn test_double_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.approve_deposit(800);
    test.deposit(
        800,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
        ],
        TimeBound {
            kind: TimeBoundKind::Before,
            timestamp: 12346,
        },
    );

    test.claim(&test.claim_users[0]);
    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.claim_users[0])),
        BigInt::from_u32(&test.env, 800)
    );
    test.claim(&test.claim_users[0]);
}

#[test]
#[should_panic(expected = "contract has been already initialized")]
fn test_deposit_after_claim_not_possible() {
    let test = ClaimableBalanceTest::setup();
    test.approve_deposit(1000);
    test.deposit(
        800,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
        ],
        TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 12344,
        },
    );

    test.claim(&test.claim_users[0]);
    assert_eq!(
        test.token
            .balance(&test.account_id_to_identifier(&test.claim_users[0])),
        BigInt::from_u32(&test.env, 800)
    );
    test.deposit(
        200,
        &vec![
            &test.env,
            test.account_id_to_identifier(&test.claim_users[0]),
        ],
        TimeBound {
            kind: TimeBoundKind::After,
            timestamp: 12344,
        },
    );
}
