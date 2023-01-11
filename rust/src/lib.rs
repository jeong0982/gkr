use ff::PrimeField;
mod convert;
pub mod gkr;
mod parser;

pub fn gen_proof<S: PrimeField<Repr = [u8; 32]>>(file: String) -> () {
    // Result<gkr::Proof<S>, ()> {

    // let _dag = parse_circom(file);
    // circuit = convert_dag_circuit(dag)
    let circuit = gkr::GKRCircuit::<S> { layer: vec![] };
    // gkr::prover::prove(circuit)
}
