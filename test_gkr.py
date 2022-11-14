from gkr import *
zero = field.FQ.zero()
one = field.FQ.one()

c = Circuit(3)
a1 = Node([0], field.FQ(36))
a2 = Node([1], field.FQ(6))
b1 = Node([0,0], field.FQ(9))
b2 = Node([0,1], field.FQ(4))
b3 = Node([1,0], field.FQ(6))
b4 = Node([1,1], field.FQ(1))
c1 = Node([0,0], field.FQ(3))
c2 = Node([0,1], field.FQ(2))
c3 = Node([1,0], field.FQ(3))
c4 = Node([1,1], field.FQ(1))


# W0
c.add_node(0, 0, [0], 36, left=b1, right=b2)
c.add_node(0, 1, [1], 6, left=b3, right=b4)

def W0func(arr):
  if(arr == [field.FQ(0)]):
    return field.FQ(36)
  elif (arr == [field.FQ(1)]):
    return field.FQ(6)

c.layers[0].add_func(W0func)

def multlayerzero(arr):
    zero = field.FQ.zero()
    one = field.FQ.one()
    if arr == [zero, zero, zero, zero, one]:
        return one
    elif arr == [one, one, zero, one, one]:
        return one
    else:
        return zero

def addlayerzero(_):
  return zero

c.layers[0].mult = multlayerzero
c.layers[0].add = addlayerzero


# W1
c.add_node(1, 0, [0,0], 9, left=c1, right=c1)
c.add_node(1, 1, [0,1], 4, left=c2, right=c2)
c.add_node(1, 2, [1,0], 6, left=c2, right=c3)
c.add_node(1, 3, [1,1], 1, left=c4, right=c4)

def W1Func(bitstring):
  if bitstring == [zero, zero]:
    return field.FQ(9)
  elif bitstring == [zero, one]:
    return field.FQ(4)
  elif bitstring == [one, zero]:
    return field.FQ(6)
  elif bitstring == [one, one]:
    return field.FQ(1)

c.layers[1].add_func(W1Func)

def multlayerone(arr):
  if arr == [zero, zero, zero, zero, zero, zero]:
    return one
  elif arr == [zero,one,zero,one,zero,one]:
    return one
  elif arr == [one, zero, zero, one, one, zero]:
    return one
  elif arr == [one, one, one, one, one, one]:
    return one
  else:
    return zero

def addlayerone(arr):
  return zero

c.layers[1].mult = multlayerone
c.layers[1].add = addlayerone

# W2
c.add_node(2, 0, [0,0], 3)
c.add_node(2, 1, [0,1], 2)
c.add_node(2, 2, [1,0], 3)
c.add_node(2, 3, [1,1], 1)

def W2func(bitstring):
  if bitstring == [zero,zero]:
    return field.FQ(3)
  elif bitstring == [zero, one]:
    return field.FQ(2)
  elif bitstring == [one, zero]:
    return field.FQ(3)
  elif bitstring == [one, one]:
    return field.FQ(1)

c.layers[2].add_func(W2func)

def multlayertwo(_):
  return zero
def addlayertwo(_):
  return zero

def D_func(arr):
  if arr == [zero]:
    return field.FQ(36)
  elif arr == [one]:
    return field.FQ(6)

proof = prove(c, D_func)
print(verify(proof))
