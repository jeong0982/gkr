from ethsnarks import field
from typing import Callable
from util import length_expansion

class term:
    def __init__(self, coeff: field.FQ, i: int, const: field.FQ) -> None:
        self.coeff = coeff
        self.x_i = i
        self.const = const
    
    def eval(self, x: field.FQ):
        return self.coeff * x + self.const

    def is_constant(self):
        if self.coeff == field.FQ.zero():
            return True
        else:
            return False
    
    def convert(self):
        return UnivariateExpansion([self.const, self.coeff], 1)

    def __mul__(self, other):
        if isinstance(other, field.FQ):
            return term(self.coeff * other, self.x_i, self.const * other)

class monomial:
    def __init__(self, coeff: field.FQ, terms: list[term]) -> None:
        self.terms = terms
        self.coeff = coeff

    def mult(self, n):
        self.coeff *= n

    def __mul__(self, other):
        return monomial(self.coeff * other.coeff, self.terms + other.terms)

    def apply(self):
        res = self.coeff
        new_terms = []
        if self.coeff == field.FQ.zero():
            return field.FQ.zero()
        for t in self.terms:
            if t.coeff == field.FQ.zero():
                if t.const == field.FQ.zero():
                    return field.FQ.zero()
                res *= t.const
            else:
                new_terms.append(t)
        if new_terms == []:
            return res
        return monomial(res, new_terms)

    # univariate
    def eval_univariate(self, x: field.FQ):
        res = self.coeff
        for t in self.terms:
            res_t = t.eval(x)
            if res_t == field.FQ.zero():
                return field.FQ.zero()
            else:
                res *= res_t
        return res
    
    def get_expansion(self):
        res = self.terms[0].convert() * self.coeff
        if len(self.terms) == 1:
            return res
        else:
            for t in self.terms[1:]:
                res *= t
            return res


class polynomial:
    def __init__(self, terms: list[monomial], c=field.FQ.zero()) -> None:
        self.terms = terms
        self.constant = c

    def __add__(self, other):
        return polynomial(self.terms + other.terms, self.constant + other.constant)
    
    def __mul__(self, other):
        new_terms = []
        for a in self.terms:
            for b in other.terms:
                new_terms.append(a * b)
        for a in self.terms:
            if other.constant != field.FQ.zero():
                new_terms.append(monomial(a.coeff * other.constant, a.terms))
        for b in other.terms:
            if self.constant != field.FQ.zero():
                new_terms.append(monomial(b.coeff * self.constant, b.terms))
        new_constant = self.constant * other.constant
        return polynomial(new_terms, new_constant)
    
    def eval_i(self, x_i: field.FQ, i: int):
        new_terms_poly = []
        new_constant = self.constant
        for mono in self.terms:
            new_terms = []
            result = mono.coeff
            for term in mono.terms:
                if term.x_i == i:
                    subres = term.eval(x_i)
                    if subres == field.FQ.zero():
                        new_terms = []
                        result = field.FQ.zero()
                        break
                    else:
                        result *= subres
                else:
                    new_terms.append(term)
            if len(new_terms) == 0:
                new_constant += result
            else:
                new_mono = monomial(result, new_terms)
                new_terms_poly.append(new_mono)
        poly = polynomial(new_terms_poly, new_constant).apply_all()
        return poly

    def is_univariate(self):
        i = 0
        for term in self.terms:
            for t in term.terms:
                if i == 0:
                    i = t.x_i
                else:
                    if i != t.x_i:
                        return False
                    else:
                        continue
        if i != 0:
            return True
        else:
            return False

    def apply_all(self):
        new_terms = []
        new_const = self.constant
        for t in self.terms:
            subres = t.apply()
            if isinstance(subres, field.FQ):
                new_const += subres
            else:
                new_terms.append(subres)
        return polynomial(new_terms, new_const)

    # for univariate
    def eval_univariate(self, x: field.FQ):
        res = field.FQ.zero()
        for term in self.terms:
            res += term.eval_univariate(x)
        return res + self.constant

    def get_highest_degree(self):
        highest = 0
        for term in self.terms:
            if len(term.terms) > highest:
                highest = len(term.terms)
        return highest
    
    def get_all_coefficients(self):
        p = self.apply_all()
        exp = p.get_expansion()
        return list(reversed(exp.coeffs))

    def get_expansion(self):
        res = UnivariateExpansion([field.FQ.zero()], 0)
        for t in self.terms:
            res += t.get_expansion()
        return res

class UnivariateExpansion:
    def __init__(self, coeffs: list[field.FQ], deg: int) -> None:
        self.coeffs = coeffs
        self.deg = deg

    def __add__(self, other):
        new_coeffs = []
        highest_deg = self.deg if self.deg >= other.deg else other.deg

        a_c = length_expansion(self.coeffs, highest_deg + 1)
        b_c = length_expansion(other.coeffs, highest_deg + 1)

        for i in range(highest_deg + 1):
            new_coeffs.append(a_c[i] + b_c[i])
        return UnivariateExpansion(new_coeffs, highest_deg)
    
    def __mul__(self, other):
        if isinstance(other, term):
            m = list(map(lambda x: x * other.coeff, self.coeffs))
            m.insert(0, field.FQ.zero())
            m_exp = UnivariateExpansion(m, self.deg + 1)
            c = list(map(lambda x: x * other.const, self.coeffs))
            c_exp = UnivariateExpansion(c, self.deg)
            return m_exp + c_exp
        elif isinstance(other, field.FQ):
            return UnivariateExpansion(list(map(lambda x: x * other, self.coeffs)), self.deg)
        else:
            raise NotImplementedError

# [[coeff, deg(x_1), ... , deg(x_v)], ...]
class MultivariateExpansion:
    def __init__(self, terms: list[list[field.FQ]], v: int) -> None:
        self.terms = terms
        self.v = v
    
    def __mul__(self, other):
        if isinstance(other, term):
            res = []
            for t in self.terms:
                new_t1 = t[:]
                i = other.x_i
                new_t1[i] += 1
                new_t1[0] *= other.coeff
                res.append(new_t1)

                new_t2 = t[:]
                new_t2[0] *= other.const
                res.append(new_t2)
            return MultivariateExpansion(res, self.v)
    
    def __add__(self, other):
        if isinstance(other, MultivariateExpansion):
            assert (self.v == other.v)
            return MultivariateExpansion(self.terms + other.terms, self.v)


# generate input {0, 1}^(bit_count)
def generate_binary(bit_count) -> list[list[field.FQ]]:
    binary = []

    def genbin(n, bs=[]):
        if len(bs) == n:
            binary.append(bs)
        else:
            b_zero = bs + [field.FQ.zero()]
            b_one = bs + [field.FQ.one()]
            genbin(n, b_zero)
            genbin(n, b_one)

    genbin(bit_count)
    return binary

# univariate
def eval_univariate(coeffs: list[field.FQ], x: field.FQ):
    result = coeffs[0]
    for i in range(1, len(coeffs)):
        result *= x
        result += coeffs[i]
    return result

# for multilinear extension
# w = {0, 1}^v
# multilinear Lagrange basis polynomials
def chi(w: list[field.FQ], x: list[field.FQ]):
    prod = field.FQ.one()
    for i in range(len(x)):
        prod = prod * (x[i]*w[i] + (field.FQ.one() - x[i])*(field.FQ.one() - w[i]))
    return prod

def chi_w(w: list[field.FQ]):
    prod = []
    for i, w_i in enumerate(w):
        if w_i == field.FQ.zero():
            prod.append(term(field.FQ(-1), i + 1, field.FQ(1)))
        elif w_i == field.FQ.one():
            prod.append(term(field.FQ(1), i + 1, field.FQ(0)))
    
    mono = monomial(field.FQ.one(), prod)
    return mono

# for f(x) in gkr
def chi_w_from_k(w: list[field.FQ], k: int):
    prod = []
    for i, w_i in enumerate(w):
        if w_i == field.FQ.zero():
            prod.append(term(field.FQ(-1), i + k, field.FQ(1)))
        elif w_i == field.FQ.one():
            prod.append(term(field.FQ(1), i + k, field.FQ(0)))
    
    mono = monomial(field.FQ.one(), prod)
    return mono

def eval_ext(f: Callable[[list[field.FQ]], field.FQ], r: list[field.FQ]):
    w = generate_binary(len(r))
    acc = field.FQ.zero()
    for w_i in w:
        acc += f(w_i) * chi(w_i, r)
    return acc

def eval_expansion(f: list[list[field.FQ]], r: list[field.FQ]) -> field.FQ:
    assert (len(r) + 1 == len(f[0]))
    res = field.FQ.zero()
    for t in f:
        subres = field.FQ.zero()
        for i, x in enumerate(t):
            if i == 0:
                subres = t[0]
            else:
                subres *= r[i - 1] ** x
        res += subres
    return res

# return expansion of multivariate polynomial
def get_multi_ext(f: Callable[[list[field.FQ]], field.FQ], v: int) -> list[list[field.FQ]]:
    w_set = generate_binary(v)
    ext_f = []
    res = []
    for w in w_set:
        res = chi_w(w)
        if f(w) == field.FQ.zero():
            continue
        res.mult(f(w))
        ext_f.append(res)

    g = []
    term_pool = dict()

    empty_term = [field.FQ.zero()] * (v + 1)
    for term in ext_f:
        subres = MultivariateExpansion([], v)
        for t in term.terms:
            if len(subres.terms) == 0:
                t_expansion1 = empty_term[:]
                t_expansion1[t.x_i] = field.FQ.one()
                t_expansion1[0] = term.coeff * t.coeff
                t_expansion2 = empty_term[:]
                t_expansion2[0] = t.const * term.coeff
                subres = MultivariateExpansion([t_expansion1, t_expansion2], v)
            else:
                subres = subres * t
        for one_term in subres.terms:
            if tuple(one_term[1:]) in term_pool:
                idx = term_pool[tuple(one_term[1:])]
                g[idx][0] += one_term[0]
            else:
                term_pool[tuple(one_term[1:])] = len(g)
                g.append(one_term)
    if len(g) == 0:
        g = [empty_term]
        return g
    g_final = []
    for term in g:
        if term[0] != field.FQ.zero():
            g_final.append(term)
    return g_final

# r : {0, 1}^v
def get_ext(f: Callable[[list[field.FQ]], field.FQ], v: int) -> polynomial:
    w_set = generate_binary(v)
    ext_f = []
    for w in w_set:
        res = chi_w(w)
        if f(w) == field.FQ.zero():
            continue
        res.mult(f(w))
        ext_f.append(res)
    return polynomial(ext_f)

def get_ext_from_k(f: Callable[[list[field.FQ]], field.FQ], v: int, k: int) -> polynomial:
    w_set = generate_binary(v)
    ext_f = []
    for w in w_set:
        res = chi_w_from_k(w, k)
        if f(w) == field.FQ.zero():
            continue
        res.mult(f(w))
        ext_f.append(res)
    return polynomial(ext_f)
