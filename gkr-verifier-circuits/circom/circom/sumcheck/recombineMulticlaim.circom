pragma circom 2.0.0;
include "../../node_modules/circomlib/circuits/mimc.circom"
include "../poly/univariate.circom"

template getChallenge(nInput) {
    signal input in[nInput];
    signal output out;
    component mimc = MultiMiMC7(nInput, 91);
    for (var i = 0; i < nInput; i++) {
        mimc.in[i] <== in[i];
    }
    out <== mimc.out;
}

template recombineMulticlaims(nClaim) {
    signal input claims[nClaim];
    signal output recombinedClaim;
    signal output evaluatedValue;

    component challenge = getChallenge(nClaim);
    for (var i = 0; i < nClaim, i++) {
        challenge.in[i] <== claims[i];
    }

    component evaluated = evalUnivariate(nClaim);
    evaluated.challenge <== challenge.out;
    for (var i = 0; i < nClaim; i++) {
        evaluated.claims[i] <== claims[i];
    }
    evaluatedValue <== evaluated.evaluated[0];
    recombinedClaim <== challenge.out;
}
