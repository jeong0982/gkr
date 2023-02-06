use super::{poly::*, sumcheck::*, GKRCircuit, Input, Proof};
use ff::PrimeField;
use mimc_rs::{Fr, Mimc7};
use std::vec;

pub fn prove<S: PrimeField<Repr = [u8; 32]> + std::hash::Hash>(
    circuit: GKRCircuit<S>,
    input: Input<S>,
) -> Proof<S> {
    println!("Proving starts..");
    let mimc = Mimc7::new(91);

    let mut sumcheck_proofs = vec![];
    let mut sumcheck_r = vec![];
    let mut q = vec![];
    let mut f_res = vec![];
    let mut r_stars = vec![];
    let mut z_zero = vec![];
    for _ in 0..circuit.layer[0].k {
        z_zero.push(S::zero());
    }
    let mut z = vec![];
    z.push(z_zero);

    for i in 0..circuit.depth() {
        let add = circuit.add(i);
        let mut add_res = vec![];
        if z[i].len() == 0 {
            add_res = add.clone();
        } else {
            add_res = partial_eval_binary_form(&add, &z[i]);
        }
        let mult = circuit.mult(i);
        let mut mult_res = vec![];
        if z[i].len() == 0 {
            mult_res = mult.clone();
        } else {
            mult_res = partial_eval_binary_form(&mult, &z[i]);
        }
        let w_i = input.w(i + 1).clone();
        let mut w_i_ext_b = vec![];
        for t in w_i.iter() {
            w_i_ext_b.push(extend_length(t, 2 * circuit.k(i + 1) + 1));
        }
        let w_i_ext_c = modify_poly_from_k(&input.w(i + 1), circuit.k(i + 1));

        let (sumcheck_proof, r) = prove_sumcheck_opt(
            &circuit.add_wire(i),
            &circuit.mult_wire(i),
            &add_res,
            &mult_res,
            &w_i_ext_b,
            &w_i_ext_c,
            2 * circuit.k(i + 1),
        );
        sumcheck_proofs.push(sumcheck_proof.clone());
        sumcheck_r.push(r.clone());

        let mut b_star = vec![];
        let mut c_star = vec![];
        b_star.extend_from_slice(&r[..circuit.k(i + 1)]);
        c_star.extend_from_slice(&r[circuit.k(i + 1)..]);

        let next_w = input.w(i + 1);
        let q_i = reduce_multiple_polynomial(&b_star, &c_star, &next_w);

        q.push(q_i);

        let r_sub = &r[0..r.len() - 1].to_vec();
        let w_i_b_res = partial_eval_from(&w_i_ext_b, r_sub, 1);
        let w_i_c_res = partial_eval_from(&w_i_ext_c, r_sub, 1);
        let add_i_res = partial_eval_from_binary_form(&add_res, r_sub, 1);
        let mult_i_res = partial_eval_from_binary_form(&mult_res, r_sub, 1);

        let w_i_b_coeffs = get_univariate_coeff(&w_i_b_res, r.len(), false);
        let w_i_c_coeffs = get_univariate_coeff(&w_i_c_res, r.len(), false);
        let add_coeffs = get_univariate_coeff(&add_i_res, r.len(), true);
        let mult_coeffs = get_univariate_coeff(&mult_i_res, r.len(), true);
        let b_c_add = add_univariate(&w_i_b_coeffs, &w_i_c_coeffs);
        let b_c_mult = mult_univariate(&w_i_b_coeffs, &w_i_c_coeffs);
        let add = mult_univariate(&b_c_add, &add_coeffs);
        let mult = mult_univariate(&b_c_mult, &mult_coeffs);
        let f_modified_uni = add_univariate(&add, &mult);

        f_res.push(eval_univariate(&f_modified_uni, &r[r.len() - 1]));

        let mimc_r_star = sumcheck_proof[sumcheck_proof.len() - 1]
            .iter()
            .map(|s| convert_s_to_fr(s))
            .collect();
        let r_star: S = convert_fr_to_s(mimc.multi_hash(mimc_r_star, &Fr::from(0)));

        let next_r = l_function(&b_star, &c_star, &r_star);
        z.push(next_r);
        r_stars.push(r_star);
    }

    Proof {
        sumcheck_proofs,
        sumcheck_r,
        f: f_res,
        d: input.d.clone(),
        q,
        z,
        r: r_stars,
        depth: circuit.depth() + 1,
        input_func: input.w(circuit.depth()),
        add: circuit.get_add_list(),
        mult: circuit.get_mult_list(),
        k: circuit.get_k_list(),
    }
}
