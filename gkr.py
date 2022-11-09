import math
import time
from poly import *
from sumcheck import *

class Node:
  def __init__(self, binary_index: list[int], value, left=None, right=None):
    self.binary_index = binary_index
    self.value = value
    self.left = left
    self.right = right

class Layer:
    def __init__(self) -> None:
        self.nodes = []

    def def_mult(self, mult):
        self.mult = mult

    def def_add(self, add):
        self.add = add

    def get_node(self, index) -> Node:
        return self.nodes[index]

    def add_node(self, index, node) -> None:
        self.nodes.insert(index, node)

    def add_func(self, func):
        self.func = func

    def len(self):
        return len(self.nodes)

class Circuit:
    def __init__(self, depth):
        layers = []
        for _ in range(depth):
            layers.append(Layer())
        self.layers : list[Layer] = layers # type: ignore
    
    def get_node(self, layer, index):
        return self.layers[layer].get_node(index)

    def add_node(self, layer, index, binary_index, value, left=None, right=None):
        self.layers[layer].add_node(index, Node(binary_index, value, left, right))

    def depth(self):
        return len(self.layers)

    def layer_length(self, layer):
        return self.layers[layer].len()
    
    def k_i(self, layer):
        return int(math.log2(self.layer_length(layer)))

    def add_i(self, i):
        return self.layers[i].add
    
    def mult_i(self, i):
        return self.layers[i].mult
    
    def w_i(self, i):
        return self.layers[i].func


# reduce verification at two points into verification at a single point
def ell(p1: list[field.FQ], p2: list[field.FQ], t: field.FQ):
    consts = p1
    output = [field.FQ.zero()]*len(p2)
    other_term = [field.FQ.zero()]*len(p2)
    for i in range(len(p2)):
        other_term[i] = p2[i] - consts[i]
    for i in range(len(p2)):
        output[i] = consts[i] + t*other_term[i]
    return output


class Proof:
    def __init__(self, proofs, r, f, D, q, z) -> None:
      self.sumcheck_proofs : list[list[list[field.FQ]]] = proofs
      self.sumcheck_r = r
      self.f = f
      self.D = D
      self.q : list[list[field.FQ]] = q
      self.z = z

def prove(circuit: Circuit, D):
    start_time = time.time()

    z = [[]] * circuit.depth()
    z[0] = [field.FQ.zero()] * circuit.k_i(0)
    sumcheck_proofs = []
    q = []
    f_res = []
    sumcheck_r = []

    for i in range(len(z[0])):
        z[0][i] = field.FQ.random() # TODO - randomness of first value

    for i in range(circuit.depth() - 1):
        add_i_ext = get_ext(circuit.add_i(i), circuit.k_i(i) + 2 * circuit.k_i(i + 1))
        for j, r in enumerate(z[i]):
            add_i_ext = add_i_ext.eval_i(r, j + 1)

        mult_i_ext = get_ext(circuit.mult_i(i), circuit.k_i(i) + 2 * circuit.k_i(i + 1))
        for j, r in enumerate(z[i]):
            mult_i_ext = mult_i_ext.eval_i(r, j + 1)

        w_i_ext_b = get_ext_from_k(circuit.w_i(i + 1), circuit.k_i(i + 1), circuit.k_i(i) + 1)
        w_i_ext_c = get_ext_from_k(circuit.w_i(i + 1), circuit.k_i(i + 1), circuit.k_i(i) + circuit.k_i(i + 1) + 1)

        first = add_i_ext * (w_i_ext_b + w_i_ext_c)
        second = mult_i_ext * w_i_ext_b * w_i_ext_c
        f = first + second

        start_idx = circuit.k_i(i) + 1

        sumcheck_proof, r = prove_sumcheck(f, 2 * circuit.k_i(i + 1), start_idx)
        sumcheck_proofs.append(sumcheck_proof)
        sumcheck_r.append(r)

        b_star = r[0: circuit.k_i(i + 1)]
        c_star = r[circuit.k_i(i + 1):(2 * circuit.k_i(i + 1))]

        q_zero = eval_ext(circuit.w_i(i + 1), ell(b_star, c_star, field.FQ.zero()))
        q_one = eval_ext(circuit.w_i(i + 1), ell(b_star, c_star, field.FQ.one()))
        q.append([q_zero, q_one])

        f_result = polynomial(f.terms, f.constant)
        f_result_value = field.FQ.zero()
        for j, x in enumerate(r):
            if j == len(r) - 1:
                f_result_value = f_result.eval_univariate(x)
            f_result = f_result.eval_i(x, j + start_idx)
        
        f_res.append(f_result_value)

        r_star = field.FQ.random()
        next_r = ell(b_star, c_star, r_star)
        z[i + 1] = next_r # r_(i + 1)

    proof = Proof(sumcheck_proofs, sumcheck_r, f_res, D, q, z)
    print("proving time :", time.time() - start_time)
    return proof

def verify(circuit: Circuit, proof: Proof):
    m = [field.FQ.zero()]*circuit.depth()
    m[0] = eval_ext(proof.D, proof.z[0])

    for i in range(circuit.depth() - 1):
        valid = verify_sumcheck(m[i], proof.sumcheck_proofs[i], proof.sumcheck_r[i], 2 * circuit.k_i(i + 1))
        if not valid:
            return False
        else:
            b_star = proof.sumcheck_r[i][0: circuit.layer_length(i + 1) // 2]
            c_star = proof.sumcheck_r[i][circuit.layer_length(i + 1) // 2:int(circuit.layer_length(i + 1))]

            q_zero = proof.q[i][0]
            q_one = proof.q[i][1]

            modified_f = eval_ext(circuit.add_i(i), proof.z[i] + b_star + c_star) * (q_zero + q_one) \
                        + eval_ext(circuit.mult_i(i), proof.z[i] + b_star + c_star) * (q_zero * q_one)

            if proof.f[i] != modified_f:
                return False
            else:
                # should be moved to prover side
                m[i + 1] = eval_ext(circuit.w_i(i + 1), proof.z[i + 1])
    if m[circuit.depth() - 1] != eval_ext(circuit.w_i(circuit.depth() - 1), proof.z[circuit.depth() - 1]):
        return False
    return True
