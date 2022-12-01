use std::vec;

use ff::PrimeField;
use mimc_rs::{Fr, FrRepr, Mimc7};

use super::poly::*;

pub fn convert_s_to_fr<S>(v: &S) -> mimc_rs::Fr
where
    S: PrimeField<Repr = [u8; 32]>,
{
    let v_bytes = v.to_repr();
    let res = FrRepr(v_bytes);
    mimc_rs::Fr::from_repr(res).unwrap()
}

pub fn convert_fr_to_s<S: PrimeField<Repr = [u8; 32]>>(v: mimc_rs::Fr) -> S {
    let FrRepr(v_bytes) = v.to_repr();
    S::from_repr(v_bytes).unwrap()
}

pub fn prove_sumcheck<S: PrimeField<Repr = [u8; 32]>>(
    g: &Vec<Vec<S>>,
    v: usize,
) -> (Vec<Vec<S>>, Vec<S>) {
    let mimc = Mimc7::new(91);
    let mut proof = vec![];
    let mut r = vec![];

    let mut g_1 = get_empty(v);
    let assignments: Vec<Vec<S>> = generate_binary(v - 1);
    for assignment in assignments {
        let mut g_1_sub = g.clone();
        for (i, x_i) in assignment.into_iter().enumerate() {
            let idx = i + 1;
            g_1_sub = partial_eval_i(g_1_sub, &x_i, idx);
        }
        g_1 = add_poly(&g_1, &g_1_sub);
    }
    let g_1_coeffs = get_univariate_coeff(g_1, 1);
    proof.push(g_1_coeffs.clone());

    let mimc_g1_coeffs = g_1_coeffs.iter().map(|s| convert_s_to_fr(s)).collect();
    let r_1 = mimc.multi_hash(mimc_g1_coeffs, &Fr::from(0));
    r.push(convert_fr_to_s(r_1));

    for j in 1..v - 1 {
        let mut g_j: Vec<Vec<S>> = get_empty(v);
        let assignments: Vec<Vec<S>> = generate_binary(v - j - 1);

        for (i, r_i) in r.iter().enumerate() {
            g_j = partial_eval_i(g_j, r_i, i + 1);
        }
        let mut res_g_j = get_empty(v);
        for assignment in assignments {
            let mut g_j_sub = g_j.clone();
            for (i, x_i) in assignment.into_iter().enumerate() {
                let idx = j + i + 2;
                g_j_sub = partial_eval_i(g_j_sub, &x_i, idx);
            }
            res_g_j = add_poly(&res_g_j, &g_j_sub);
        }
        let g_j_coeffs = get_univariate_coeff(res_g_j, j + 1);
        proof.push(g_j_coeffs.clone());

        let mimc_gj_coeffs = g_j_coeffs.iter().map(|s| convert_s_to_fr(s)).collect();
        let r_n = mimc.multi_hash(mimc_gj_coeffs, &Fr::from(0));
        r.push(convert_fr_to_s(r_n));
    }

    let g_v = partial_eval(g.clone(), &r);
    let g_v_coeffs = get_univariate_coeff(g_v, v);
    proof.push(g_v_coeffs.clone());

    let mimc_gv_coeffs = g_v_coeffs.iter().map(|s| convert_s_to_fr(s)).collect();
    let r_v = mimc.multi_hash(mimc_gv_coeffs, &Fr::from(0));
    r.push(convert_fr_to_s(r_v));

    (proof, r)
}
