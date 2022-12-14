use ethers_core::types::U256;
use ff::PrimeField;
use std::{collections::HashMap, vec};

fn fe_to_u256<F>(f: F) -> U256
where
    F: PrimeField<Repr = [u8; 32]>,
{
    U256::from_little_endian(f.to_repr().as_ref())
}

pub fn get_empty<S: PrimeField>(l: usize) -> Vec<Vec<S>> {
    vec![vec![S::ZERO; l + 1]; 1]
}

fn minus_one<S: PrimeField>() -> S {
    S::ZERO - S::ONE
}

fn constant_one<S: PrimeField>(l: usize) -> Vec<S> {
    let mut vec = vec![S::ZERO; l];
    vec[0] = S::ONE;
    vec
}

fn chi_w<S: PrimeField>(w: Vec<usize>) -> Vec<Vec<S>> {
    let mut prod = Vec::new();
    let l = w.len();
    for (i, w_i) in w.iter().enumerate() {
        let idx = i + 1;
        if *w_i == 0 {
            let mut subres = vec![];
            let mut term = constant_one::<S>(l);
            term[idx] = minus_one();
            let one = constant_one::<S>(l);
            subres.push(term);
            subres.push(one);
            prod.push(subres);
        } else if *w_i == 1 {
            let mut term = constant_one::<S>(l);
            term[idx] = S::ONE;
            let mut subres = vec![];
            subres.push(term);
            prod.push(subres);
        }
    }
    let mut res = vec![];
    res.push(constant_one::<S>(l));
    for term in prod {}
    res
}

fn generate_binary_string(l: usize) -> Vec<String> {
    if l == 0 {
        return vec![];
    } else if l == 1 {
        return vec!["0".to_string(), "1".to_string()];
    } else {
        let mut result = vec![];
        let substrings = generate_binary_string(l - 1);
        for s in substrings {
            result.push(format!("{}0", s));
            result.push(format!("{}1", s));
        }
        return result;
    }
}

pub fn generate_binary<S: PrimeField>(l: usize) -> Vec<Vec<S>> {
    fn genbin<S: PrimeField>(n: usize, current: usize, acc: Vec<Vec<S>>) -> Vec<Vec<S>> {
        if current == n {
            acc
        } else {
            let mut new_acc = vec![];
            if acc.len() == 0 {
                let b_zero = vec![S::ZERO];
                let b_one = vec![S::ONE];
                new_acc.push(b_zero);
                new_acc.push(b_one);
            } else {
                for b in acc {
                    let mut b_zero = b.clone();
                    let mut b_one = b.clone();
                    b_zero.push(S::ZERO);
                    b_one.push(S::ONE);
                    new_acc.push(b_zero);
                    new_acc.push(b_one);
                }
            }
            genbin(n, current + 1, new_acc)
        }
    }
    genbin(l, 0, vec![])
}

pub fn partial_eval_i<S: PrimeField<Repr = [u8; 32]>>(
    f: &Vec<Vec<S>>,
    x: &S,
    i: usize,
) -> Vec<Vec<S>> {
    let mut res_f = vec![];
    for t in f.iter() {
        let mut new_t = t.clone();
        let exp = fe_to_u256(t[i]).as_usize();
        let mut x_pow = *x;
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

pub fn eval_univariate<S: PrimeField<Repr = [u8; 32]>>(f: &Vec<S>, x: &S) -> S {
    let mut res = f[0];
    for i in f.iter().skip(1) {
        res *= x;
        res += *i;
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

pub fn get_univariate_coeff<S: PrimeField<Repr = [u8; 32]>>(f: &Vec<Vec<S>>, i: usize) -> Vec<S> {
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

fn mult_univariate<S: PrimeField<Repr = [u8; 32]>>(p: Vec<S>, q: Vec<S>) -> Vec<S> {
    let h_deg_p = p.len() - 1;
    let h_deg_q = q.len() - 1;
    let h_deg = h_deg_p + h_deg_q;
    let mut res = vec![S::ZERO; h_deg + 1];
    for (i, p_i) in p.iter().enumerate() {
        for (j, q_i) in q.iter().enumerate() {
            let deg = i + j;
            let coeff = *p_i * (*q_i);
            res[deg] += coeff;
        }
    }
    res
}

fn add_univariate<S: PrimeField<Repr = [u8; 32]>>(p: Vec<S>, q: Vec<S>) -> Vec<S> {
    let h_deg = std::cmp::max(p.len(), q.len());
    let mut res = vec![S::ZERO; h_deg];
    for i in 0..h_deg {
        if i > p.len() - 1 {
            res[i] = q[i];
        } else if i > q.len() - 1 {
            res[i] = p[i];
        } else {
            res[i] = p[i] + q[i];
        }
    }
    res
}

pub fn reduce_multiple_polynomial<S: PrimeField<Repr = [u8; 32]>>(
    b: &Vec<S>,
    c: &Vec<S>,
    w: Vec<Vec<S>>,
) -> Vec<S> {
    assert_eq!(b.len(), c.len());
    let mut res = vec![];
    let mut t = vec![];
    let iterator = b.iter().zip(c.iter());
    for (b_i, c_i) in iterator {
        let gradient = *c_i - *b_i;
        let new_const = *b_i;
        t.push((new_const, gradient));
    }
    for terms in w {
        let mut new_poly = vec![S::ONE];
        for (i, d) in terms.iter().enumerate() {
            if i == 0 {
                new_poly[0] = *d;
                continue;
            }
            let idx = i - 1;
            let deg = fe_to_u256(*d).as_usize();
            for _ in 0..deg {
                let term = vec![t[idx].0, t[idx].1];
                new_poly = mult_univariate(new_poly, term);
            }
        }
        res = add_univariate(res, new_poly);
    }
    res
}

pub fn get_multi_ext<S: PrimeField<Repr = [u8; 32]>>(
    b: HashMap<String, S>,
    v: usize,
) -> Vec<Vec<S>> {
    let binary = generate_binary_string(v);
    let mut res = vec![];
    for idx in binary {
        if !b.contains_key(&idx) || b.get(&idx).unwrap().clone() == S::ZERO {
            continue;
        } else {
            todo!()
        }
    }
    res
}

pub fn l_function<S: PrimeField<Repr = [u8; 32]>>(b: &Vec<S>, c: &Vec<S>, r: &S) -> Vec<S> {
    let mut res = vec![];
    let mut t = vec![];
    let iterator = b.iter().zip(c.iter());
    for (b_i, c_i) in iterator {
        let gradient = *c_i - *b_i;
        let new_const = *b_i;
        t.push((new_const, gradient));
    }
    for t_i in t {
        res.push(t_i.0 + t_i.1 * (*r));
    }
    res
}
