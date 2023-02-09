# Proof aggregator using recursive GKR scheme
This is Cli tool for **generation of aggregated proof for multiple inputs**.

In this version, it supports circom circuit.

## Preliminaries
circom and snarkjs should be installed already.

You can check that by this command:
```sh
snarkjs --help
```
```sh
circom --help
```
## How to use
### 1. Install gkr
```sh
cargo install --path ./rust
```
### 2. Move to `./rust`
```sh
cd rust
```
### 3. Write a circuit in `./rust` and inputs in `./rust/example/` (`/example` is not mandatory)

### 4. Create GKR proof for inputs
You can give inputs by commands:
```sh
gkr-aggregator -c circuit.circom -i ./example/input1.json ./example/input2.json ./example/input3.json
```

You can get a message from cli:
```sh
Proving by groth16 can be done
```

### 4. Prepare zkey
You should prepare an appropriate ptau file.
```sh
snarkjs groth16 setup aggregated.r1cs pot.ptau c0.zkey
snarkjs zkey contribute c0.zkey c1.zkey --name=“mock” -v
```
Give random string for contribution, and then
```sh
snarkjs zkey beacon c1.zkey c.zkey 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 10 -n="Final Beacon phase2"
```

### 5. Create aggregated Groth16 proof
```sh
gkr-aggregator mock-groth -z c.zkey
```
You can get `proof.json` and `public.json`.

## Implementation details
### Internal
#### Initial round
Get input from `input.json`, make `d` in proof with it.  
Parse r1cs file and convert it to `GKRCircuit`. (Let's call this $C$)  
Make proof $\pi_0$ from `d` and `GKRCircuit`.
#### Iterative round (0 < $i$ < n)
There are two circuit $C_i$ and $C_{v_{i - 1}}$. $C_{v_{i - 1}}$ is circuit that can verify $C_{i - 1}$.  
$C_{v_i}$ can be different form for each circuit $C_i$. 
To make aggregated proof for previous proof and current round's proof, we need
- input (for $C_i$)
- proof $\pi_{i - 1}$

Make integrated circuit $C'_i$.

Use those inputs, make proof $\pi_i$.
To be specific, input and proof $\pi_{i - 1}$
#### Last round
Also there are two circuit $C_n$ and $C_{v_{n - 1}}$. To send aggregated proof to on-chain verifier, we can use groth16 prover in `snarkjs`.  
Integrated circuit $C'_{n}$ can be proved with `snarkjs` also.  
So final proof $\pi_n$ is groth16 or plonk proof
