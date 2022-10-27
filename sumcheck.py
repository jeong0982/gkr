from ethsnarks import field

def generate_binary_strings(bit_count):
    binary_strings = []

    def genbin(n, bs=""):
        if len(bs) == n:
            binary_strings.append(bs)
        else:
            genbin(n, bs + "0")
            genbin(n, bs + "1")

    genbin(bit_count)
    return binary_strings

def Convert(string):
    list1 = []
    list1[:0] = string
    return list1

# TODO
# FS transform
# separate prover and verifier
def sumcheck(c: field.FQ, g, v):

    if(v == 1 and (g([0]) + g([1])) == c):
        return True

    g_vector = [field.FQ(0)] * v
    r = [field.FQ(0)] * v

    # first round
    # g1(X1)=∑(x2,⋯,xv)∈{0,1}^v g(X_1,x_2,⋯,x_v)
    def g_1(x_1):
        assignment = generate_binary_strings(v - 1)
        for i in range(len(assignment)):
            assignment[i] = Convert(assignment[i])
            for j in range(len(assignment[i])):
                assignment[i][j] = field.FQ(int(assignment[i][j]))

        for i in range(len(assignment)):
            assignment[i].insert(0, x_1)

        output = field.FQ(0)

        for i in range(2 ** (v - 1)):
            output += g(assignment[i])
        return output

    if (g_1(field.FQ(0)) + g_1(field.FQ(1))) != c:
        return False
    else:
        r[0] = field.FQ.random()
        g_vector[0] = g_1(r[0])

    for j in range(1, v - 1): # 1 < j < v round
        def g_j(x):
            assignment = generate_binary_strings(v - j - 1)
            for i in range(len(assignment)):
                assignment[i] = Convert(assignment[i])
                for k in range(len(assignment[i])):
                    assignment[i][k] = field.FQ(int(assignment[i][k]))

            for i in range(len(assignment)):
                assignment[i] = r[0 : j] + [x] + assignment[i]

            output = field.FQ(0)
            for i in range(len(assignment)):
                output += g(assignment[i]) 
            return output

        if g_vector[j - 1] != (g_j(0) + g_j(1)):
            return False
        else:
            r[j] = field.FQ.random()
            g_vector[j] = g_j(r[j])

    def g_v(x_v):
        eval_vector = r
        eval_vector[v - 1] = x_v
        return g(eval_vector)

    if (g_v(0) + g_v(1)) != g_vector[v - 2]:
        return False
    else:
        r[v - 1] = field.FQ.random()
        g_vector[v - 1] = g_v(r[v - 1])
        
        if (g(r) != g_vector[v - 1]):
            return False
        return True
