use std::vec;

use ff::PrimeField;

use super::poly::{add_poly, generate_binary, get_empty, partial_eval_i};

pub fn prove_sumcheck<S: PrimeField>(g: Vec<Vec<S>>, v: usize) -> (Vec<Vec<S>>, Vec<S>)
where
    <S as PrimeField>::Repr: AsRef<[u64]>,
{
    let mut proof = vec![];
    let mut r = vec![];

    let mut g_1 = get_empty(v);
    let assignments: Vec<Vec<S>> = generate_binary(v - 1);
    for assignment in assignments {
        let mut g_1_sub = g.clone();
        for (i, x_i) in assignment.into_iter().enumerate() {
            let idx = i + 1;
            g_1_sub = partial_eval_i(g_1_sub, x_i, idx);
        }
        g_1 = add_poly(&g_1, &g_1_sub);
    }

    

    (proof, r)
}
