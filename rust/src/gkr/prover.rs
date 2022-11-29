use super::{GKRCircuit, Proof};
use ff::PrimeField;

pub fn prove<S: PrimeField>(_circuit: GKRCircuit<S>) -> Result<Proof<S>, ()> {
    Err(())
}
