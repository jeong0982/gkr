use constraint_generation::extract_dag;
use dag::*;
use parser::run_parser;

use clap::Parser;
use r1cs_file::*;
use std::{fs::File, io::Result};
use wtns_file::*;

const VERSION: &'static str = "2.1.0";

pub fn parse_circom(file: String) -> DAG {
    let parse_result = run_parser(file, VERSION, vec![]);
    let program = match parse_result {
        Ok(r) => r.0,
        _ => panic!("Parse error"),
    };

    extract_dag(program)
}

/// zero-knowledge circuit in R1CS format, witness in .wtns binary format
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input R1CS file
    r1cs_file: String,

    /// Input witness file
    wtns_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    const FS: usize = 32;
    let r1cs = R1csFile::<FS>::read(File::open(args.r1cs_file)?)?;
    let wtns = WtnsFile::<FS>::read(File::open(args.wtns_file)?)?;

    Ok(())
}
