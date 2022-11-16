pragma circom 2.1.0;
include "../poly/univariate.circom"

template SumcheckVerify(v, nTerms) {
    signal input proofs[v][nTerms];
    signal input claim;
    signal input r[v - 1];

    signal output isValid;

    signal expected[v];
    expected[0] <== claim;

    component qZero[v];
    component qOne[v];
    component next[v - 1];
    for (var i = 0; i < v; i++) {
        qZero[i] = evalUnivariate(nTerms);
        qOne[i] = evalUnivariate(nTerms);

        qZero[i].x <== 0;
        qOne[i].x <== 1;

        for (var j = 0; j < nTerms; j++) {
            qZero[i].coeffs[j] <== proofs[i][j];
            qOne[i].coeffs[j] <== proofs[i][j];
        }

        qZero[i].result + qOne[i].result === expected[i];

        if (i != v - 1) {
            next[i] = evalUnivariate(nTerms);
            next[i].x <== r[i];
            for (var j = 0; j < nTerms; j++) {
                next[i].coeffs[j] <== proofs[i][j];
            }
            expected[i + 1] <== next[i].result;
        }
    }

    isValid <== 1;
}
