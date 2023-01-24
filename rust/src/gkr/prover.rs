use super::{poly::*, sumcheck::*, GKRCircuit, Input, Proof};
use ff::PrimeField;
use mimc_rs::{Fr, Mimc7};
use std::vec;

pub fn prove<S: PrimeField<Repr = [u8; 32]> + std::hash::Hash>(
    circuit: GKRCircuit<S>,
    input: Input<S>,
) -> Proof<S> {
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
            add_res = partial_eval_binary_form(add, &z[i]);
        }
        let mult = circuit.mult(i);
        let mut mult_res = vec![];
        if z[i].len() == 0 {
            mult_res = mult.clone();
        } else {
            mult_res = partial_eval_binary_form(mult, &z[i]);
        }

        let w_i_ext_b = input.w(i + 1).clone();
        let w_i_ext_c = modify_poly_from_k(input.w(i + 1), circuit.k(i + 1));

        let w_i_ext_add = add_poly(&w_i_ext_b, &w_i_ext_c);
        let first = mult_poly(&add_res, &w_i_ext_add);

        let w_i_ext_mult = mult_poly(&w_i_ext_b, &w_i_ext_c);
        let second = mult_poly(&mult_res, &w_i_ext_mult);

        let f = add_poly(&first, &second);
        let (sumcheck_proof, r) = prove_sumcheck(&f, 2 * circuit.k(i + 1));
        sumcheck_proofs.push(sumcheck_proof.clone());
        sumcheck_r.push(r.clone());

        let mut b_star = vec![];
        let mut c_star = vec![];
        b_star.extend_from_slice(&r[..circuit.k(i + 1)]);
        c_star.extend_from_slice(&r[circuit.k(i + 1)..]);

        let next_w = input.w(i + 1);
        let q_i = reduce_multiple_polynomial(&b_star, &c_star, next_w);

        q.push(q_i);

        let mut f_modified = f.clone();
        let mut f_modified_uni: Vec<S> = vec![];
        for (j, x) in r.iter().enumerate() {
            if j == r.len() - 1 {
                f_res.push(eval_univariate(&f_modified_uni, x))
            } else {
                f_modified = partial_eval_i(&f_modified, x, j + 1);
                if j == r.len() - 2 {
                    f_modified_uni = get_univariate_coeff(&f_modified, r.len());
                }
            }
        }

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
