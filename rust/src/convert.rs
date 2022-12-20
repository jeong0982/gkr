use ff::PrimeField;
use r1cs_file::{Constraint, FieldElement, R1csFile};

use crate::gkr::GKRCircuit;
use std::collections::{VecDeque, HashMap};
use halo2curves::bn256::Fr;

enum Expression<T> {
    Value(T),
    Variable(u32),
}
struct IntermediateNode<T> {
    mult: bool,
    add: bool,
    value: Option<Expression<T>>,
    left: Option<Box<IntermediateNode<T>>>,
    right: Option<Box<IntermediateNode<T>>>,
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
) -> IntermediateNode<FieldElement<32>> {
    if nodes.len() == 1 {
        return nodes[0];
    }

    let new = vec![];
    let width = nodes.len() / 2 - 1;
    for i in 0..width {
        let left = nodes[2 * i];
        let right = nodes[2 * i + 1];
        let node = IntermediateNode::<FieldElement<32>> {
            mult: false,
            add: true,
            value: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        new.push(node);
    }
    if nodes.len() % 2 == 1 {
        let merged = merge_nodes(new);
        let new_node = IntermediateNode::<FieldElement<32>> {
            mult: false,
            add: true,
            value: None,
            left: Some(Box::new(merged)),
            right: Some(Box::new(nodes[nodes.len() - 1])),
        };
        new_node
    } else {
        merge_nodes(new)
    }
}

fn make_node_from_constraint(constraint: Constraint<32>) -> IntermediateNode<FieldElement<32>> {
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
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
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
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
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
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
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
        left: Some(Box::new(root_a)),
        right: Some(Box::new(root_b)),
    };
    IntermediateNode {
        mult: false,
        add: true,
        value: None,
        left: Some(Box::new(a_times_b)),
        right: Some(Box::new(root_c)),
    }
}

// fn combine_gkr_circuit(c1: GKRCircuit<S>, c2: GKRCircuit<S>) -> GKRCircuit<S> {}

fn convert_intermediate_node_gkr(nodes: Vec<IntermediateNode<FieldElement<32>>>) -> GKRCircuit<Fr> {
    let mut layer_map = HashMap::new();
    for node in nodes {
        let mut queue = VecDeque::new();

        queue.push_back((node, 0));
        let mut layer = layer_map.get_mut(0);
        while !queue.is_empty() {
            let nodetuple = queue.pop_front().unwrap();
            let depth = nodetuple.1;
            let node = nodetuple.0;

            if 
            if let Some(left) = &node.left {
                queue.push_back((left, depth + 1));
            }
            if let Some(right) = &node.right {
                queue.push_back((right, depth + 1));
            }
        }
    }
}
// pub fn convert_r1cs_gkr(r1cs: R1csFile<32>) -> GKRCircuit<S> {}
