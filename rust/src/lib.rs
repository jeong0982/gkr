use crate::parser::parse_circom;
use ff::PrimeField;

pub mod gkr;
mod parser;

pub fn gen_proof<S: PrimeField<Repr = [u8; 32]>>(file: String) -> Result<gkr::Proof<S>, ()> {
    let _dag = parse_circom(file);
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

    #[test]
    fn test_dag() {
        let _dag = parse_circom(String::from("test.circom"));
    }
}
