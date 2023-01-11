use clap::Parser;
use r1cs_file::*;
use std::{fs::File, io::Result};
use wtns_file::*;

use crate::{convert::convert_r1cs_wtns_gkr, gkr::prover};

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

    let result = convert_r1cs_wtns_gkr(r1cs, wtns);
    let proof = prover::prove(result.0, result.1);

    Ok(())
}
