use ff::PrimeField;
use halo2curves::bn256::Fr;
use serde::{Serialize, Deserialize};
use num_bigint::BigInt;
use num_traits::Num;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

use crate::convert::Output;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    value_map: HashMap<String, String>,
}

pub fn stringify_fr(f: Fr) -> String {
    let r = f.to_repr();
    let mut s = String::from("0x");
    for &b in r.iter().rev() {
        s = format!("{}{:02x}", s, b);
    }
    let decimal = BigInt::from_str_radix(&s, 16).unwrap().to_str_radix(10);
    decimal
}

fn make_output_value_map(output: Output<Fr>) -> Data {
    let mut value_map = HashMap::new();
    for (k, i) in output.wire_map.iter() {
        let name = output.get_name(k.clone()).expect("Wire map and name map should have a same key");
        let value = stringify_fr(i.clone());
        value_map.insert(name, value);
    }
    Data { value_map }
}

pub fn write_output(path: String, output: Output<Fr>) {
    let data = make_output_value_map(output);
    let json_string = serde_json::to_string(&data).unwrap();

    fs::write(path, json_string).expect("Unable to write file");
}
