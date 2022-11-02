from ethsnarks import field
from typing import Callable

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

class monomial:
    def __init__(self, coeff: field.FQ, terms: list[term]) -> None:
        self.terms = terms
        self.coeff = coeff

    def mult(self, n):
        self.coeff *= n

    def __mult__(self, other):
        return monomial(self.coeff * other.coeff, self.terms + other.terms)

    # univariate
    def eval_univariate(self, x: field.FQ):
        res = field.FQ.one()
        for t in self.terms:
            res *= t.eval(x)
        return res
    
    def derivative(self):
        res = []
        for i in range(len(self.terms)):
            new_coeff = self.coeff * self.terms[i].coeff
            new_terms = self.terms[:]
            new_terms.pop(i)
            if len(new_terms) == 0:
                new_terms = [term(field.FQ.zero(), 0, field.FQ.one())]
            new_mono = monomial(new_coeff, new_terms)
            res.append(new_mono)
        return res

class polynomial:
    def __init__(self, terms: list[monomial]) -> None:
        self.terms = terms

    def __add__(self, other):
        return polynomial(self.terms + other.terms)
    
    def __mult__(self, other):
        new_terms = []
        for a in self.terms:
            for b in other.terms:
                new_terms.append(a * b)
        return polynomial(new_terms)
    
    def eval_i(self, x_i: field.FQ, i: int):
        new_terms_poly = []
        for mono in self.terms:
            new_terms = []
            result = mono.coeff
            for term in mono.terms:
                if term.x_i == i:
                    result *= term.eval(x_i)
                else:
                    new_terms.append(term)
            new_mono = monomial(result, new_terms)
            new_terms_poly.append(new_mono)
        return polynomial(new_terms_poly)

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

    # for univariate
    def eval_univariate(self, x: field.FQ):
        res = field.FQ.zero()
        for term in self.terms:
            res += term.eval_univariate(x)
        return res

    def get_highest_degree(self):
        highest = 0
        for term in self.terms:
            if len(term.terms) > highest:
                highest = len(term.terms)
        return highest
    
    def derivative(self):
        res = []
        for term in self.terms:
            res += term.derivative()
        return polynomial(res)
    
    def get_all_coefficients(self):
        zero = field.FQ.zero()
        
        deg = self.get_highest_degree()
        coeffs = [self.eval_univariate(zero)]
        
        d = self.derivative()
        coeffs.append(d.eval_univariate(zero))

        pi = field.FQ.one()
        for i in range(2, deg):
            pi *= field.FQ(i)
            d = d.derivative()
            coeffs.append(d.eval_univariate(zero) * field.FQ.inv(pi))
        
        coeffs.reverse()
        return coeffs
                

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

def eval_ext(f: Callable[[list[field.FQ]], field.FQ], r: list[field.FQ]):
    w = generate_binary(len(r))
    acc = field.FQ.zero()
    for w_i in w:
        acc += f(w_i) * chi(w_i, r)
    return acc

# r : {0, 1}^v
def get_ext(f: Callable[[list[field.FQ]], field.FQ], v: int):
    w_set = generate_binary(v)
    ext_f = []
    for w in w_set:
        new_mono = chi_w(w).mult(f(w))
        ext_f.append(new_mono)
