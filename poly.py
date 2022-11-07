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
        return expansion([self.const, self.coeff], 1)

class monomial:
    def __init__(self, coeff: field.FQ, terms: list[term]) -> None:
        self.terms = terms
        self.coeff = coeff

    def mult(self, n):
        self.coeff *= n

    def __mul__(self, other):
        return monomial(self.coeff * other.coeff, self.terms + other.terms)

    def apply(self):
        res = field.FQ.one()
        new_terms = []
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
        res = field.FQ.one()
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
                    new_terms.append(term)
            if len(new_terms) == 0:
                new_constant += result
            else:
                new_mono = monomial(result, new_terms)
                new_terms_poly.append(new_mono)
        return polynomial(new_terms_poly, new_constant)

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
                        return True

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
        res = expansion([], 0)
        for t in self.terms:
            res += t.get_expansion()
        return res

class expansion:
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
        return expansion(new_coeffs, highest_deg)
    
    def __mul__(self, other):
        if isinstance(other, term):
            m = list(map(lambda x: x * other.coeff, self.coeffs))
            m.insert(0, field.FQ.zero())
            m_exp = expansion(m, self.deg + 1)
            c = list(map(lambda x: x * other.const, self.coeffs))
            c_exp = expansion(c, self.deg)
            return m_exp + c_exp
        elif isinstance(other, field.FQ):
            return expansion(list(map(lambda x: x * other, self.coeffs)), self.deg)
        else:
            raise NotImplementedError
    

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
    result = coeffs[len(coeffs) - 1]
    for i in range(len(coeffs) - 2, 0, -1):
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
            prod.append(term(field.FQ(-1), i, field.FQ(1)))
        elif w_i == field.FQ.one():
            prod.append(term(field.FQ(1), i, field.FQ(0)))
    
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

# r : {0, 1}^v
def get_ext(f: Callable[[list[field.FQ]], field.FQ], v: int) -> polynomial:
    w_set = generate_binary(v)
    ext_f = []
    for w in w_set:
        res = chi_w(w)
        res.mult(f(w))
        ext_f.append(res)
    return polynomial(ext_f)

def get_ext_from_k(f: Callable[[list[field.FQ]], field.FQ], v: int, k: int) -> polynomial:
    w_set = generate_binary(v)
    ext_f = []
    for w in w_set:
        res = chi_w_from_k(w, k)
        res.mult(f(w))
        ext_f.append(res)
    return polynomial(ext_f)
