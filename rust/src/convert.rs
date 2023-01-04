use r1cs_file::{Constraint, FieldElement, R1csFile};
use wtns_file::*;

use crate::gkr::GKRCircuit;
use halo2curves::bn256::Fr;
use halo2curves::group::ff::PrimeField;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum Expression<T> {
    Value(T),
    Variable(u32),
}

#[derive(Clone, Copy)]
enum NodeType<T> {
    Mult,
    Add,
    Value(Expression<T>),
}

#[derive(Clone)]
struct IntermediateNode<T> {
    node_type: NodeType<T>,
    left: Option<Box<IntermediateNode<T>>>,
    right: Option<Box<IntermediateNode<T>>>,
}

impl<T: Copy> IntermediateNode<T> {
    fn copy(&self) -> Self {
        IntermediateNode {
            node_type: self.node_type,
            left: self.left.as_ref().map(|node| Box::new(node.copy())),
            right: self.right.as_ref().map(|node| Box::new(node.copy())),
        }
    }
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

    fn depth(&self) -> usize {
        let left_depth = self.left.as_ref().map(|node| node.depth()).unwrap_or(0);
        let right_depth = self.right.as_ref().map(|node| node.depth()).unwrap_or(0);
        std::cmp::max(left_depth, right_depth) + 1
    }
}

fn zero_node() -> IntermediateNode<FieldElement<32>> {
    let zero = FieldElement::from((Fr::zero()).to_repr());
    IntermediateNode {
        node_type: NodeType::Value(Expression::Value(zero)),
        left: None,
        right: None,
    }
}

struct IntermediateLayer<T> {
    node_types: Vec<NodeType<T>>,
    operand_index: Vec<(usize, usize)>,
}

fn merge_nodes(
    nodes: Vec<IntermediateNode<FieldElement<32>>>,
) -> IntermediateNode<FieldElement<32>> {
    if nodes.len() == 1 {
        return nodes[0].clone();
    }

    let mut new = vec![];
    let width = nodes.len() / 2 - 1;
    for i in 0..width {
        let left = nodes[2 * i].clone();
        let right = nodes[2 * i + 1].clone();
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
            right: Some(Box::new(nodes[nodes.len() - 1].clone())),
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

    let mut node_a = vec![];
    let mut node_b = vec![];
    let mut node_c = vec![];

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
    if node_a.len() != 0 && node_b.len() != 0 {
        let root_a = merge_nodes(node_a);
        let root_b = merge_nodes(node_b);
        let root_c = merge_nodes(node_c);

        let a_times_b = IntermediateNode {
            node_type: NodeType::Mult,
            left: Some(Box::new(root_a)),
            right: Some(Box::new(root_b)),
        };

        let minus_one_val =
            Expression::Value(FieldElement::from((Fr::zero() - Fr::one()).to_repr()));
        let minus_one = IntermediateNode::<FieldElement<32>> {
            node_type: NodeType::Value(minus_one_val),
            left: None,
            right: None,
        };
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
    } else {
        // [] * [] - C = 0
        merge_nodes(node_c)
    }
}

fn get_k(n: usize) -> usize {
    let mut k = 0;
    let mut m = n;
    while m > 0 {
        m >>= 1;
        k += 1;
    }
    k
}

fn compile(
    nodes: Vec<IntermediateNode<FieldElement<32>>>,
) -> Vec<IntermediateLayer<FieldElement<32>>> {
    let mut layers = vec![];

    let zero = FieldElement::from((Fr::zero()).to_repr());

    let height = nodes.iter().map(|node| node.depth()).max().unwrap_or(0);
    if height == 0 {
        return layers;
    }

    let mut used: HashMap<Expression<FieldElement<32>>, usize> = HashMap::new();
    let mut current_nodes = nodes.clone();
    let mut next_nodes = vec![];
    let mut zero_index = None;
    for d in 0..height - 1 {
        let mut layer_operand_idx = vec![];
        let mut node_types = vec![];

        let k = get_k(current_nodes.len());
        let full_num = 2 << k;
        let diff = full_num - k;
        let added_zero_idx = current_nodes.len();
        for _ in 0..diff {
            current_nodes.push(zero_node());
        }
        for node in current_nodes.iter() {
            match node.node_type {
                NodeType::Mult | NodeType::Add => {
                    node_types.push(node.node_type);
                    let operand_index = (next_nodes.len(), next_nodes.len() + 1);
                    layer_operand_idx.push(operand_index);
                    node.left
                        .as_ref()
                        .map(|node| next_nodes.push(*(node.clone())));
                    node.right
                        .as_ref()
                        .map(|node| next_nodes.push(*(node.clone())));
                }
                NodeType::Value(e) => {
                    if used.contains_key(&e) {
                        node_types.push(NodeType::Add);
                        let operand_index = (used.get(&e).unwrap().clone(), zero_index.unwrap());
                        layer_operand_idx.push(operand_index);
                    } else {
                        if zero_index == None {
                            zero_index = Some(next_nodes.len());
                            next_nodes.push(zero_node());
                        }
                        match e {
                            Expression::Value(v) => {
                                node_types.push(NodeType::Add);
                                if v == zero {
                                    used.insert(e, zero_index.unwrap());
                                    let operand_index = (zero_index.unwrap(), zero_index.unwrap());
                                    layer_operand_idx.push(operand_index);
                                } else {
                                    used.insert(e, next_nodes.len());
                                    let operand_index = (next_nodes.len(), zero_index.unwrap());
                                    next_nodes.push(IntermediateNode::new_from_value(v));
                                    layer_operand_idx.push(operand_index);
                                }
                            }
                            Expression::Variable(var) => {
                                node_types.push(NodeType::Add);
                                used.insert(e, next_nodes.len());
                                let operand_index = (next_nodes.len(), zero_index.unwrap());
                                next_nodes.push(IntermediateNode::new_from_variable(var));
                                layer_operand_idx.push(operand_index);
                            }
                        }
                    }
                }
            }
        }
        layers.push(IntermediateLayer {
            node_types,
            operand_index: layer_operand_idx,
        });
        zero_index = None;
        current_nodes = next_nodes;
        next_nodes = vec![];
        used = HashMap::new();
    }
    layers
}

fn convert_constraints_to_nodes(r1cs: R1csFile<32>) -> Vec<IntermediateNode<FieldElement<32>>> {
    let constraints = r1cs.constraints;
    let mut nodes = vec![];
    for constraint in constraints.0 {
        nodes.push(make_node_from_constraint(constraint));
    }
    nodes
}

pub fn convert_r1cs_gkr(r1cs: R1csFile<32>) -> () {
    let layers = compile(convert_constraints_to_nodes(r1cs));
}
