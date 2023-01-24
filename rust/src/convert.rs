use r1cs_file::{Constraint, FieldElement, R1csFile};
use wtns_file::*;

use crate::{
    file_utils::stringify_fr,
    gkr::{poly::*, GKRCircuit, Input, Layer},
};
use halo2curves::bn256::Fr;
use halo2curves::group::ff::PrimeField;
use rayon::prelude::*;
use std::{collections::HashMap, fmt::Debug, fs::File, io::Read, ops::Deref};

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

fn make_node_from_constraint(constraint: &Constraint<32>) -> IntermediateNode<FieldElement<32>> {
    let one = FieldElement(Fr::one().to_repr());
    let a = &constraint.0;
    let b = &constraint.1;
    let c = &constraint.2;

    let mut node_a = vec![];
    let mut node_b = vec![];
    let mut node_c = vec![];

    for (coeff, x_i) in a {
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
    for (coeff, x_i) in b {
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
    for (coeff, x_i) in c {
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
    nodes: Vec<IntermediateNode<FieldElement<32>>>,
) -> (
    Vec<IntermediateLayer<FieldElement<32>>>,
    Vec<NodeType<FieldElement<32>>>,
) {
    println!("Compile nodes..");
    let mut layers = vec![];

    let zero = FieldElement::from((Fr::zero()).to_repr());

    let height = nodes.iter().map(|node| node.depth()).max().unwrap_or(0);
    if height == 0 {
        return (layers, vec![]);
    }
    let mut inputs = vec![];

    let mut used: HashMap<Expression<FieldElement<32>>, usize> = HashMap::new();
    let mut current_nodes = nodes.clone();
    let mut next_nodes = vec![];
    let mut zero_index = None;
    for d in 0..(height + 1) {
        let mut layer_operand_idx = vec![];
        let mut node_types = vec![];
        println!("n leng {:?}", current_nodes.len());
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
    (layers, inputs)
}

fn convert_constraints_to_nodes(r1cs: &R1csFile<32>) -> Vec<IntermediateNode<FieldElement<32>>> {
    let constraints = &r1cs.constraints;
    let mut nodes = vec![];
    for constraint in &constraints.0 {
        nodes.push(make_node_from_constraint(constraint));
    }
    nodes
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

fn make_output(witness: Vec<wtns_file::FieldElement<32>>, sym: Vec<String>) -> Output<Fr> {
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
) -> (GKRCircuit<Fr>, Input<Fr>, Output<Fr>) {
    let circuit_info = compile(convert_constraints_to_nodes(&r1cs));
    println!("r1cs is converted to GKR intermediate layers");

    let layers = circuit_info.0;
    let input = circuit_info.1;

    let mut input_k = get_k(input.len());
    let input_gkr = calculate_input(layers.clone(), input, &wtns.witness);
    println!("All inputs are calculated");
    let output_gkr = make_output(
        wtns.witness.0,
        parse_sym(sym, r1cs.header.n_pub_in + r1cs.header.n_pub_out),
    );
    println!("Layer depth: {:?}", layers.len());

    let mut gkr_layers = vec![];
    for i in 0..(layers.len() - 1) {
        println!("{:?} layer starts", i);
        let k_i = get_k(layers[i].node_types.len());
        let k_next = get_k(layers[i + 1].node_types.len());
        let v = k_i + 2 * k_next;

        let mut mult_i = get_empty::<Fr>(v);
        let mut add_i = get_empty::<Fr>(v);

        let mut add_m = false;
        let mut mult_m = false;

        let binary_inputs: Vec<String> = generate_binary_string(v);
        add_i = binary_inputs.par_iter()
        .filter(|b| {
            let curr = usize::from_str_radix(&b[0..k_i], 2).unwrap_or(0);
            let next_left = usize::from_str_radix(&b[k_i..k_i + k_next], 2).unwrap();
            let next_right = usize::from_str_radix(&b[k_i + k_next..], 2).unwrap();
            layers[i].operand_index[curr] == (next_left, next_right)
                && layers[i].node_types[curr] == NodeType::Add
        })
        .map(|b| chi_w_for_binary::<Fr>(b))
        .reduce(|| get_empty::<Fr>(v - 1), |a, b| add_cb(&a, &b));

        mult_i = binary_inputs.par_iter()
        .filter(|b| {
            let curr = usize::from_str_radix(&b[0..k_i], 2).unwrap_or(0);
            let next_left = usize::from_str_radix(&b[k_i..k_i + k_next], 2).unwrap();
            let next_right = usize::from_str_radix(&b[k_i + k_next..], 2).unwrap();
            layers[i].operand_index[curr] == (next_left, next_right)
                && layers[i].node_types[curr] == NodeType::Mult
            })
            .map(|b| chi_w_for_binary::<Fr>(b))
            .reduce(|| get_empty::<Fr>(v - 1), |a, b| add_cb(&a, &b));
        
        if add_i.len() == 0 {
            add_i = get_empty::<Fr>(v - 1);
        }
        if mult_i.len() == 0 {
            mult_i = get_empty::<Fr>(v - 1);
        }
        gkr_layers.push(Layer::new(k_i, add_i, mult_i));
    }
    (GKRCircuit::new(gkr_layers, input_k), input_gkr, output_gkr)
}

fn calculate_input(
    ir_circuit: Vec<IntermediateLayer<FieldElement<32>>>,
    input_layer: Vec<NodeType<FieldElement<32>>>,
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
                    let value = witness[var as usize];
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
