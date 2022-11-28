use gkr::{prover, verifier};
use crate::parser::parse_circom;

pub mod gkr;
mod parser;

pub fn gen_proof(file: String) -> Result<gkr::Proof, ()> {
    let dag = parse_circom(file);
    prover::prove(dag)
}
