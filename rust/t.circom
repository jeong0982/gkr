pragma circom 2.0.0;
include "../gkr-verifier-circuits/circom/node_modules/circomlib/circuits/mimc.circom";

template A(){
    signal input in1;
    signal input in2;
    signal output out;

    component hasher = MiMC7(91);
    hasher.x_in <== in1;
    hasher.k <== 0;
    
    out <== hasher.out;
}

component main {public [in1]}= A();
