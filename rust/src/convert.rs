use ff::PrimeField;
use r1cs_file::{Constraint, FieldElement, R1csFile};

use crate::gkr::GKRCircuit;

const S: PrimeField = halo2curves::bn256::Fr;

enum Expression<T> {
    Value(T),
    Variable(u32),
}
struct IntermediateNode<T> {
    mult: bool,
    add: bool,
    value: Option<Expression<T>>,
    left: Option<IntermediateNode>,
    right: Option<IntermediateNode>,
}

impl<T> IntermediateNode<T> {
    fn new_from_value(value: T) -> Self {
        IntermediateNode {
            mult: false,
            add: false,
            value: Some(Expression::Value(value)),
            left: None,
            right: None,
        }
    }

    fn new_from_variable(var: u32) -> Self {
        IntermediateNode {
            mult: false,
            add: false,
            value: Some(Expression::Variable(var)),
            left: None,
            right: None,
        }
    }
}

fn merge_nodes(
    nodes: Vec<IntermediateNode<FieldElement<32>>>,
) -> Vec<IntermediateNode<FieldElement<32>>> {
    if nodes.len() == 1 {
        return nodes;
    }
    let res = vec![];
    let new = vec![];
    let width = nodes.len() / 2 - 1;
    for i in 0..width {
        let left = nodes[2 * i];
        let right = nodes[2 * i + 1];
        let node = IntermediateNode::<FieldElement<32>> {
            mult: false,
            add: true,
            value: None,
            left: Some(left),
            right: Some(right),
        };
        new.push(node);
    }
    if nodes.len() % 2 == 1 {
        let merged = merge_nodes(new)[0];
        let new_node = IntermediateNode::<FieldElement<32>> {
            mult: false,
            add: true,
            value: None,
            left: Some(merged),
            right: Some(nodes[nodes.len() - 1]),
        };
        vec![new_node]
    } else {
        merge_nodes(new)
    }
}

fn make_gkr_circuit(constraint: Constraint<32>) -> GKRCircuit<S> {
    let a = constraint.0;
    let b = constraint.1;
    let c = constraint.2;

    let node_a = vec![];
    let node_b = vec![];
    let node_c = vec![];

    for (coeff, x_i) in a {
        let left = IntermediateNode::new_from_value(coeff);
        let right = IntermediateNode::new_from_variable(x_i);
        let node = IntermediateNode::<FieldElement<32>> {
            mult: true,
            add: false,
            value: None,
            left: Some(left),
            right: Some(right),
        };
        node_a.push(node);
    }
    for (coeff, x_i) in b {
        let left = IntermediateNode::new_from_value(coeff);
        let right = IntermediateNode::new_from_variable(x_i);
        let node = IntermediateNode::<FieldElement<32>> {
            mult: true,
            add: false,
            value: None,
            left: Some(left),
            right: Some(right),
        };
        node_b.push(node);
    }
    for (coeff, x_i) in c {
        let left = IntermediateNode::new_from_value(coeff);
        let right = IntermediateNode::new_from_variable(x_i);
        let node = IntermediateNode::<FieldElement<32>> {
            mult: true,
            add: false,
            value: None,
            left: Some(left),
            right: Some(right),
        };
        node_c.push(node);
    }
    let root_a = merge_nodes(node_a);
    let root_b = merge_nodes(node_b);
    let root_c = merge_nodes(node_c);

    let a_times_b = IntermediateNode {
        mult: true,
        add: false,
        value: None,
        left: Some(root_a),
        right: Some(root_b),
    };
    let final = IntermediateNode {
        mult: false,
        add: true,
        value: None,
        left: Some(a_times_b),
        right: Some(root_c),
    };
}

fn combine_gkr_circuit(c1: GKRCircuit<S>, c2: GKRCircuit<S>) -> GKRCircuit<S> {}

pub fn convert_r1cs_gkr(r1cs: R1csFile<32>) -> GKRCircuit<S> {}
