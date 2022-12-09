# GKR protocol
General GKR prover - arithmetic circuit version

## Getting started
Proof aggregation implemented in Rust is in development. To make GKR proof, use `./python/`.  
Circuit modification can be done by modifying `Circuit` class in `test_gkr.py`.\
To use circom circuit, uncomment line 115 in `test_gkr.py` (`generate_json(proof)`) and run circom to `./gkr-verifier-circuits/circom/circom/verifier.circom`.

# Proof aggregation using GKR

`GKRCircuit`: Generic struct of GKR circuit  

## Circom-GKR
### Internal
#### Initial round
Get input from `input.json`, make `d` in proof with it.  
Parse r1cs file and convert it to `GKRCircuit`. (Let's call this $C_0$)  
Make proof $\pi_0$ from `d` and `GKRCircuit`.
#### Iterative round (0 < $i$ < n)
There are two circuit $C_i$ and $C_{v_{i - 1}}$. $C_{v_{i - 1}}$ is circuit that can verify $C_{i - 1}$.  
$C_{v_i}$ can be different form for each circuit $C_i$. If use same circuit for all $C_i$, then $C_v$ will be same circuit.  
To make aggregated proof for previous proof and current round's proof, we need
- input (for $C_i$)
- proof $\pi_{i - 1}$

Make integrated circuit $C'_{i}$. Use those inputs, make proof $\pi_{i}$.
To be specific, input and proof $\pi_{i - 1}$
#### Last round
Also there are two circuit $C_n$ and $C_{v_{n - 1}}$. To send aggregated proof to on-chain verifier, we can use groth16 prover in `snarkjs`.  
Integrated circuit $C'_{n}$ can be proved with `snarkjs` also.  
So final proof $\pi_n$ is groth16 or plonk proof

### External (For use)
*CLI is in development*
#### Use same circuit for aggregation
1. Put circuit(`.circom`) to aggregator
2. Give each input for circuit to aggregator
3. Get final aggregated proof from it
#### Use different circuit for aggregation
Not sure if this is secure or not.
1. Put circuits to aggregator
2. Give input for each circuits to aggregator
3. Specify what circuit each input goes for
4. Get final aggregated proof from it

## halo2-GKR
### Internal
Most of steps are same. halo2-GKR is from halo2 `Circuit` struct.
Convert `Circuit` to `GKRCircuit`. After it, all the steps are same as circom-GKR.
### External
**TBD**  
halo2-GKR can be given by form of library. Use function from lib to make recursion and final proof can be made by halo2.
