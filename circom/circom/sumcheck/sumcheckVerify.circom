pragma circom 2.0.0;
include "./recombineMulticlaim.circom"

template SumcheckVerify(nClaim, lenProof) {
    signal input claims[nClaim];
    signal input sumcheckProof[lenProof];
    signal output challenges[lenProof];

    component recombined = recombineMulticlaims(nClaim);
    for (var i = 0; i < nClaim; i++) {
        recombined.claims[i] <== claims[i];
    }
    component actual[lenProof];
    for (var i = 0; i < lenProof; i++) {
        actual[i] = evalUnivariate();
    }
}
