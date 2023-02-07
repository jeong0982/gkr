#![feature(let_chains)]

use ff::PrimeField;
mod aggregator;
mod convert;
mod file_utils;
pub mod gkr;
mod parser;

pub fn gen_proof<S: PrimeField<Repr = [u8; 32]>>() -> () {}
