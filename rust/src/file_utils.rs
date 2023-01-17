use ff::PrimeField;
use halo2curves::bn256::Fr;
use num_bigint::BigInt;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, from_str, Value};
use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::process::Command;

use crate::aggregator::CircomInputProof;
use crate::convert::Output;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    value_map: HashMap<String, String>,
}

pub fn stringify_fr(f: &Fr) -> String {
    let r = f.to_repr();
    let mut s = String::from("");
    for &b in r.iter().rev() {
        s = format!("{}{:02x}", s, b);
    }
    let decimal = BigInt::from_str_radix(&s, 16).unwrap().to_str_radix(10);
    decimal
}

fn make_output_value_map(output: Output<Fr>) -> Data {
    let mut value_map = HashMap::new();
    for (k, i) in output.wire_map.iter() {
        let name = output
            .get_name(k.clone())
            .expect("Wire map and name map should have a same key");
        let value = stringify_fr(i);
        value_map.insert(name, value);
    }
    Data { value_map }
}

pub fn write_output(path: String, output: Output<Fr>) {
    let data = make_output_value_map(output);
    let json_string = serde_json::to_string(&data.value_map).unwrap();

    fs::write(path, json_string).expect("Unable to write file");
}

pub fn write_aggregated_input(path: String, input: CircomInputProof) -> String {
    let file = fs::File::open(path).unwrap();
    let mut input_json: HashMap<String, Value> = from_reader(file).unwrap();

    let proof_string = serde_json::to_string(&input).unwrap();
    let proof_data: HashMap<String, Value> = from_str(&proof_string).unwrap();

    for (k, v) in proof_data {
        input_json.insert(k, v);
    }
    let json_string = serde_json::to_string(&input_json).unwrap();

    let root = current_dir().unwrap();
    let new_path = root.join("aggregated.json");
    fs::write(&new_path, json_string).unwrap();
    new_path.into_os_string().into_string().unwrap()
}

pub fn get_name(path: &String) -> String {
    let binding = path.clone();
    let path_str: Vec<&str> = binding.as_str().split('/').collect();
    let name_tuple: Vec<&str> = path_str[path_str.len() - 1].split('.').collect();
    String::from(name_tuple[0])
}

pub fn execute_circom(path: String, input_path: &String) -> (String, String) {
    let _ = Command::new("circom")
        .arg(path.clone())
        .arg("--r1cs")
        .arg("--sym")
        .arg("--wasm")
        .output()
        .expect("circom command failed");
    println!("");
    let path_str: Vec<&str> = path.as_str().split('/').collect();
    let mut path_cloned = path_str.clone();
    path_cloned.pop();
    let mut root_path = String::new();
    for slice in path_cloned {
        root_path = format!("{}/", slice);
    }
    let circom_name: Vec<&str> = path_str[path_str.len() - 1].split('.').collect();
    let name = circom_name[0];

    let witness_gen_name = format!("{}_js/", name);
    let witness_gen_file = current_dir()
        .unwrap()
        .join(witness_gen_name.clone())
        .join("generate_witness.js");
    let wasm = current_dir()
        .unwrap()
        .join(witness_gen_name)
        .join(format!("{}.wasm", name));

    let _ = Command::new("node")
        .arg(witness_gen_file.clone())
        .arg(wasm)
        .arg(input_path.clone())
        .arg("witness.wtns")
        .status()
        .expect("witness calculator generation failed");
    println!("");
    (String::from(name), root_path)
}

#[cfg(test)]
mod tests {
    use crate::aggregator::CircomInputProof;

    use super::write_aggregated_input;

    #[test]
    fn test_aggregate_input() {
        let cp = CircomInputProof::empty();
        write_aggregated_input(String::from("./input.json"), cp);
    }
}
