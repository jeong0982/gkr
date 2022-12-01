use ethers_core::types::U256;
use ff::PrimeField;
use std::vec;

fn fe_to_u256<F>(f: F) -> U256
where
    F: PrimeField<Repr = [u8; 32]>,
{
    U256::from_little_endian(f.to_repr().as_ref())
}

pub fn get_empty<S: PrimeField>(l: usize) -> Vec<Vec<S>> {
    vec![vec![S::ZERO; l + 1]; 1]
}

pub fn generate_binary<S: PrimeField>(l: usize) -> Vec<Vec<S>> {
    fn genbin<S: PrimeField>(n: usize, current: usize, acc: Vec<Vec<S>>) -> Vec<Vec<S>> {
        if current == n {
            acc
        } else {
            let mut new_acc = vec![];
            for b in acc {
                let mut b_zero = b.clone();
                let mut b_one = b.clone();
                b_zero.push(S::ZERO);
                b_one.push(S::ONE);
                new_acc.push(b_zero);
                new_acc.push(b_one);
            }
            genbin(n, current + 1, new_acc)
        }
    }
    let empty = vec![];
    genbin(l, 0, Vec::from(empty))
}

pub fn partial_eval_i<S: PrimeField<Repr = [u8; 32]>>(
    f: Vec<Vec<S>>,
    x: &S,
    i: usize,
) -> Vec<Vec<S>> {
    let mut res_f = vec![];
    for t in f.iter() {
        let mut new_t = t.clone();
        let exp = fe_to_u256(t[i]).as_usize();
        let mut x_pow = x.clone();
        for _ in 0..exp {
            x_pow *= x;
        }
        let constant = t[0] * x_pow;
        new_t[0] = constant;
        new_t[i] = S::ZERO;
        res_f.push(new_t);
    }
    res_f
}

pub fn partial_eval<S: PrimeField<Repr = [u8; 32]>>(f: Vec<Vec<S>>, r: &Vec<S>) -> Vec<Vec<S>> {
    assert!(f[0].len() > r.len());
    let mut res_f = vec![];
    for t in f.iter() {
        let mut new_t = vec![];
        let mut constant = t[0];
        for i in 0..r.len() - 1 {
            let x = fe_to_u256(t[i]).as_usize();
            if x == 0 {
                continue;
            }
            for _ in 0..x {
                constant *= r[i];
            }
        }
        new_t.push(constant);
        new_t.extend_from_slice(&t[r.len() + 1..]);
        res_f.push(new_t);
    }
    res_f
}

pub fn eval_univariate<S: PrimeField<Repr = [u8; 32]>>(f: Vec<S>, x: &S) -> S {
    let mut res = f[0];
    for i in 1..f.len() {
        res *= x;
        res += f[i];
    }
    res
}

pub fn modify_poly_from_k<S: PrimeField>(f: Vec<Vec<S>>, k: usize) -> Vec<Vec<S>> {
    let mut res_f = vec![];
    for t in f.iter() {
        let mut new_t = vec![t[0]];
        let mut zeros = vec![S::ZERO; k];
        new_t.append(&mut zeros);
        new_t.extend_from_slice(&t[1..]);

        res_f.push(new_t);
    }
    res_f
}

fn extend_length<S: PrimeField>(f: &Vec<S>, l: usize) -> Vec<S> {
    if f.len() == l {
        f.clone()
    } else {
        let mut new_f = f.clone();
        let mut zeros = vec![S::ZERO; l - f.len()];
        new_f.append(&mut zeros);
        new_f
    }
}

pub fn add_poly<S: PrimeField>(f1: &Vec<Vec<S>>, f2: &Vec<Vec<S>>) -> Vec<Vec<S>> {
    let len1 = f1[0].len();
    let len2 = f2[0].len();
    let len = if len1 > len2 { len1 } else { len2 };
    let mut res = vec![];
    for t in f1 {
        let t_ext = extend_length(t, len);
        res.push(t_ext);
    }
    for t in f2 {
        let t_ext = extend_length(t, len);
        res.push(t_ext);
    }
    res
}

fn mult_mono<S: PrimeField>(t1: Vec<S>, t2: Vec<S>) -> Vec<S> {
    assert_eq!(t1.len(), t2.len());
    let mut res = vec![];
    for i in 0..t1.len() {
        if i == 0 {
            res.push(t1[0] * t2[0]);
        } else {
            res.push(t1[i] + t2[i]);
        }
    }
    res
}

pub fn mult_poly<S: PrimeField>(f1: &Vec<Vec<S>>, f2: &Vec<Vec<S>>) -> Vec<Vec<S>> {
    let len1 = f1[0].len();
    let len2 = f2[0].len();
    let len = if len1 > len2 { len1 } else { len2 };

    let mut res = vec![];

    for t1 in f1 {
        for t2 in f2 {
            res.push(mult_mono(extend_length(t1, len), extend_length(t2, len)));
        }
    }

    res
}

pub fn get_univariate_coeff<S: PrimeField<Repr = [u8; 32]>>(f: Vec<Vec<S>>, i: usize) -> Vec<S> {
    let mut coeffs = vec![];
    for t in f {
        let deg_u256 = fe_to_u256(t[i]);
        let deg = deg_u256.as_usize();
        if coeffs.len() + 1 < deg {
            let mut acc = vec![S::ZERO; deg - coeffs.len()];
            coeffs.append(&mut acc);
        }
        coeffs[deg] += t[0];
    }
    coeffs
}
