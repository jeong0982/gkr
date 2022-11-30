use super::{poly::*, sumcheck::prove_sumcheck, GKRCircuit, Proof};
use ff::PrimeField;
use std::vec;

pub fn prove<S: PrimeField>(circuit: GKRCircuit<S>) -> Result<Proof<S>, ()>
where
    <S as PrimeField>::Repr: AsRef<[u64]>,
{
    let mut z_zero = vec![];
    for _ in 0..circuit.layer[0].k {
        z_zero.push(S::ZERO);
    }
    let mut z = vec![];
    z.push(z_zero);

    for i in 0..circuit.depth() - 1 {
        let add = circuit.add(i);
        let add_res = partial_eval(add, &z[i]);
        let mult = circuit.mult(i);
        let mult_res = partial_eval(mult, &z[i]);

        let w_i_ext_b = modify_poly_from_k(circuit.w(i), circuit.k(i));
        let w_i_ext_c = modify_poly_from_k(circuit.w(i), circuit.k(i) + circuit.k(i + 1));

        let w_i_ext_add = add_poly(&w_i_ext_b, &w_i_ext_c);
        let first = mult_poly(&add_res, &w_i_ext_add);

        let w_i_ext_mult = mult_poly(&w_i_ext_b, &w_i_ext_c);
        let second = mult_poly(&mult_res, &w_i_ext_mult);

        let f = add_poly(&first, &second);

        let (sumcheck_proof, r) = prove_sumcheck(f, 2 * circuit.k(i + 1));
    }

    Err(())
}
