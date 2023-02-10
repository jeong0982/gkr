use r1cs_file::{Constraint, FieldElement, R1csFile};
use wtns_file::*;

use crate::gkr::{poly::*, GKRCircuit, Input, Layer};
use halo2curves::bn256::Fr;
use halo2curves::group::ff::PrimeField;
use rayon::prelude::*;
use std::{collections::HashMap, fmt::Debug, fs::File, io::Read, ops::Deref};

const DEPTH_LIMIT: usize = 10;
const WIDTH_LIMIT: usize = 20;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Expression<T> {
    Value(T),
    Variable(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<T: PartialEq + Debug> PartialEq for IntermediateNode<T> {
    fn eq(&self, other: &IntermediateNode<T>) -> bool {
        let mut property = self.node_type == other.node_type;
        if !((self.left.is_some() == other.left.is_some())
            && (self.right.is_some() == self.right.is_some()))
        {
            return false;
        }
        if self.left.is_some() && other.left.is_some() {
            property &= self.left.as_ref().unwrap().deref().eq(other
                .left
                .as_ref()
                .unwrap()
                .deref());
        }
        if self.right.is_some() && other.right.is_some() {
            property &= self.right.as_ref().unwrap().deref().eq(other
                .right
                .as_ref()
                .unwrap()
                .deref());
        }
        return property;
    }
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

#[derive(Clone)]
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
    let width = nodes.len() / 2;
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

fn get_k(n: usize) -> usize {
    let mut k = 0;
    let mut m = n;
    while m > 1 {
        m >>= 1;
        k += 1;
    }
    if n & (n - 1) == 0 {
        k
    } else {
        k + 1
    }
}

fn compile(
    nodes: Vec<Vec<IntermediateNode<FieldElement<32>>>>,
) -> (
    Vec<Vec<IntermediateLayer<FieldElement<32>>>>,
    Vec<Vec<NodeType<FieldElement<32>>>>,
) {
    println!("Compile nodes..");
    let mut total = vec![];
    let mut total_inputs = vec![];

    let mut nodes_sorted = nodes.clone();
    nodes_sorted.sort_by(|a, b| {
        let a_height = a.iter().map(|node| node.depth()).max().unwrap_or(0);
        let b_height = b.iter().map(|node| node.depth()).max().unwrap_or(0);
        a_height.cmp(&b_height)
    });

    let mut width = nodes_sorted.len();
    while width > WIDTH_LIMIT {
        let mut new_nodes = vec![];
        let new_width = width / 2;
        for i in 0..new_width {
            let mut first = nodes_sorted[2 * i].clone();
            let second = nodes_sorted[2 * i + 1].clone();
            first.extend(second);
            new_nodes.push(first);
        }
        if width % 2 == 1 {
            new_nodes.push(nodes_sorted[width - 1].clone());
        }
        nodes_sorted = new_nodes;
        width = nodes_sorted.len();
    }
    for one_circuit in nodes_sorted.iter() {
        let mut layers = vec![];

        let zero = FieldElement::from((Fr::zero()).to_repr());

        let height = one_circuit
            .iter()
            .map(|node| node.depth())
            .max()
            .unwrap_or(0);
        if height == 0 {
            return (vec![layers], vec![]);
        }
        let mut inputs = vec![];

        let mut used: HashMap<Expression<FieldElement<32>>, usize> = HashMap::new();
        let mut current_nodes = one_circuit.clone();
        let mut next_nodes = vec![];
        let mut zero_index = None;
        for d in 0..(height + 1) {
            let mut layer_operand_idx = vec![];
            let mut node_types = vec![];
            let k = get_k(current_nodes.len());
            let full_num = 1 << k;
            let diff = full_num - current_nodes.len();
            for _ in 0..diff {
                current_nodes.push(zero_node());
            }
            if d == height {
                inputs = current_nodes
                    .iter()
                    .map(|node| node.node_type.clone())
                    .collect();
                break;
            }
            if d == height - 1 {
                for node in current_nodes.iter() {
                    match node.node_type {
                        NodeType::Mult | NodeType::Add => {
                            panic!("Unsupported");
                        }
                        NodeType::Value(e) => {
                            if used.contains_key(&e) {
                                node_types.push(NodeType::Add);
                                let operand_index =
                                    (used.get(&e).unwrap().clone(), zero_index.unwrap());
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
                                            let operand_index =
                                                (zero_index.unwrap(), zero_index.unwrap());
                                            layer_operand_idx.push(operand_index);
                                        } else {
                                            used.insert(e, next_nodes.len());
                                            let operand_index =
                                                (next_nodes.len(), zero_index.unwrap());
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
                continue;
            }

            for node in current_nodes.iter() {
                match node.node_type {
                    NodeType::Mult | NodeType::Add => {
                        node_types.push(node.node_type);
                        let left = node.left.as_ref().unwrap().deref();
                        let right = node.right.as_ref().unwrap().deref();

                        let mut left_index = next_nodes.len();
                        let mut right_index = next_nodes.len();

                        if next_nodes.contains(left) {
                            left_index = next_nodes.iter().position(|node| node == left).unwrap();
                        } else {
                            node.left
                                .as_ref()
                                .map(|node| next_nodes.push(*(node.clone())));
                            left_index = next_nodes.len() - 1;
                        }
                        if next_nodes.contains(right) {
                            right_index = next_nodes.iter().position(|node| node == right).unwrap();
                        } else {
                            node.right
                                .as_ref()
                                .map(|node| next_nodes.push(*(node.clone())));
                            right_index = next_nodes.len() - 1;
                        }
                        let operand_index = (left_index, right_index);
                        layer_operand_idx.push(operand_index);
                    }
                    NodeType::Value(e) => {
                        if used.contains_key(&e) {
                            node_types.push(NodeType::Add);
                            let operand_index =
                                (used.get(&e).unwrap().clone(), zero_index.unwrap());
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
                                        let operand_index =
                                            (zero_index.unwrap(), zero_index.unwrap());
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
        total.push(layers);
        total_inputs.push(inputs);
    }
    (total, total_inputs)
}

fn convert_constraints_to_nodes(
    r1cs: &R1csFile<32>,
) -> Vec<Vec<IntermediateNode<FieldElement<32>>>> {
    fn count_mult(v: &Vec<(FieldElement<32>, u32)>) -> (i32, i32) {
        let one = FieldElement(Fr::one().to_repr());
        let minus_one = FieldElement::from((Fr::zero() - Fr::one()).to_repr());
        let mut a = 0;
        let mut b = 0;
        for (coeff, x_i) in v {
            if coeff.clone() == one {
                b += 1;
            } else if coeff.clone() == minus_one {
                a += 1;
            } else {
                a += 1;
                b += 1;
            }
        }
        (a, b)
    }
    fn update_symbol_table(
        symbol_table: &mut HashMap<
            u32,
            (IntermediateNode<FieldElement<32>>, usize, FieldElement<32>),
        >,
        a: &IntermediateNode<FieldElement<32>>,
        c: &Vec<(FieldElement<32>, u32)>,
        idx: usize,
        neg: &bool,
    ) -> () {
        fn make_node_except_i(
            i: usize,
            v: &Vec<(FieldElement<32>, u32)>,
            neg: &bool,
        ) -> IntermediateNode<FieldElement<32>> {
            let minus_one = FieldElement::from((Fr::zero() - Fr::one()).to_repr());
            let mut node_c = vec![];
            let one = FieldElement(Fr::one().to_repr());
            for (idx, (coeff, x_i)) in v.iter().enumerate() {
                if idx == i {
                    continue;
                }
                if neg.clone() == true {
                    if coeff.clone() == one {
                        let node = IntermediateNode::new_from_variable(x_i.clone());
                        node_c.push(node);
                    } else {
                        let left = IntermediateNode::new_from_value(coeff.clone());
                        let right = IntermediateNode::new_from_variable(x_i.clone());
                        let node = IntermediateNode::<FieldElement<32>> {
                            node_type: NodeType::Mult,
                            left: Some(Box::new(left)),
                            right: Some(Box::new(right)),
                        };
                        node_c.push(node);
                    }
                } else {
                    if coeff.clone() == minus_one {
                        let node = IntermediateNode::new_from_variable(x_i.clone());
                        node_c.push(node);
                    } else {
                        let coeff_fr = Fr::from_repr(coeff.clone().0).unwrap();
                        let new_coeff =
                            FieldElement((coeff_fr * (Fr::zero() - Fr::one())).to_repr());
                        let left = IntermediateNode::new_from_value(new_coeff.clone());
                        let right = IntermediateNode::new_from_variable(x_i.clone());
                        let node = IntermediateNode::<FieldElement<32>> {
                            node_type: NodeType::Mult,
                            left: Some(Box::new(left)),
                            right: Some(Box::new(right)),
                        };
                        node_c.push(node);
                    }
                }
            }

            merge_nodes(node_c)
        }
        if c.len() == 1 {
            if neg.clone() {
                let coeff = Fr::from_repr(c[0].0.clone().0).unwrap();
                let new_coeff = FieldElement((coeff * (Fr::zero() - Fr::one())).to_repr());
                symbol_table.insert(c[0].1.clone(), (a.clone(), idx, new_coeff));
            } else {
                symbol_table.insert(c[0].1.clone(), (a.clone(), idx, c[0].0.clone()));
            }
        } else {
            for (i, (coeff, x_i)) in c.iter().enumerate() {
                let node = make_node_except_i(i, &c, neg);
                let res = IntermediateNode {
                    node_type: NodeType::Add,
                    left: Some(Box::new(a.clone())),
                    right: Some(Box::new(node)),
                };
                symbol_table.insert(x_i.clone(), (res, idx, coeff.clone()));
            }
        }
    }
    let mut used = vec![];

    let constraints = &r1cs.constraints;
    let mut nodes = vec![];
    let mut sym_tbl: HashMap<u32, (IntermediateNode<FieldElement<32>>, usize, FieldElement<32>)> =
        HashMap::new();
    let one = FieldElement(Fr::one().to_repr());
    let minus_one = FieldElement::from((Fr::zero() - Fr::one()).to_repr());
    for (i, constraint) in constraints.0.iter().enumerate() {
        let mut neg = false;
        let mut a = &constraint.0;
        let mut b = &constraint.1;
        let mut c = &constraint.2;

        let mut node_a = vec![];
        let mut node_b = vec![];
        let mut node_c = vec![];

        let cnt_a = count_mult(a);
        let cnt_b = count_mult(b);
        let cnt_c = count_mult(c);

        let mult_cnt = cnt_a.0 + cnt_b.0 + cnt_c.1;
        let m_mult_cnt = cnt_a.1 + cnt_b.1 + cnt_c.0;

        if mult_cnt > m_mult_cnt {
            neg = true;
        }
        for (coeff, x_i) in a {
            let e = sym_tbl.get(x_i);
            if let Some(lookup_res) = e && lookup_res.0.depth() < DEPTH_LIMIT {
                let lookup_res_cloned = lookup_res.clone();
                let coeff_fr = Fr::from_repr(coeff.clone().0).unwrap();
                let new_coeff = FieldElement((coeff_fr * (Fr::zero() - Fr::one())).to_repr());
                if coeff.clone() == lookup_res_cloned.2 {
                    if neg {
                        let minus_one_node = IntermediateNode::new_from_value(minus_one.clone());
                        let new_node = IntermediateNode::<FieldElement<32>> {
                            node_type: NodeType::Mult,
                            left: Some(Box::new(lookup_res_cloned.0)),
                            right: Some(Box::new(minus_one_node)),
                        };
                        node_a.push(new_node);
                    } else {
                        node_a.push(lookup_res_cloned.0);
                    }
                    used.push(lookup_res_cloned.1);
                    continue;
                } else if new_coeff == lookup_res_cloned.2 && neg {
                    node_a.push(lookup_res_cloned.0);
                    used.push(lookup_res_cloned.1);
                    continue;
                }
            }
            if neg == true {
                if coeff.clone() == minus_one {
                    let node = IntermediateNode::new_from_variable(x_i.clone());
                    node_a.push(node);
                } else {
                    let coeff_fr = Fr::from_repr(coeff.clone().0).unwrap();
                    let new_coeff = FieldElement((coeff_fr * (Fr::zero() - Fr::one())).to_repr());
                    let left = IntermediateNode::new_from_value(new_coeff.clone());
                    let right = IntermediateNode::new_from_variable(x_i.clone());
                    let node = IntermediateNode::<FieldElement<32>> {
                        node_type: NodeType::Mult,
                        left: Some(Box::new(left)),
                        right: Some(Box::new(right)),
                    };
                    node_a.push(node);
                }
            } else {
                if coeff.clone() == one {
                    let node = IntermediateNode::new_from_variable(x_i.clone());
                    node_a.push(node);
                } else {
                    let left = IntermediateNode::new_from_value(coeff.clone());
                    let right = IntermediateNode::new_from_variable(x_i.clone());
                    let node = IntermediateNode::<FieldElement<32>> {
                        node_type: NodeType::Mult,
                        left: Some(Box::new(left)),
                        right: Some(Box::new(right)),
                    };
                    node_a.push(node);
                }
            }
        }
        for (coeff, x_i) in b {
            let e = sym_tbl.get(x_i);
            if let Some(lookup_res) = e && lookup_res.0.depth() < DEPTH_LIMIT {
                let lookup_res_cloned = lookup_res.clone();
                if coeff.clone() == lookup_res_cloned.2 {
                    node_b.push(lookup_res_cloned.0);
                    used.push(lookup_res_cloned.1);
                    continue;
                }
            }
            if coeff.clone() == one {
                let node = IntermediateNode::new_from_variable(x_i.clone());
                node_b.push(node);
            } else {
                let left = IntermediateNode::new_from_value(coeff.clone());
                let right = IntermediateNode::new_from_variable(x_i.clone());
                let node = IntermediateNode::<FieldElement<32>> {
                    node_type: NodeType::Mult,
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                };
                node_b.push(node);
            }
        }
        if node_a.len() != 0 && node_b.len() != 0 {
            let root_a = merge_nodes(node_a);
            let root_b = merge_nodes(node_b);
            let a_times_b = IntermediateNode {
                node_type: NodeType::Mult,
                left: Some(Box::new(root_a)),
                right: Some(Box::new(root_b)),
            };
            // update_symbol_table(&mut sym_tbl, &a_times_b, c, i, &neg);

            for (coeff, x_i) in c {
                if neg == true {
                    if coeff.clone() == one {
                        let node = IntermediateNode::new_from_variable(x_i.clone());
                        node_c.push(node);
                    } else {
                        let left = IntermediateNode::new_from_value(coeff.clone());
                        let right = IntermediateNode::new_from_variable(x_i.clone());
                        let node = IntermediateNode::<FieldElement<32>> {
                            node_type: NodeType::Mult,
                            left: Some(Box::new(left)),
                            right: Some(Box::new(right)),
                        };
                        node_c.push(node);
                    }
                } else {
                    if coeff.clone() == minus_one {
                        let node = IntermediateNode::new_from_variable(x_i.clone());
                        node_c.push(node);
                    } else {
                        let coeff_fr = Fr::from_repr(coeff.clone().0).unwrap();
                        let new_coeff =
                            FieldElement((coeff_fr * (Fr::zero() - Fr::one())).to_repr());
                        let left = IntermediateNode::new_from_value(new_coeff.clone());
                        let right = IntermediateNode::new_from_variable(x_i.clone());
                        let node = IntermediateNode::<FieldElement<32>> {
                            node_type: NodeType::Mult,
                            left: Some(Box::new(left)),
                            right: Some(Box::new(right)),
                        };
                        node_c.push(node);
                    }
                }
            }
            let root_c = merge_nodes(node_c);

            nodes.push(IntermediateNode {
                node_type: NodeType::Add,
                left: Some(Box::new(a_times_b)),
                right: Some(Box::new(root_c)),
            });
        } else {
            // [] * [] - C = 0
            nodes.push(merge_nodes(node_c));
        }
    }

    let mut opt_nodes = vec![];
    for (i, node) in nodes.iter().enumerate() {
        if !used.contains(&i) {
            opt_nodes.push(vec![node.clone()]);
        }
    }
    opt_nodes
}

pub struct Output<S: PrimeField> {
    pub wire_map: HashMap<usize, S>,
    pub name_map: HashMap<usize, String>,
}

impl<S: PrimeField> Output<S> {
    fn new() -> Self {
        Output {
            wire_map: HashMap::new(),
            name_map: HashMap::new(),
        }
    }

    pub fn get_name(&self, w: usize) -> Option<String> {
        self.name_map.get(&w).cloned()
    }
}

fn make_output(witness: &Vec<wtns_file::FieldElement<32>>, sym: Vec<String>) -> Output<Fr> {
    let n_public = sym.len();

    let mut public = Output::<Fr>::new();

    for i in 0..n_public {
        public
            .wire_map
            .insert(i + 1, Fr::from_repr(witness[i + 1].0).unwrap());
        public.name_map.insert(i + 1, sym[i].clone());
    }

    public
}

pub fn convert_r1cs_wtns_gkr(
    r1cs: R1csFile<32>,
    wtns: WtnsFile<32>,
    sym: String,
) -> (Vec<GKRCircuit<Fr>>, Vec<Input<Fr>>, Output<Fr>) {
    fn append_binary_set(a: &Vec<Vec<Fr>>, b: &Vec<Vec<Fr>>) -> Vec<Vec<Fr>> {
        let mut res = a.clone();
        assert!(b.len() == 1);
        res.push(b[0].clone());
        res
    }
    fn convert_binary_to_vec(b: &String) -> Vec<Fr> {
        let mut res = vec![];
        for c in b.chars() {
            if c == '0' {
                res.push(Fr::zero());
            } else {
                res.push(Fr::one());
            }
        }
        res
    }
    let circuit_info = compile(convert_constraints_to_nodes(&r1cs));
    println!("r1cs is converted to GKR intermediate layers");

    let output_gkr = make_output(
        &wtns.witness.0,
        parse_sym(sym, r1cs.header.n_pub_in + r1cs.header.n_pub_out),
    );

    let mut circuits = vec![];
    let mut inputs = vec![];
    for (layers, input) in circuit_info.0.iter().zip(circuit_info.1.iter()) {
        let mut input_k = get_k(input.len());
        let input_gkr = calculate_input(layers, input, &wtns.witness);

        let mut gkr_layers = vec![];
        for i in 0..layers.len() {
            let k_i = get_k(layers[i].node_types.len());
            let mut v = 0;
            let mut k_next = 0;
            if i == layers.len() - 1 {
                k_next = input_k;
            } else {
                k_next = get_k(layers[i + 1].node_types.len());
            }
            v = k_i + 2 * k_next;

            let mut add_bin_strings: Vec<String> = layers[i]
                .node_types
                .par_iter()
                .enumerate()
                .filter(|(_, node)| **node == NodeType::Add)
                .map(|(curr, node)| {
                    let mut curr_string = format!("{:0k$b}", curr, k = k_i);
                    if k_i == 0 {
                        curr_string = String::new();
                    }
                    let operand_index = layers[i].operand_index[curr];
                    let left_string = format!("{:0k$b}", operand_index.0, k = k_next);
                    let right_string = format!("{:0k$b}", operand_index.1, k = k_next);
                    format!("{}{}{}", curr_string, left_string, right_string)
                })
                .collect();

            let mut add_bin: Vec<Vec<Fr>> = add_bin_strings
                .par_iter()
                .map(|s| convert_binary_to_vec(s))
                .collect();

            let mut add_i = add_bin_strings
                .par_iter()
                .map(|s| chi_w_for_binary::<Fr>(s))
                .reduce(|| get_empty::<Fr>(v), |a, b| add_poly(&a, &b));

            let mut mult_bin_strings: Vec<String> = layers[i]
                .node_types
                .par_iter()
                .enumerate()
                .filter(|(_, node)| **node == NodeType::Mult)
                .map(|(curr, node)| {
                    let mut curr_string = format!("{:0k$b}", curr, k = k_i);
                    if k_i == 0 {
                        curr_string = String::new();
                    }
                    let operand_index = layers[i].operand_index[curr];
                    let left_string = format!("{:0k$b}", operand_index.0, k = k_next);
                    let right_string = format!("{:0k$b}", operand_index.1, k = k_next);
                    format!("{}{}{}", curr_string, left_string, right_string)
                })
                .collect();

            let mut mult_bin: Vec<Vec<Fr>> = mult_bin_strings
                .par_iter()
                .map(|s| convert_binary_to_vec(s))
                .collect();

            let mut mult_i = mult_bin_strings
                .par_iter()
                .map(|s| chi_w_for_binary::<Fr>(s))
                .reduce(|| get_empty::<Fr>(v), |a, b| add_poly(&a, &b));

            if add_i.len() == 0 {
                add_i = get_empty::<Fr>(v);
            }
            if mult_i.len() == 0 {
                mult_i = get_empty::<Fr>(v);
            }
            let wire = (add_bin, mult_bin);
            gkr_layers.push(Layer::new(k_i, add_i, mult_i, wire));
        }
        let circuit = GKRCircuit::new(gkr_layers, input_k);
        circuits.push(circuit);
        inputs.push(input_gkr);
    }

    println!("Convert done.");
    (circuits, inputs, output_gkr)
}

fn calculate_input(
    ir_circuit: &Vec<IntermediateLayer<FieldElement<32>>>,
    input_layer: &Vec<NodeType<FieldElement<32>>>,
    wtns: &Witness<32>,
) -> Input<Fr> {
    let witness = &wtns.0;
    let mut w_values = vec![];
    let mut input = vec![];

    for node in input_layer {
        match node {
            NodeType::Value(e) => match e {
                Expression::Value(v) => {
                    let v_fr = Fr::from_repr(v.0).unwrap();
                    input.push(v_fr);
                }
                Expression::Variable(var) => {
                    let value = witness[var.clone() as usize];
                    input.push(Fr::from_repr(value.0).unwrap());
                }
            },
            _ => panic!("Input value should be an expression"),
        }
    }
    w_values.push(input);
    for layer in ir_circuit.iter().rev() {
        let mut values = vec![];
        for i in 0..layer.node_types.len() {
            match layer.node_types[i] {
                NodeType::Add => {
                    let left = w_values[w_values.len() - 1][layer.operand_index[i].0];
                    let right = w_values[w_values.len() - 1][layer.operand_index[i].1];
                    values.push(left + right);
                }
                NodeType::Mult => {
                    let left = w_values[w_values.len() - 1][layer.operand_index[i].0];
                    let right = w_values[w_values.len() - 1][layer.operand_index[i].1];
                    values.push(left * right);
                }
                NodeType::Value(_) => panic!("Layer types should not be a value"),
            }
        }
        w_values.push(values);
    }
    w_values.reverse();

    let mut w = vec![];
    // d = w[0]
    let d_values = w_values[0].clone();

    // check constraint
    assert_eq!(Fr::zero(), d_values[0]);

    let d = get_multi_ext(&d_values, get_k(d_values.len()));
    w.push(d.clone());
    for (i, layer_value) in w_values.iter().enumerate() {
        if i == 0 {
            continue;
        }
        w.push(get_multi_ext(layer_value, get_k(layer_value.len())));
    }
    Input { w, d }
}

fn parse_sym(sym: String, num_public: u32) -> Vec<String> {
    let mut res = vec![];
    if num_public == 0 {
        return res;
    }

    let mut f = File::open(sym).expect("sym file not found");
    let mut sym_content = String::new();
    f.read_to_string(&mut sym_content).expect("Reading error");

    for line in sym_content.lines() {
        let l: Vec<&str> = line.split(',').collect();
        let name_main: Vec<&str> = l[3].split('.').collect();
        let name = name_main[1].to_string();
        res.push(name);
        if res.len() == (num_public as usize) {
            break;
        }
    }
    res
}
