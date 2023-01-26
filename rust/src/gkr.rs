pub mod poly;
pub mod prover;
pub mod sumcheck;

use ff::PrimeField;

#[derive(Clone, Debug)]
pub struct Proof<S: PrimeField> {
    pub sumcheck_proofs: Vec<Vec<Vec<S>>>,
    pub sumcheck_r: Vec<Vec<S>>,
    pub f: Vec<S>,
    pub d: Vec<Vec<S>>,
    pub q: Vec<Vec<S>>,
    pub z: Vec<Vec<S>>,
    pub r: Vec<S>,

    pub depth: usize,
    pub input_func: Vec<Vec<S>>,
    pub add: Vec<Vec<Vec<S>>>,
    pub mult: Vec<Vec<Vec<S>>>,
    pub k: Vec<usize>,
}

pub struct Input<S: PrimeField> {
    // w[i] is function that gets index and returns value of each gate.
    // polynomial form
    pub w: Vec<Vec<Vec<S>>>,
    // d is output of circuit
    pub d: Vec<Vec<S>>,
}

impl<S: PrimeField> Input<S> {
    pub fn w(&self, i: usize) -> Vec<Vec<S>> {
        self.w[i].clone()
    }
}

pub struct Layer<S: PrimeField> {
    pub k: usize,
    pub add: Vec<Vec<S>>,
    pub mult: Vec<Vec<S>>,
    pub wire: (Vec<Vec<S>>, Vec<Vec<S>>),
}

impl<S: PrimeField> Layer<S> {
    pub fn new(
        k: usize,
        add: Vec<Vec<S>>,
        mult: Vec<Vec<S>>,
        wire: (Vec<Vec<S>>, Vec<Vec<S>>),
    ) -> Self {
        Layer { k, add, mult, wire }
    }
}

pub struct GKRCircuit<S: PrimeField> {
    pub layer: Vec<Layer<S>>,
    input_k: usize,
}

impl<S: PrimeField> GKRCircuit<S> {
    pub fn new(layer: Vec<Layer<S>>, input_k: usize) -> Self {
        GKRCircuit { layer, input_k }
    }

    pub fn depth(&self) -> usize {
        self.layer.len()
    }

    pub fn add(&self, i: usize) -> Vec<Vec<S>> {
        self.layer[i].add.clone()
    }

    pub fn add_wire(&self, i: usize) -> Vec<Vec<S>> {
        self.layer[i].wire.0.clone()
    }

    pub fn mult(&self, i: usize) -> Vec<Vec<S>> {
        self.layer[i].mult.clone()
    }

    pub fn mult_wire(&self, i: usize) -> Vec<Vec<S>> {
        self.layer[i].wire.1.clone()
    }

    pub fn k(&self, i: usize) -> usize {
        if i == self.layer.len() {
            return self.input_k;
        }
        self.layer[i].k
    }

    pub fn get_k_list(&self) -> Vec<usize> {
        let mut ks = vec![];
        for i in 0..self.depth() {
            ks.push(self.k(i));
        }
        ks.push(self.input_k);
        ks
    }

    pub fn get_add_list(&self) -> Vec<Vec<Vec<S>>> {
        let mut adds = vec![];
        for i in 0..self.depth() {
            adds.push(self.add(i));
        }
        adds
    }

    pub fn get_mult_list(&self) -> Vec<Vec<Vec<S>>> {
        let mut mults = vec![];
        for i in 0..self.depth() {
            mults.push(self.mult(i));
        }
        mults
    }
}
