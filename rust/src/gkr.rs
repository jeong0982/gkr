pub mod prover;

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

pub struct GKRNode<S: PrimeField> {
    pub binary_index: Vec<usize>,
    pub value: S,
}

pub struct Layer<S: PrimeField> {
    pub k: usize,
    pub nodes: Vec<GKRNode<S>>,
    pub add: Vec<Vec<S>>,
    pub mult: Vec<Vec<S>>,
    pub w: Vec<Vec<S>>,
}

pub struct GKRCircuit<S: PrimeField> {
    pub layer: Vec<Layer<S>>,
}
