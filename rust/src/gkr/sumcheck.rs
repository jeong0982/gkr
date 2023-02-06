use std::vec;

use ff::PrimeField;
use itertools::Itertools;
use mimc_rs::{Fr, FrRepr, Mimc7};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

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

fn n_trailing_bits<S: PrimeField<Repr = [u8; 32]> + std::hash::Hash>(
    wire: &Vec<Vec<S>>,
    n: usize,
) -> Vec<Vec<S>> {
    let mut res: Vec<Vec<S>> = wire
        .iter()
        .map(|inner_vec| inner_vec.iter().rev().take(n).rev().cloned().collect())
        .collect();
    res.into_iter().unique().collect()
}

// only can be run for f: add_i(f1 + f2) + mult_i(f1 * f2)
pub fn prove_sumcheck_opt<S: PrimeField<Repr = [u8; 32]> + std::hash::Hash>(
    add_wire: &Vec<Vec<S>>,
    mult_wire: &Vec<Vec<S>>,
    add_i: &Vec<Vec<S>>,
    mult_i: &Vec<Vec<S>>,
    f1: &Vec<Vec<S>>,
    f2: &Vec<Vec<S>>,
    v: usize,
) -> (Vec<Vec<S>>, Vec<S>) {
    let mimc = Mimc7::new(91);
    let mut proof = vec![];
    let mut r = vec![];

    let add_assignments: Vec<Vec<S>> = n_trailing_bits(add_wire, v - 1);
    let g_1_add = add_assignments
        .par_iter()
        .map(|assignment| {
            let mut add_1_sub = add_i.clone();
            let f2_1_sub = partial_eval_from(f2.clone(), assignment, 2);
            let f1_1_sub = partial_eval_from(f1.clone(), assignment, 2);
            let add_1_sub = partial_eval_from_binary_form(&add_i.clone(), assignment, 2);

            let f1_1_coeffs = get_univariate_coeff(&f1_1_sub, 1, false);
            let f2_1_coeffs = get_univariate_coeff(&f2_1_sub, 1, false);
            let add_1_coeffs = get_univariate_coeff(&add_1_sub, 1, true);
            let f1_f2_add = add_univariate(&f1_1_coeffs, &f2_1_coeffs);
            mult_univariate(&f1_f2_add, &add_1_coeffs)
        })
        .reduce(|| vec![], |a, b| add_univariate(&a, &b));
    let mult_assignments: Vec<Vec<S>> = n_trailing_bits(mult_wire, v - 1);
    let g_1_mult = mult_assignments
        .par_iter()
        .map(|assignment| {
            let f2_1_sub = partial_eval_from(f2.clone(), assignment, 2);
            let f1_1_sub = partial_eval_from(f1.clone(), assignment, 2);
            let mult_1_sub = partial_eval_from_binary_form(&mult_i.clone(), assignment, 2);

            let f1_1_coeffs = get_univariate_coeff(&f1_1_sub, 1, false);
            let f2_1_coeffs = get_univariate_coeff(&f2_1_sub, 1, false);
            let mult_1_coeffs = get_univariate_coeff(&mult_1_sub, 1, true);
            let f1_f2_mult = mult_univariate(&f1_1_coeffs, &f2_1_coeffs);
            mult_univariate(&f1_f2_mult, &mult_1_coeffs)
        })
        .reduce(|| vec![], |a, b| add_univariate(&a, &b));

    let g_1 = add_univariate(&g_1_add, &g_1_mult);
    proof.push(g_1.clone());
    
    let mimc_g1_coeffs = g_1.iter().map(|s| convert_s_to_fr(s)).collect();
    let r_1 = mimc.multi_hash(mimc_g1_coeffs, &Fr::from(0));
    r.push(convert_fr_to_s(r_1));
    let mut f1_j = f1.clone();
    let mut f2_j = f2.clone();
    let mut add_j = add_i.clone();
    let mut mult_j = mult_i.clone();
    for j in 1..v - 1 {
        f1_j = partial_eval_i(&f1_j, &r[r.len() - 1], r.len());
        f2_j = partial_eval_i(&f2_j, &r[r.len() - 1], r.len());
        add_j = partial_eval_i_binary_form(&add_j, &r[r.len() - 1], r.len());
        mult_j = partial_eval_i_binary_form(&mult_j, &r[r.len() - 1], r.len());
        let add_assignments: Vec<Vec<S>> = n_trailing_bits(add_wire, v - j - 1);
        let mult_assignments: Vec<Vec<S>> = n_trailing_bits(mult_wire, v - j - 1);
        let g_j_add = add_assignments
        .par_iter()
        .map(|assignment| {
            let f1_j_sub = partial_eval_from(f1_j.clone(), assignment, j + 2);
            let f2_j_sub = partial_eval_from(f2_j.clone(), assignment, j + 2);
            let add_j_sub = partial_eval_from_binary_form(&add_j.clone(), assignment, j + 2);

            let f1_j_coeffs = get_univariate_coeff(&f1_j_sub, j + 1, false);
            let f2_j_coeffs = get_univariate_coeff(&f2_j_sub, j + 1, false);
            let add_j_coeffs = get_univariate_coeff(&add_j_sub, j + 1, true);
            let f1_f2_add = add_univariate(&f1_j_coeffs, &f2_j_coeffs);
            mult_univariate(&f1_f2_add, &add_j_coeffs)
        })
        .reduce(|| vec![], |a, b| add_univariate(&a, &b));
        let g_j_mult = mult_assignments
        .par_iter()
        .map(|assignment| {
            let f1_j_sub = partial_eval_from(f1_j.clone(), assignment, j + 2);
            let f2_j_sub = partial_eval_from(f2_j.clone(), assignment, j + 2);
            let mult_j_sub = partial_eval_from_binary_form(&mult_j.clone(), assignment, j + 2);

            let f1_j_coeffs = get_univariate_coeff(&f1_j_sub, j + 1, false);
            let f2_j_coeffs = get_univariate_coeff(&f2_j_sub, j + 1, false);
            let mult_j_coeffs = get_univariate_coeff(&mult_j_sub, j + 1, true);
            let f1_f2_mult = mult_univariate(&f1_j_coeffs, &f2_j_coeffs);
            mult_univariate(&f1_f2_mult, &mult_j_coeffs)
        })
        .reduce(|| vec![], |a, b| add_univariate(&a, &b));
        let g_j = add_univariate(&g_j_add, &g_j_mult);
        proof.push(g_j.clone());

        let mimc_gj_coeffs = g_j.iter().map(|s| convert_s_to_fr(s)).collect();
        let r_n = mimc.multi_hash(mimc_gj_coeffs, &Fr::from(0));
        r.push(convert_fr_to_s(r_n));
    }
    let mut f1_v = f1.clone();
    let mut f2_v = f2.clone();
    let mut add_v = add_i.clone();
    let mut mult_v = mult_i.clone();
    f1_v = partial_eval(f1_v, &r);
    f2_v = partial_eval(f2_v, &r);
    add_v = partial_eval_binary_form(&add_v, &r);
    mult_v = partial_eval_binary_form(&mult_v, &r);

    let f1_v_coeffs = get_univariate_coeff(&f1_v, 1, false);
    let f2_v_coeffs = get_univariate_coeff(&f2_v, 1, false);
    let add_v_coeffs = get_univariate_coeff(&add_v, 1, true);
    let mult_v_coeffs = get_univariate_coeff(&mult_v, 1, true);
    let f1_f2_add = add_univariate(&f1_v_coeffs, &f2_v_coeffs);
    let f1_f2_mult = mult_univariate(&f1_v_coeffs, &f2_v_coeffs);
    let add = mult_univariate(&f1_f2_add, &add_v_coeffs);
    let mult = mult_univariate(&f1_f2_mult, &mult_v_coeffs);
    let f = add_univariate(&add, &mult);
    proof.push(f.clone());
    let mimc_gv_coeffs = f.iter().map(|s| convert_s_to_fr(s)).collect();
    let r_v = mimc.multi_hash(mimc_gv_coeffs, &Fr::from(0));
    r.push(convert_fr_to_s(r_v));

    (proof, r)
}

pub fn prove_sumcheck<S: PrimeField<Repr = [u8; 32]> + std::hash::Hash>(
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
            let idx = i + 2;
            g_1_sub = partial_eval_i(&g_1_sub, &x_i, idx);
        }
        g_1 = add_poly(&g_1, &g_1_sub);
    }
    let g_1_coeffs = get_univariate_coeff(&g_1, 1, false);
    proof.push(g_1_coeffs.clone());

    let mimc_g1_coeffs = g_1_coeffs.iter().map(|s| convert_s_to_fr(s)).collect();
    let r_1 = mimc.multi_hash(mimc_g1_coeffs, &Fr::from(0));
    r.push(convert_fr_to_s(r_1));

    for j in 1..v - 1 {
        let mut g_j: Vec<Vec<S>> = g.clone();
        let assignments: Vec<Vec<S>> = generate_binary(v - j - 1);

        for (i, r_i) in r.iter().enumerate() {
            g_j = partial_eval_i(&g_j, r_i, i + 1);
        }
        let mut res_g_j = get_empty(v);
        for assignment in assignments {
            let mut g_j_sub = g_j.clone();
            for (i, x_i) in assignment.into_iter().enumerate() {
                let idx = j + i + 2;
                g_j_sub = partial_eval_i(&g_j_sub, &x_i, idx);
            }
            res_g_j = add_poly(&res_g_j, &g_j_sub);
        }
        let g_j_coeffs = get_univariate_coeff(&res_g_j, j + 1, false);
        proof.push(g_j_coeffs.clone());

        let mimc_gj_coeffs = g_j_coeffs.iter().map(|s| convert_s_to_fr(s)).collect();
        let r_n = mimc.multi_hash(mimc_gj_coeffs, &Fr::from(0));
        r.push(convert_fr_to_s(r_n));
    }
    let g_v = partial_eval(g.clone(), &r);
    let g_v_coeffs = get_univariate_coeff(&g_v, 1, false);
    proof.push(g_v_coeffs.clone());
    let mimc_gv_coeffs = g_v_coeffs.iter().map(|s| convert_s_to_fr(s)).collect();
    let r_v = mimc.multi_hash(mimc_gv_coeffs, &Fr::from(0));
    r.push(convert_fr_to_s(r_v));

    (proof, r)
}
