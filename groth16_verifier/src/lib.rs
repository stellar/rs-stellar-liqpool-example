#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    crypto::bls12_381::{Fr, G1Affine, G2Affine},
    vec, BytesN, Env, Vec,
};

#[derive(Clone)]
#[contracttype]
pub struct VerificationKey {
    pub alpha: BytesN<96>,
    pub beta: BytesN<192>,
    pub gamma: BytesN<192>,
    pub delta: BytesN<192>,
    pub ic: Vec<BytesN<96>>,
}

#[derive(Clone)]
#[contracttype]
pub struct Proof {
    pub a: BytesN<96>,
    pub neg_a: BytesN<96>,
    pub b: BytesN<192>,
    pub c: BytesN<96>,
}

#[contract]
pub struct Groth16Verifier;

#[contractimpl]
impl Groth16Verifier {
    pub fn verify_proof(env: Env, vk: VerificationKey, proof: Proof, pub_signals: Vec<Fr>) -> bool {
        let bls = env.crypto().bls12_381();

        // vk_x = IC0 + pub_signals[0]*IC1
        // Here we have only one public input. If there were more, we'd sum them similarly.
        let mut vk_x = G1Affine::from_bytes(vk.ic.get(0).unwrap());
        if let Some(s) = pub_signals.get(0) {
            let term = bls.g1_mul(&G1Affine::from_bytes(vk.ic.get(1).unwrap()), &s);
            vk_x = bls.g1_add(&vk_x, &term);
        }

        // We need to compute the pairing:
        // e(-A, B) * e(alpha, beta) * e(vk_x, gamma) * e(C, delta) == 1
        let neg_a = G1Affine::from_bytes(proof.neg_a);
        let vp1 = vec![
            &env,
            neg_a,
            G1Affine::from_bytes(vk.alpha),
            vk_x,
            G1Affine::from_bytes(proof.c),
        ];
        let vp2 = vec![
            &env,
            G2Affine::from_bytes(proof.b),
            G2Affine::from_bytes(vk.beta),
            G2Affine::from_bytes(vk.gamma),
            G2Affine::from_bytes(vk.delta),
        ];

        bls.pairing_check(vp1, vp2)
    }
}

mod test;
