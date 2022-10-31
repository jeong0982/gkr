from ethsnarks import field

def eval_univariate(coeffs: list[field.FQ], x: field.FQ):
    result = coeffs[len(coeffs) - 1]
    for i in range(len(coeffs) - 2, 0, -1):
        result *= x
        result += coeffs[i]
    return result
