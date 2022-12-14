use ff::PrimeField;
use r1cs_file::R1csFile;

use crate::gkr::GKRCircuit;

const S: PrimeField = halo2curves::bn256::Fr;
pub fn convert_r1cs_gkr(r1cs: R1csFile<32>) -> GKRCircuit<S> {
    
}
