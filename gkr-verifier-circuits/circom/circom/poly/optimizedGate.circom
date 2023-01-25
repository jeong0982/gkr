pragma circom 2.0.4;

template evalGateFunction(nTerms, v) {
    signal input x[v];
    signal input terms[nTerms][v + 1];

    signal output result;
    signal subres[nTerms];
    signal termres[nTerms][v + 1];
    for (var i = 0; i < nTerms; i++) {
        for (var j = 0; j < v + 1; j++) {
            if (j == 0) {
                termres[i][0] <-- terms[i][0];
            } else {
                termres[i][j] <-- termres[i][j - 1] 
                * (((1 / 2) * terms[i][j] * (terms[i][j] - 1) * x[j - 1]) 
                + ((1 / 2) * (terms[i][j] - 1) * (terms[i][j] - 2))
                + ((x[j - 1] - 1) * terms[i][j] * (terms[i][j] - 2)));
            }
        }
        if (i == 0) {
            subres[0] <-- termres[0][v];
        } else {
            subres[i] <-- subres[i - 1] + termres[i][v];
        }
    }
    result <== subres[nTerms - 1];
}
