use crate::parser::parse_circom;
use ff::PrimeField;
use r1cs_file::R1csFile;
use std::fs::File;

mod convert;
pub mod gkr;
mod parser;

pub fn gen_proof<S: PrimeField<Repr = [u8; 32]>>(file: String) -> Result<gkr::Proof<S>, ()> {
    const FS: usize = 32;
    let r1cs = R1csFile::<FS>::read(File::open(file))?;

    // let _dag = parse_circom(file);
    // circuit = convert_dag_circuit(dag)
    let circuit = gkr::GKRCircuit {
        layer: vec![],
        d: vec![],
    };
    gkr::prover::prove(circuit)
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_circom;
    use crate::File;
    use crate::R1csFile;

    #[test]
    fn test_dag() {
        let _dag = parse_circom(String::from("test.circom"));
    }
}
