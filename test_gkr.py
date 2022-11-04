from gkr import *

c = Circuit(3)
a1 = Node([0],36)
a2 = Node([1],6)
b1 = Node([0,0], 9)
b2 = Node([0,1], 4)
b3 = Node([1,0], 6)
b4 = Node([1,1], 1)
c1 = Node([0,0],3)
c2 = Node([0,1], 2)
c3 = Node([1,0],3)
c4 = Node([1,1],1)


# W0
c.add_node(0, 0, [0], 36, left=b1, right=b2)
c.add_node(0, 1, [1], 6, left=b3, right=b4)

def W0func(arr):
  if(arr == [0]):
    return 25
  elif (arr == [1]):
    return 160

c.layers[0].add_func(W0func)

def multlayerzero(arr):
  if arr == [0,0,0,0,1]:
    return 1
  elif arr == [1,1,0,1,1]:
    return 1
  else:
    return 0

def addlayerzero(_):
  return 0

c.layers[0].mult = multlayerzero
c.layers[0].add = addlayerzero


# W1
c.add_node(1, 0, [0,0], 9, left=c1, right=c1)
c.add_node(1, 1, [0,1], 4, left=c2, right=c2)
c.add_node(1, 2, [1,0], 6, left=c2, right=c3)
c.add_node(1, 3, [1,1], 1, left=c4, right=c4)

def W1Func(bitstring):
  if bitstring == [0,0]:
    return 1
  elif bitstring == [0,1]:
    return 25
  elif bitstring == [1,0]:
    return 40
  elif bitstring == [1,1]:
    return 4
c.layers[1].add_func(W1Func)

def multlayerone(arr):
  if arr == [0,0,0,0,0,0]:
    return 1
  elif arr == [0,1,0,1,0,1]:
    return 1
  elif arr == [1,0,0,1,1,0]:
    return 1
  elif arr == [1,1,1,1,1,1]:
    return 1
  else:
    return 0

def addlayerone(arr):
  return 0

c.layers[1].mult = multlayerone
c.layers[1].add = addlayerone

# W2
c.add_node(2, 0, [0,0], 3)
c.add_node(2, 1, [0,1], 2)
c.add_node(2, 2, [1,0], 3)
c.add_node(2, 3, [1,1], 1)

def W2func(bitstring):
  if bitstring == [0,0]:
    return 1
  elif bitstring == [0,1]:
    return 5
  elif bitstring == [1,0]:
    return 8
  elif bitstring == [1,1]:
    return 2
c.layers[2].add_func(W2func)

def multlayertwo(_):
  return 0
def addlayertwo(_):
  return 0

def D_func(arr):
  if arr == [0]:
    return 25
  elif arr == [1]:
    return 160

proof = prove(c, D_func)
print(verify(c, proof))
