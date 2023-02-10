pragma circom 2.0.4;

include "./poly/univariate.circom";
include "./poly/multivariate.circom";
include "./sumcheck/sumcheckVerify.circom";


template VerifyGKR(meta) {
    // metadata of circuit
    // 0 --> d
    // 1 --> largest_k
    // 2 --> k_i(0)
    // 3 --> # of terms of D
    // 4 --> largest # of terms among sumcheck proofs (highest degree)
    // 5 --> largest # of terms among q
    // 6 --> # of terms in w_d
    // 7 --> k_i(d - 1)
    // 8 ~ 8 + d - 1 : i --> k_i(i - 11)
    var d = meta[0];
    var largest_k = meta[1];

    signal input sumcheckProof[d - 1][2 * largest_k][meta[4]];
    signal input sumcheckr[d - 1][2 * largest_k];
    signal input q[d - 1][meta[5]];
    signal input D[meta[3]][meta[2] + 1];
    signal input z[d][largest_k];
    signal input r[d - 1];

    signal input inputFunc[meta[6]][meta[7] + 1];

    component m[d - 1];

    component sumcheckVerifier[d - 1];
    component qZero[d - 1];
    component qOne[d - 1];

    component inputValue = evalMultivariate(meta[6], meta[7]);

    for (var i = 0; i < d - 1; i++) {
        sumcheckVerifier[i] = SumcheckVerify(2 * meta[i + 9], meta[4]);
        if (i == 0) {
            sumcheckVerifier[i].claim <== 0;
        } else {
            sumcheckVerifier[i].claim <== m[i - 1].result;
        }
        
        for (var j = 0; j < 2 * meta[i + 9] - 1; j++) {
            sumcheckVerifier[i].r[j] <== sumcheckr[i][j];
        }
        for (var j = 0; j < 2 * meta[i + 9]; j++) {
            for (var k = 0; k < meta[4]; k++) {
                sumcheckVerifier[i].proofs[j][k] <== sumcheckProof[i][j][k];
            }
        }

        m[i] = evalUnivariate(meta[5]);
        for (var j = 0; j < meta[5]; j++) {
            m[i].coeffs[j] <== q[i][j];
        }
        m[i].x <== r[i];
    }

    for (var i = 0; i < meta[6]; i++) {
        for (var j = 0; j < meta[7] + 1; j++) {
            inputValue.terms[i][j] <== inputFunc[i][j];
        }
    }
    for (var j = 0; j < meta[7]; j++) {
        inputValue.x[j] <== z[d - 1][j];
    }
    m[d - 2].result === inputValue.result;
}
