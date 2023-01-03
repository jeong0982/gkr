use ethers_core::k256::elliptic_curve::PrimeField;
use ff::PrimeField;
use r1cs_file::{Constraint, FieldElement, R1csFile};
use wtns_file::*;

use crate::gkr::GKRCircuit;
use halo2curves::bn256::Fr;
use std::collections::VecDeque;

const MINUS_ONE: Fr = Fr::zero() - Fr::one();

enum Expression<T> {
    Value(T),
    Variable(u32),
}

enum NodeType<T> {
    Mult,
    Add,
    Value(Expression<T>),
}

struct IntermediateNode<T> {
    node_type: NodeType<T>,
    left: Option<Box<IntermediateNode<T>>>,
    right: Option<Box<IntermediateNode<T>>>,
}

impl<T> IntermediateNode<T> {
    fn new_from_value(value: T) -> Self {
        IntermediateNode {
            node_type: NodeType::Value(Expression::Value(value)),
            left: None,
            right: None,
        }
    }

    fn new_from_variable(var: u32) -> Self {
        IntermediateNode {
            node_type: NodeType::Value(Expression::Variable(var)),
            left: None,
            right: None,
        }
    }

    fn is_leaf(&self) -> bool {
        match self.node_type {
            NodeType::Value(_) => true,
            _ => false,
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
            node_type: NodeType::Add,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        new.push(node);
    }
    if nodes.len() % 2 == 1 {
        let merged = merge_nodes(new);
        let new_node = IntermediateNode::<FieldElement<32>> {
            node_type: NodeType::Add,
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
            node_type: NodeType::Mult,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        node_a.push(node);
    }
    for (coeff, x_i) in b {
        let left = IntermediateNode::new_from_value(coeff);
        let right = IntermediateNode::new_from_variable(x_i);
        let node = IntermediateNode::<FieldElement<32>> {
            node_type: NodeType::Mult,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        node_b.push(node);
    }
    for (coeff, x_i) in c {
        let left = IntermediateNode::new_from_value(coeff);
        let right = IntermediateNode::new_from_variable(x_i);
        let node = IntermediateNode::<FieldElement<32>> {
            node_type: NodeType::Mult,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        node_c.push(node);
    }
    let root_a = merge_nodes(node_a);
    let root_b = merge_nodes(node_b);
    let root_c = merge_nodes(node_c);

    let a_times_b = IntermediateNode {
        node_type: NodeType::Mult,
        left: Some(Box::new(root_a)),
        right: Some(Box::new(root_b)),
    };

    let minus_one = MINUS_ONE.to_repr() as FieldElement<32>;
    let minus_c = IntermediateNode {
        node_type: NodeType::Mult,
        left: Some(Box::new(root_c)),
        right: Some(Box::new(minus_one)),
    };
    IntermediateNode {
        node_type: NodeType::Add,
        left: Some(Box::new(a_times_b)),
        right: Some(Box::new(minus_c)),
    }
}

fn convert_intermediate_node_gkr(nodes: Vec<IntermediateNode<FieldElement<32>>>) -> GKRCircuit<Fr> {
    let mut layers = Vec::new();
    for node in nodes {
        let mut queue = VecDeque::new();

        queue.push_back((node, 0));

        while !queue.is_empty() {
            let nodetuple = queue.pop_front().unwrap();
            let depth = nodetuple.1;
            let node = nodetuple.0;

            if layers.len() < depth + 1 {
                let mut layer = Vec::new();
                layer.push(node);
                layers.push(layer);
            } else {
                let mut layer = layers[depth];
                layer.push(node);
            }
            if let Some(left) = &node.left {
                queue.push_back((left, depth + 1));
            }
            if let Some(right) = &node.right {
                queue.push_back((right, depth + 1));
            }
        }
    }
}

pub fn convert_r1cs_gkr(r1cs: R1csFile<32>) -> GKRCircuit<Fr> {
    todo!()
}
