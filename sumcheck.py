from ethsnarks import field
from poly import *
from util import *
from typing import Callable

# TODO
# FS transform
# separate prover and verifier
def prove_sumcheck(c, g, v: int):
    proof = [[field.FQ.zero()]] * v
    
    return proof

def verify_sumcheck(claims, p: list[list[field.FQ]], g, v: int):
    bn = len(p)
    if(v == 1 and (g([0]) + g([1])) == claims[0]):
        return True, []
    expected = claims[0]
    for i in range(bn):
        q_zero = eval_univariate(p[i], field.FQ.zero())
        q_one = eval_univariate(p[i], field.FQ.one())

        if q_zero + q_one != expected:
            return False

        r = get_challenge(p[i])
        expected = eval_univariate(p[i], r)

    return True

def sumcheck(c: field.FQ, g: Callable[[list[field.FQ]], field.FQ], v):

    if(v == 1 and (g([field.FQ.zero()]) + g([field.FQ.one()])) == c):
        return True, []

    g_vector = [field.FQ(0)] * v
    r = [field.FQ(0)] * v

    # first round
    # g1(X1)=∑(x2,⋯,xv)∈{0,1}^v g(X_1,x_2,⋯,x_v)
    def g_1(x_1):
        assignment = generate_binary(v - 1)
        for i in range(len(assignment)):
            assignment[i].insert(0, x_1)

        output = field.FQ(0)

        for i in range(2 ** (v - 1)):
            output += g(assignment[i])
        return output

    if (g_1(field.FQ(0)) + g_1(field.FQ(1))) != c:
        return False, []
    else:
        r[0] = field.FQ.random()
        g_vector[0] = g_1(r[0])

    for j in range(1, v - 1): # 1 < j < v round
        def g_j(x: field.FQ):
            assignment = generate_binary(v - j - 1)
            for i in range(len(assignment)):
                assignment[i] = r[0 : j] + [x] + assignment[i]

            output = field.FQ(0)
            for i in range(len(assignment)):
                output += g(assignment[i]) 
            return output

        if g_vector[j - 1] != (g_j(field.FQ.zero()) + g_j(field.FQ.one())):
            return False, []
        else:
            r[j] = field.FQ.random()
            g_vector[j] = g_j(r[j])

    def g_v(x_v):
        eval_vector = r
        eval_vector[v - 1] = x_v
        return g(eval_vector)

    if (g_v(0) + g_v(1)) != g_vector[v - 2]:
        return False, []
    else:
        r[v - 1] = field.FQ.random()
        g_vector[v - 1] = g_v(r[v - 1])
        
        if (g(r) != g_vector[v - 1]):
            return False, []
        return True, r
