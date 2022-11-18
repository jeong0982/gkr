from gkr import *
import json
import functools

# Generate input.json to run circom circuit
def generate_json(proof: Proof):
    file_path = "./input.json"
    modified_proof = modify_proof_for_circom(proof)
    dict_proof = modified_proof.to_dict()
    with open(file_path, 'w') as out:
        json.dump(dict_proof, out, sort_keys=True, indent=4)

def modify_proof_for_circom(proof: Proof):
    def max_terms(x):
        return functools.reduce(max, x, 0)
    
    def pad_with_zeros(x, target_length, front=True):
        assert(len(x) <= target_length)
        zeros = [field.FQ.zero()] * (target_length - len(x))
        if front:
            return zeros + x
        else:
            return x + zeros

    max_sp_terms = max_terms(list(map(lambda x: max_terms(list(map(lambda y: len(y), x))), proof.sumcheck_proofs)))
    largest_2k = max_terms(list(map(lambda x: len(x), proof.sumcheck_proofs)))
    new_sumcheck_proofs = []
    for p in proof.sumcheck_proofs:
        new_p = []
        for eq in p:
            new_p.append(pad_with_zeros(eq, max_sp_terms))
        if len(new_p) < largest_2k:
            pad = [pad_with_zeros([], max_sp_terms)] * (largest_2k - len(new_p))
            new_p += pad
        new_sumcheck_proofs.append(new_p)
    
    new_sumcheck_r = []
    for r in proof.sumcheck_r:
        new_sumcheck_r.append(pad_with_zeros(r, largest_2k, False))
    
    max_q_terms = max_terms(list(map(lambda x: len(x), proof.q)))
    new_q = []
    for q in proof.q:
        new_q.append(pad_with_zeros(q, max_q_terms))
    
    new_z = []
    for z in proof.z:
        new_z.append(pad_with_zeros(z, largest_2k // 2, False))
    
    new_add = []
    max_add_i = max_terms(list(map(lambda x: len(x), proof.add)))
    for add in proof.add:
        new_add_i = []
        for poly in add:
            new_add_i.append(pad_with_zeros(poly, largest_2k // 2 + largest_2k + 1, False))
        if len(new_add_i) < max_add_i:
            pad = [pad_with_zeros([], largest_2k // 2 + largest_2k + 1)] * (max_add_i - len(new_add_i))
            new_add_i += pad
        new_add.append(new_add_i)

    new_mult = []
    max_mult_i = max_terms(list(map(lambda x: len(x), proof.mult)))
    for mult in proof.mult:
        new_mult_i = []
        for poly in mult:
            new_mult_i.append(pad_with_zeros(poly, largest_2k // 2 + largest_2k + 1, False))
        if len(new_mult_i) < max_mult_i:
            pad = [pad_with_zeros([], largest_2k // 2 + largest_2k + 1)] * (max_mult_i - len(new_mult_i))
            new_mult_i += pad
        new_mult.append(new_mult_i)
    
    return Proof(new_sumcheck_proofs, new_sumcheck_r, proof.f, proof.D, new_q, new_z, proof.r, proof.d, proof.input_func, new_add, new_mult, proof.k)
    