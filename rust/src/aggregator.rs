use crate::{file_utils::stringify_fr, gkr::Proof};
use halo2curves::bn256::Fr;
use serde::{Deserialize, Serialize};

/// Circom-GKR
struct Meta(Vec<usize>);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct CircomInputProof {
    pub sumcheckProof: Vec<Vec<Vec<String>>>,
    pub sumcheckr: Vec<Vec<String>>,
    pub q: Vec<Vec<String>>,
    pub f: Vec<String>,
    pub D: Vec<Vec<String>>,
    pub z: Vec<Vec<String>>,
    pub r: Vec<String>,
    pub inputFunc: Vec<Vec<String>>,
    pub add: Vec<Vec<Vec<String>>>,
    pub mult: Vec<Vec<Vec<String>>>,
}

impl CircomInputProof {
    fn new_from_proof(proof: Proof<Fr>) -> Self {
        let sp: Vec<Vec<Vec<String>>> = proof
            .sumcheck_proofs
            .iter()
            .map(|p| p.iter().map(|f| stringify_fr_vector(f)).collect())
            .collect();

        let sr: Vec<Vec<String>> = proof.sumcheck_r.iter().map(|p| stringify_fr_vector(p)).collect();
        let q: Vec<Vec<String>> = proof.q.iter().map(|p| stringify_fr_vector(p)).collect();
        let f: Vec<String> = stringify_fr_vector(&proof.f);
        let d: Vec<Vec<String>> = proof.d.iter().map(|p| stringify_fr_vector(p)).collect();
        let z: Vec<Vec<String>> = proof.z.iter().map(|p| stringify_fr_vector(p)).collect();
        let r: Vec<String> = stringify_fr_vector(&proof.r);
        let input_func: Vec<Vec<String>> = proof.input_func.iter().map(|p| stringify_fr_vector(p)).collect();
        let add: Vec<Vec<Vec<String>>> = proof
            .add
            .iter()
            .map(|p| p.iter().map(|f| stringify_fr_vector(f)).collect())
            .collect();
        let mult: Vec<Vec<Vec<String>>> = proof
            .mult
            .iter()
            .map(|p| p.iter().map(|f| stringify_fr_vector(f)).collect())
            .collect();
        
        CircomInputProof { sumcheckProof: sp, sumcheckr: sr, q, f, D: d, z, r, inputFunc: input_func, add, mult }
    }
}

fn stringify_fr_vector(v: &Vec<Fr>) -> Vec<String> {
    v.iter().map(|f| stringify_fr(f)).collect()
}

fn zeros(l: usize) -> Vec<Fr> {
    vec![Fr::zero(); l]
}

fn get_meta(proof: &Proof<Fr>) -> Meta {
    let mut meta = vec![];
    // meta[0] = depth
    meta.push(proof.depth);

    // meta[1] = largest k
    let largest_k = proof
        .k
        .iter()
        .max()
        .cloned()
        .expect("Empty proof : k is None");
    meta.push(largest_k);

    // meta[2] = k_i(0)
    meta.push(proof.k[0]);

    // meta[3] = # of terms of D
    let n_terms_d = proof.d.len();
    meta.push(n_terms_d);

    // meta[4] = largest # of terms among sumcheck proofs (highest degree)
    let largest_deg = proof
        .sumcheck_proofs
        .iter()
        .map(|p| p.iter().map(|terms| terms.len()).max().unwrap())
        .max()
        .unwrap();
    meta.push(largest_deg);

    // meta[5] = largest # of terms among q
    let largest_terms_q = proof.q.iter().map(|p| p.len()).max().unwrap();
    meta.push(largest_terms_q);

    // meta[6] = # of terms in w_d
    let n_terms_input_func = proof.input_func.iter().map(|p| p.len()).max().unwrap();
    meta.push(n_terms_input_func);

    // meta[7] = k_i(d - 1)
    let k_input = proof.k[proof.depth - 1];
    meta.push(k_input);

    // meta[8], meta[9] = largest # of terms among {add_i, mult_i}
    let l_add = proof.add.iter().map(|p| p.len()).max().unwrap();
    let l_mult = proof.mult.iter().map(|p| p.len()).max().unwrap();
    meta.push(l_add);
    meta.push(l_mult);

    meta.append(&mut proof.k.clone());

    Meta(meta)
}

fn modify_proof_for_circom(proof: Proof<Fr>, meta_value: Meta) -> Proof<Fr> {
    let meta = meta_value.0;
    let mut sumcheck_proofs = vec![];
    for p in proof.sumcheck_proofs.iter() {
        let mut new_p = vec![];
        for terms in p.iter() {
            let mut new_terms = terms.clone();
            if terms.len() < meta[4] {
                let mut z = zeros(meta[4] - terms.len());
                z.append(&mut new_terms);
                new_p.push(z);
            } else {
                new_p.push(new_terms);
            }
        }
        if p.len() < 2 * meta[1] {
            for _ in 0..(2 * meta[1] - p.len()) {
                let mut new_terms = zeros(meta[4]);
                new_p.push(new_terms);
            }
        }
        sumcheck_proofs.push(new_p);
    }

    let mut sumcheck_r = vec![];
    for p in proof.sumcheck_r.iter() {
        let mut new_p = p.clone();
        if p.len() < 2 * meta[1] {
            new_p.extend(zeros(2 * meta[1] - p.len()));
        }
        sumcheck_r.push(new_p);
    }

    let mut q = vec![];
    for p in proof.q.iter() {
        let mut new_p = p.clone();
        if p.len() < meta[5] {
            let mut z = zeros(meta[5] - p.len());
            z.append(&mut new_p);
            q.push(z);
        } else {
            q.push(new_p);
        }
    }

    let mut z = vec![];
    for p in proof.z.iter() {
        let mut new_p = p.clone();
        if p.len() < meta[1] {
            new_p.extend(zeros(meta[1] - p.len()));
        }
        z.push(new_p);
    }

    let mut add = vec![];
    for p in proof.add.iter() {
        let mut new_p = vec![];
        for terms in p.iter() {
            let mut new_terms = terms.clone();
            if terms.len() < 3 * meta[1] + 1 {
                let mut z = zeros(3 * meta[1] + 1 - terms.len());
                new_terms.append(&mut z);
            }
            new_p.push(new_terms);
        }
        if p.len() < meta[8] {
            for _ in 0..(meta[8] - p.len()) {
                let mut new_terms = zeros(3 * meta[1] + 1);
                new_p.push(new_terms);
            }
        }
        add.push(new_p);
    }

    let mut mult = vec![];
    for p in proof.mult.iter() {
        let mut new_p = vec![];
        for terms in p.iter() {
            let mut new_terms = terms.clone();
            if terms.len() < 3 * meta[1] + 1 {
                let mut z = zeros(3 * meta[1] + 1 - terms.len());
                new_terms.append(&mut z);
            }
            new_p.push(new_terms);
        }
        if p.len() < meta[9] {
            for _ in 0..(meta[9] - p.len()) {
                let mut new_terms = zeros(3 * meta[1] + 1);
                new_p.push(new_terms);
            }
        }
        mult.push(new_p);
    }
    Proof {
        sumcheck_proofs,
        sumcheck_r,
        q,
        z,
        f: proof.f,
        d: proof.d,
        r: proof.r,
        add,
        mult,
        depth: proof.depth,
        input_func: proof.input_func,
        k: proof.k,
    }
}

pub fn prove_recursively_circom() -> () {}
