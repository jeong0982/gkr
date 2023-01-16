use clap::Parser;
use r1cs_file::*;
use std::{fs::File, io::Result};
use wtns_file::*;

use crate::{convert::convert_r1cs_wtns_gkr, file_utils::write_output, gkr::prover};

/// zero-knowledge circuit in R1CS format, witness in .wtns binary format
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input R1CS file and sym file
    r1cs_file: String,
    sym_file: String,

    /// Input witness file
    wtns_file: String,

    /// Output json file path
    output_path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    const FS: usize = 32;
    let r1cs = R1csFile::<FS>::read(File::open(args.r1cs_file)?)?;
    let wtns = WtnsFile::<FS>::read(File::open(args.wtns_file)?)?;
    let sym = args.sym_file;

    let result = convert_r1cs_wtns_gkr(r1cs, wtns, sym);
    let proof = prover::prove(result.0, result.1);
    write_output(args.output_path, result.2);

    Ok(())
}

#[cfg(test)]
mod tests {
    use r1cs_file::*;
    use std::fs::File;
    use wtns_file::*;

    #[test]
    fn test_wtns() {
        const FS: usize = 32;
        let r1cs = R1csFile::<FS>::read(File::open("./main.r1cs").unwrap()).unwrap();
        let wtns = WtnsFile::<FS>::read(File::open("./main.wtns").unwrap()).unwrap();
        println!("{}, {}", r1cs.header.n_labels, r1cs.header.n_wires);
        println!("{}", wtns.witness.0.len());
        for i in r1cs.map.0 {
            println!("{}", i);
        }
    }
}
