use soroban_sdk::{serde::Serialize, Account, BigInt, BytesN, Env, EnvVal, Symbol, TryIntoVal};

use crate::public_types::{
    Identifier, KeyedAccountAuthorization, KeyedAuthorization, KeyedEd25519Signature, Message,
    MessageV0,
};

pub trait NonceAuth {
    fn read_nonce(e: &Env, id: Identifier) -> BigInt;
    fn read_and_increment_nonce(&self, e: &Env, id: Identifier) -> BigInt;
    fn get_keyed_auth(&self) -> &KeyedAuthorization;
}

fn check_ed25519_auth(e: &Env, auth: &KeyedEd25519Signature, function: Symbol, parameters: EnvVal) {
    let msg = MessageV0 {
        function,
        contrct_id: e.get_current_contract(),
        network_id: e.ledger().network_passphrase(),
        parameters: parameters.to_raw().try_into_val(e).unwrap(),
    };
    let msg_bin = Message::V0(msg).serialize(e);

    e.verify_sig_ed25519(
        auth.public_key.clone().into(),
        msg_bin,
        auth.signature.clone().into(),
    );
}

fn check_account_auth(
    e: &Env,
    auth: &KeyedAccountAuthorization,
    function: Symbol,
    parameters: EnvVal,
) {
    let acc = Account::from_public_key(&auth.public_key).unwrap();

    let msg = MessageV0 {
        function,
        contrct_id: e.get_current_contract(),
        network_id: e.ledger().network_passphrase(),
        parameters: parameters.to_raw().try_into_val(e).unwrap(),
    };
    let msg_bin = Message::V0(msg).serialize(e);

    let threshold = acc.medium_threshold();
    let mut weight = 0u32;

    let sigs = &auth.signatures;
    let mut prev_pk: Option<BytesN<32>> = None;
    for sig in sigs.iter().map(Result::unwrap) {
        // Cannot take multiple signatures from the same key
        if let Some(prev) = prev_pk {
            if prev >= sig.public_key {
                panic!("signature out of order")
            }
        }

        e.verify_sig_ed25519(
            sig.public_key.clone().into(),
            msg_bin.clone(),
            sig.signature.into(),
        );
        // TODO: Check for overflow
        weight += acc.signer_weight(&sig.public_key);

        prev_pk = Some(sig.public_key);
    }

    if weight < threshold {
        panic!("insufficient signing weight")
    }
}

// Note that nonce is not used by KeyedAuthorization::Contract
pub fn check_auth<T>(e: &Env, auth: &T, nonce: BigInt, function: Symbol, parameters: EnvVal)
where
    T: NonceAuth,
{
    match auth.get_keyed_auth() {
        KeyedAuthorization::Contract => {
            if nonce != BigInt::from_i32(e, 0) {
                panic!("nonce should be zero for Contract")
            }
            e.get_invoking_contract();
        }
        KeyedAuthorization::Ed25519(kea) => {
            let stored_nonce =
                auth.read_and_increment_nonce(e, Identifier::Ed25519(kea.public_key.clone()));
            if nonce != stored_nonce {
                panic!("incorrect nonce")
            }
            check_ed25519_auth(e, &kea, function, parameters)
        }
        KeyedAuthorization::Account(kaa) => {
            let stored_nonce =
                auth.read_and_increment_nonce(e, Identifier::Account(kaa.public_key.clone()));
            if nonce != stored_nonce {
                panic!("incorrect nonce")
            }
            check_account_auth(e, &kaa, function, parameters)
        }
    }
}
