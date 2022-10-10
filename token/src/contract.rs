use crate::admin::{check_admin, has_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::balance::{read_state, write_state};
use crate::metadata::{
    read_decimal, read_name, read_symbol, write_decimal, write_name, write_symbol,
};
use crate::storage_types::DataKey;
use soroban_auth::verify;
use soroban_auth::{Identifier, Signature};
use soroban_sdk::{contractimpl, symbol, BigInt, Bytes, Env};

pub trait TokenTrait {
    fn initialize(e: Env, admin: Identifier, decimal: u32, name: Bytes, symbol: Bytes);

    fn nonce(e: Env, id: Identifier) -> BigInt;

    fn allowance(e: Env, from: Identifier, spender: Identifier) -> BigInt;

    fn approve(e: Env, from: Signature, nonce: BigInt, spender: Identifier, amount: BigInt);

    fn balance(e: Env, id: Identifier) -> BigInt;

    fn is_frozen(e: Env, id: Identifier) -> bool;

    fn xfer(e: Env, from: Signature, nonce: BigInt, to: Identifier, amount: BigInt);

    fn xfer_from(
        e: Env,
        spender: Signature,
        nonce: BigInt,
        from: Identifier,
        to: Identifier,
        amount: BigInt,
    );

    fn burn(e: Env, admin: Signature, nonce: BigInt, from: Identifier, amount: BigInt);

    fn freeze(e: Env, admin: Signature, nonce: BigInt, id: Identifier);

    fn mint(e: Env, admin: Signature, nonce: BigInt, to: Identifier, amount: BigInt);

    fn set_admin(e: Env, admin: Signature, nonce: BigInt, new_admin: Identifier);

    fn unfreeze(e: Env, admin: Signature, nonce: BigInt, id: Identifier);

    fn decimals(e: Env) -> u32;

    fn name(e: Env) -> Bytes;

    fn symbol(e: Env) -> Bytes;
}

fn read_nonce(e: &Env, id: &Identifier) -> BigInt {
    let key = DataKey::Nonce(id.clone());
    e.data()
        .get(key)
        .unwrap_or_else(|| Ok(BigInt::zero(e)))
        .unwrap()
}

fn verify_and_consume_nonce(e: &Env, auth: &Signature, expected_nonce: &BigInt) {
    match auth {
        Signature::Invoker => {
            if BigInt::zero(&e) != expected_nonce {
                panic!("nonce should be zero for Invoker")
            }
            return;
        }
        _ => {}
    }

    let id = auth.identifier(&e);
    let key = DataKey::Nonce(id.clone());
    let nonce = read_nonce(e, &id);

    if nonce != expected_nonce {
        panic!("incorrect nonce")
    }
    e.data().set(key, &nonce + 1);
}

pub struct Token;

#[contractimpl]
impl TokenTrait for Token {
    fn initialize(e: Env, admin: Identifier, decimal: u32, name: Bytes, symbol: Bytes) {
        if has_administrator(&e) {
            panic!("already initialized")
        }
        write_administrator(&e, admin);

        write_decimal(&e, u8::try_from(decimal).expect("Decimal must fit in a u8"));
        write_name(&e, name);
        write_symbol(&e, symbol);
    }

    fn nonce(e: Env, id: Identifier) -> BigInt {
        read_nonce(&e, &id)
    }

    fn allowance(e: Env, from: Identifier, spender: Identifier) -> BigInt {
        read_allowance(&e, from, spender)
    }

    fn approve(e: Env, from: Signature, nonce: BigInt, spender: Identifier, amount: BigInt) {
        verify_and_consume_nonce(&e, &from, &nonce);

        let from_id = from.identifier(&e);

        verify(
            &e,
            &from,
            symbol!("approve"),
            (&from_id, nonce, &spender, &amount),
        );
        write_allowance(&e, from_id, spender, amount);
    }

    fn balance(e: Env, id: Identifier) -> BigInt {
        read_balance(&e, id)
    }

    fn is_frozen(e: Env, id: Identifier) -> bool {
        read_state(&e, id)
    }

    fn xfer(e: Env, from: Signature, nonce: BigInt, to: Identifier, amount: BigInt) {
        verify_and_consume_nonce(&e, &from, &nonce);

        let from_id = from.identifier(&e);

        verify(&e, &from, symbol!("xfer"), (&from_id, nonce, &to, &amount));
        spend_balance(&e, from_id, amount.clone());
        receive_balance(&e, to, amount);
    }

    fn xfer_from(
        e: Env,
        spender: Signature,
        nonce: BigInt,
        from: Identifier,
        to: Identifier,
        amount: BigInt,
    ) {
        verify_and_consume_nonce(&e, &spender, &nonce);

        let spender_id = spender.identifier(&e);

        verify(
            &e,
            &spender,
            symbol!("xfer_from"),
            (&spender_id, nonce, &from, &to, &amount),
        );
        spend_allowance(&e, from.clone(), spender_id, amount.clone());
        spend_balance(&e, from, amount.clone());
        receive_balance(&e, to, amount);
    }

    fn burn(e: Env, admin: Signature, nonce: BigInt, from: Identifier, amount: BigInt) {
        check_admin(&e, &admin);
        verify_and_consume_nonce(&e, &admin, &nonce);

        let admin_id = admin.identifier(&e);

        verify(
            &e,
            &admin,
            symbol!("burn"),
            (admin_id, nonce, &from, &amount),
        );
        spend_balance(&e, from, amount);
    }

    fn freeze(e: Env, admin: Signature, nonce: BigInt, id: Identifier) {
        check_admin(&e, &admin);

        verify_and_consume_nonce(&e, &admin, &nonce);

        let admin_id = admin.identifier(&e);

        verify(&e, &admin, symbol!("freeze"), (admin_id, nonce, &id));
        write_state(&e, id, true);
    }

    fn mint(e: Env, admin: Signature, nonce: BigInt, to: Identifier, amount: BigInt) {
        check_admin(&e, &admin);

        verify_and_consume_nonce(&e, &admin, &nonce);

        let admin_id = admin.identifier(&e);

        verify(&e, &admin, symbol!("mint"), (admin_id, nonce, &to, &amount));
        receive_balance(&e, to, amount);
    }

    fn set_admin(e: Env, admin: Signature, nonce: BigInt, new_admin: Identifier) {
        check_admin(&e, &admin);

        verify_and_consume_nonce(&e, &admin, &nonce);

        let admin_id = admin.identifier(&e);

        verify(
            &e,
            &admin,
            symbol!("set_admin"),
            (admin_id, nonce, &new_admin),
        );
        write_administrator(&e, new_admin);
    }

    fn unfreeze(e: Env, admin: Signature, nonce: BigInt, id: Identifier) {
        check_admin(&e, &admin);

        verify_and_consume_nonce(&e, &admin, &nonce);

        let admin_id = admin.identifier(&e);

        verify(&e, &admin, symbol!("unfreeze"), (admin_id, nonce, &id));
        write_state(&e, id, false);
    }

    fn decimals(e: Env) -> u32 {
        read_decimal(&e)
    }

    fn name(e: Env) -> Bytes {
        read_name(&e)
    }

    fn symbol(e: Env) -> Bytes {
        read_symbol(&e)
    }
}
