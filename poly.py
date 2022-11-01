from ethsnarks import field

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

def eval_ext(f, r: list[field.FQ]):
    w = generate_binary(len(r))
    acc = field.FQ.zero()
    for w_i in w:
        acc += f(w_i) * chi(w_i, r)
    return acc
