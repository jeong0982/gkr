pragma circom 2.0.0;
include "./recombineMulticlaim.circom"

template SumcheckVerify(nClaim, lenProof) {
    signal input claims[nClaim];
    signal input sumcheckProof[lenProof][nClaim];
    signal output challenges[lenProof];
    signal output expectedValue;

    component recombined = recombineMulticlaims(nClaim);
    for (var i = 0; i < nClaim; i++) {
        recombined.claims[i] <== claims[i];
    }

    component actual[lenProof];
    component atone[lenProof];
    component expected = evalUnivariate(nClaim);

    for (var i = 0; i < lenProof; i++) {
        actual[i] = evalUnivariate(nClaim);
        atone[i] = evalUnivariate(nClaim);
        actual[i].x <== 0;
        atone[i].x <== 1;
        for (var j = 0; j < nClaim; j++) {
            actual[i].coeffs[j] <== sumcheckProof[i][j];
        }

        actual[i].evaluated[0] === atone[i].evaluated[0];
        component challenge = getChallenge(nClaim);
        for (var j = 0; j < nClaim, j++) {
            challenge.in[j] <== sumcheckProof[i][j];
        }

        challenges[i] <== challenge.out;

        if (i == lenProof - 1) {
            expected.x <== challenge.out;
            for (var j = 0; j < nClaim; j++) {
                expected.coeffs[j] <== sumcheckProof[i][j];
            }
        }
    }
    expectedValue <== expected.evaluated[0];
}
