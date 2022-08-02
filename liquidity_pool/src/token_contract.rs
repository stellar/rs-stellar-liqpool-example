use soroban_sdk::{Binary, Env, FixedBinary};
use soroban_token_contract::public_types::U256;

#[cfg(any(not(feature = "testutils"), feature = "token-wasm"))]
pub const TOKEN_CONTRACT: &[u8] = include_bytes!("../../soroban_token_contract.wasm");

#[cfg(any(not(feature = "testutils"), feature = "token-wasm"))]
pub fn create_contract(e: &Env, token_a: &U256, token_b: &U256) -> FixedBinary<32> {
    let bin = Binary::from_slice(e, TOKEN_CONTRACT);
    let mut salt = Binary::new(&e);
    salt.append(&token_a.clone().into());
    salt.append(&token_b.clone().into());
    let salt = e.compute_hash_sha256(salt);
    e.create_contract_from_contract(bin, salt)
}

#[cfg(all(feature = "testutils", not(feature = "token-wasm")))]
pub fn create_contract(e: &Env, token_a: &U256, token_b: &U256) -> FixedBinary<32> {
    use sha2::{Digest, Sha256};
    use soroban_sdk::IntoVal;
    use std::vec::Vec;
    use stellar_xdr::{Hash, HashIdPreimage, HashIdPreimageContractId, Uint256, WriteXdr};

    let salt = {
        let mut salt_bin = Binary::new(&e);
        salt_bin.append(&token_a.clone().into());
        salt_bin.append(&token_b.clone().into());
        Uint256(e.compute_hash_sha256(salt_bin).into())
    };

    let contract_id = Hash(e.get_current_contract().into());

    let new_contract_id = {
        let pre_image =
            HashIdPreimage::ContractIdFromContract(HashIdPreimageContractId { contract_id, salt });
        let mut buf = Vec::new();
        pre_image.write_xdr(&mut buf).unwrap();
        Sha256::digest(buf).into_val(e)
    };

    soroban_token_contract::testutils::register_test_contract(e, &new_contract_id);
    FixedBinary::from_array(e, new_contract_id)
}
