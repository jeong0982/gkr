from poly import *
from util import *
from typing import Callable

# TODO
# FS transform
# separate prover and verifier
def prove_sumcheck(g: polynomial, v: int):
    proof = []
    r = []
    # first round
    # g1(X1)=∑(x2,⋯,xv)∈{0,1}^v g(X_1,x_2,⋯,x_v)    
    g_1 = polynomial([])
    assignments = generate_binary(v - 1)
    for assignment in assignments:
        g_1_sub = polynomial(g.terms[:])
        for i, x_i in enumerate(assignment):
            idx = i + 2
            g_1_sub.eval_i(x_i, idx)
        g_1 += g_1_sub
    proof.append(g_1.get_all_coefficients())
    
    r.append(field.FQ.random()) # TODO FS

    # 1 < j < v round
    for j in range(1, v - 1):
        g_j = polynomial(g.terms[:])
        assignments = generate_binary(v - j - 1)
        for i, r_i in enumerate(r):
            idx = i + 1
            g_j = g_j.eval_i(r_i, idx)
        for assignment in assignments:
            g_j_sub = polynomial(g_j.terms[:])
            for x_i in assignment:
                idx = j + 1
                g_j_sub.eval_i(x_i, idx)
            g_j += g_j_sub
        proof.append(g_j.get_all_coefficients())

        r.append(field.FQ.random())

    g_v = polynomial(g.terms[:])
    for i, r_i in enumerate(r):
        idx = i + 1
        g_v = g_v.eval_i(r_i, idx)
    proof.append(g_v.get_all_coefficients())

    return proof, r

def verify_sumcheck(claim: field.FQ, proof: list[list[field.FQ]], r, v: int):
    bn = len(proof)
    if(v == 1 and (eval_univariate(proof[0], field.FQ.zero()) + eval_univariate(proof[0], field.FQ.one())) == claim):
        return True
    expected = claim
    for i in range(bn):
        q_zero = eval_univariate(proof[i], field.FQ.zero())
        q_one = eval_univariate(proof[i], field.FQ.one())

        if q_zero + q_one != expected:
            return False

        expected = eval_univariate(proof[i], r[i])

    return True
