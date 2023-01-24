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
    vec![vec![S::zero(); l + 1]; 1]
}

fn minus_one<S: PrimeField>() -> S {
    S::zero() - S::one()
}

fn constant_one<S: PrimeField>(l: usize) -> Vec<S> {
    let mut vec = vec![S::zero(); l + 1];
    vec[0] = S::one();
    vec
}

fn mult_multi_poly<S: PrimeField>(l: &Vec<S>, r: &Vec<S>) -> Vec<S> {
    let length = l.len();
    let mut res = vec![];
    for i in 0..length {
        if i == 0 {
            res.push(l[i] * r[i]);
        } else {
            res.push(l[i] + r[i])
        };
    }
    res
}

/// For add_i and mult_i, they have only two types for term, x or 1 - x.
/// Represent x as 1, (1 - x) as 2.
pub fn chi_w_for_binary<S: PrimeField>(w: &String) -> Vec<Vec<S>> {
    let l = w.len();
    let mut prod = constant_one::<S>(l);
    for (i, w_i) in w.chars().enumerate() {
        if w_i == '0' {
            // 1 - x_i
            prod[i + 1] = S::one();
        } else if w_i == '1' {
            // x_i
            prod[i + 1] = S::one() + S::one();
        }
    }
    vec![prod]
}

pub fn partial_eval_binary_form<S: PrimeField>(f: &Vec<Vec<S>>, x: &Vec<S>) -> Vec<Vec<S>> {
    let l = x.len();
    let mut new_f = vec![];
    for term in f {
        let mut new_term = vec![];
        let mut new_const = term[0];
        for i in 0..l {
            let idx = i + 1;
            if term[idx] == S::one() {
                new_const *= S::one() - x[i];
            } else if term[idx] == S::one() + S::one() {
                new_const *= x[i];
            }
        }
        new_term.push(new_const);
        new_term.extend_from_slice(&term[l + 1..]);
        new_f.push(new_term);
    }
    new_f
}

pub fn partial_eval_i_binary_form<S: PrimeField<Repr = [u8; 32]>>(
    f: &Vec<Vec<S>>,
    x: &S,
    i: usize,
) -> Vec<Vec<S>> {
    let mut res_f = vec![];
    for t in f.iter() {
        let mut new_t = t.clone();
        let mut constant = t[0];
        if t[i] == S::one() {
            constant *= (S::one() - x);
        } else if t[i] == S::one() + S::one() {
            constant *= x;
        }
        new_t[0] = constant;
        new_t[i] = S::zero();
        res_f.push(new_t);
    }
    res_f
}

pub fn chi_w<S: PrimeField>(w: &String) -> Vec<Vec<S>> {
    let l = w.len();
    let mut prod_single = constant_one::<S>(l);
    let mut prod_double = Vec::new();
    for (i, w_i) in w.chars().enumerate() {
        let idx = i + 1;
        if w_i == '0' {
            let mut subres = vec![];
            let mut term = constant_one::<S>(l);
            term[0] = minus_one();
            term[idx] = S::one();
            let one = constant_one::<S>(l);
            subres.push(term);
            subres.push(one);
            prod_double.push(subres);
        } else if w_i == '1' {
            prod_single[idx] = S::one();
        }
    }
    let mut res = vec![prod_single];
    for poly in prod_double {
        let mut new_res = vec![];
        for term in poly.iter() {
            for res_term in res.iter() {
                new_res.push(mult_multi_poly(term, res_term));
            }
        }
        res = new_res;
    }
    res
}

pub fn generate_binary_string(l: usize) -> Vec<String> {
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
                let b_zero = vec![S::zero()];
                let b_one = vec![S::one()];
                new_acc.push(b_zero);
                new_acc.push(b_one);
            } else {
                for b in acc {
                    let mut b_zero = b.clone();
                    let mut b_one = b.clone();
                    b_zero.push(S::zero());
                    b_one.push(S::one());
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
        let mut x_pow = S::one();
        for _ in 0..exp {
            x_pow *= x;
        }
        let constant = t[0] * x_pow;
        new_t[0] = constant;
        new_t[i] = S::zero();
        res_f.push(new_t);
    }
    res_f
}

pub fn partial_eval<S: PrimeField<Repr = [u8; 32]>>(f: Vec<Vec<S>>, r: &Vec<S>) -> Vec<Vec<S>> {
    assert!(f[0].len() > r.len());
    if r.len() == 0 {
        return f;
    }
    let mut res_f = vec![];
    for t in f.iter() {
        let mut new_t = vec![];
        let mut constant = t[0];
        for i in 0..r.len() {
            let x = fe_to_u256(t[i + 1]).as_usize();
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
        let mut zeros = vec![S::zero(); k];
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
        let mut zeros = vec![S::zero(); l - f.len()];
        new_f.append(&mut zeros);
        new_f
    }
}

pub fn add_poly<S: PrimeField + std::hash::Hash>(
    f1: &Vec<Vec<S>>,
    f2: &Vec<Vec<S>>,
) -> Vec<Vec<S>> {
    let mut map: HashMap<Vec<S>, S> = HashMap::new();
    let mut len1 = 0;
    let mut len2 = 0;
    if f1.len() != 0 {
        len1 = f1[0].len();
    }
    if f2.len() != 0 {
        len2 = f2[0].len();
    }
    let len = if len1 > len2 { len1 } else { len2 };
    let mut res = vec![];
    for t in f1 {
        let t_ext = extend_length(t, len);
        if map.contains_key(&t_ext[1..]) {
            *map.get_mut(&t_ext[1..]).unwrap() += t_ext[0];
        } else {
            map.insert(t_ext[1..].to_vec(), t_ext[0]);
        }
    }
    for t in f2 {
        let t_ext = extend_length(t, len);
        if map.contains_key(&t_ext[1..]) {
            *map.get_mut(&t_ext[1..]).unwrap() += t_ext[0];
        } else {
            map.insert(t_ext[1..].to_vec(), t_ext[0]);
        }
    }
    for (poly, constant) in map.iter() {
        if constant.clone() == S::zero() {
            continue;
        }
        let mut new_poly = vec![constant.clone()];
        let mut p_cloned = poly.clone();
        new_poly.append(&mut p_cloned);
        res.push(new_poly)
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

pub fn mult_poly<S: PrimeField + std::hash::Hash>(
    f1: &Vec<Vec<S>>,
    f2: &Vec<Vec<S>>,
) -> Vec<Vec<S>> {
    let mut map: HashMap<Vec<S>, S> = HashMap::new();
    let mut len1 = 0;
    let mut len2 = 0;
    if f1.len() != 0 {
        len1 = f1[0].len();
    }
    if f2.len() != 0 {
        len2 = f2[0].len();
    }
    let len = if len1 > len2 { len1 } else { len2 };

    let mut res = vec![];

    for t1 in f1 {
        for t2 in f2 {
            let t = mult_mono(extend_length(t1, len), extend_length(t2, len));
            if map.contains_key(&t[1..]) {
                *map.get_mut(&t[1..]).unwrap() += t[0];
            } else {
                map.insert(t[1..].to_vec(), t[0]);
            }
        }
    }
    for (poly, constant) in map.iter() {
        if constant.clone() == S::zero() {
            continue;
        }
        let mut new_poly = vec![constant.clone()];
        let mut p_cloned = poly.clone();
        new_poly.append(&mut p_cloned);
        res.push(new_poly)
    }
    res
}

pub fn get_univariate_coeff<S: PrimeField<Repr = [u8; 32]>>(
    f: &Vec<Vec<S>>,
    i: usize,
    is_binary_form: bool,
) -> Vec<S> {
    if is_binary_form {
        let mut coeffs = vec![S::zero(); 2];
        for t in f {
            let constant = t[0];
            if t[i] == S::one() {
                coeffs[0] += constant;
                coeffs[1] += minus_one::<S>() * constant;
            } else if t[i] == S::one() + S::one() {
                coeffs[1] += constant;
            }
        }
        coeffs
    } else {
        let mut coeffs = vec![S::zero()];
        for t in f {
            let deg_u256 = fe_to_u256(t[i]);
            let deg = deg_u256.as_usize();
            if coeffs.len() - 1 < deg {
                let mut acc = vec![S::zero(); deg - coeffs.len() + 1];
                coeffs.append(&mut acc);
            }
            coeffs[deg] += t[0];
        }
        coeffs.reverse();
        coeffs
    }
}

pub fn mult_univariate<S: PrimeField<Repr = [u8; 32]>>(p: &Vec<S>, q: &Vec<S>) -> Vec<S> {
    let h_deg_p = p.len() - 1;
    let h_deg_q = q.len() - 1;
    let mut p_rev = p.clone();
    let mut q_rev = q.clone();
    p_rev.reverse();
    q_rev.reverse();

    let h_deg = h_deg_p + h_deg_q;
    let mut res = vec![S::zero(); h_deg + 1];

    for (i, p_i) in p_rev.iter().enumerate() {
        for (j, q_i) in q_rev.iter().enumerate() {
            let deg = i + j;
            let coeff = *p_i * (*q_i);
            res[deg] += coeff;
        }
    }
    res.reverse();
    res
}

pub fn add_univariate<S: PrimeField<Repr = [u8; 32]>>(p: &Vec<S>, q: &Vec<S>) -> Vec<S> {
    if p.len() == 0 {
        return q.clone();
    } else if q.len() == 0 {
        return p.clone();
    }
    let h_deg = std::cmp::max(p.len(), q.len());
    let mut p_rev = p.clone();
    let mut q_rev = q.clone();
    p_rev.reverse();
    q_rev.reverse();
    let mut res = vec![S::zero(); h_deg];
    for i in 0..h_deg {
        if i > p.len() - 1 {
            res[i] = q_rev[i];
        } else if i > q.len() - 1 {
            res[i] = p_rev[i];
        } else {
            res[i] = p_rev[i] + q_rev[i];
        }
    }
    res.reverse();
    res
}

pub fn reduce_multiple_polynomial<S: PrimeField<Repr = [u8; 32]>>(
    b: &Vec<S>,
    c: &Vec<S>,
    w: Vec<Vec<S>>,
) -> Vec<S> {
    assert_eq!(b.len(), c.len());
    let mut res = vec![S::zero()];
    let mut t = vec![];
    let iterator = b.iter().zip(c.iter());
    for (b_i, c_i) in iterator {
        let gradient = *c_i - *b_i;
        let new_const = *b_i;
        t.push((new_const, gradient));
    }
    for terms in w {
        let mut new_poly = vec![S::one()];
        for (i, d) in terms.iter().enumerate() {
            if i == 0 {
                new_poly[0] = d.clone();
                continue;
            }
            let idx = i - 1;
            let deg = fe_to_u256(*d).as_usize();
            for _ in 0..deg {
                let term = vec![t[idx].1, t[idx].0];
                new_poly = mult_univariate(&new_poly, &term);
            }
        }
        res = add_univariate(&res, &new_poly);
    }
    res
}

pub fn get_multi_ext<S: PrimeField<Repr = [u8; 32]>>(value: &Vec<S>, v: usize) -> Vec<Vec<S>> {
    let binary = generate_binary_string(v);
    let mut polynomial: Vec<Vec<S>> = vec![];
    for b in binary {
        let idx = usize::from_str_radix(&b, 2).unwrap();
        let val = value[idx];
        if val == S::zero() {
            continue;
        }
        let mut res = chi_w::<S>(&b);
        for i in 0..res.len() {
            res[i][0] *= val
        }
        polynomial.append(&mut res);
    }
    polynomial
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
