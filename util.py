from ethsnarks import field

def length_expansion(l: list[field.FQ], v: int):
    if len(l) == v:
        return l
    elif len(l) < v:
        k = [field.FQ.zero()] * (v - len(l))
        return l + k
    else:
        raise IndexError

def get_challenge(seed: list[field.FQ]):
    return seed[0] # to be fixed
