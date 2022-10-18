pragma circom 2.0.0;

template evalUnivariate(nClaim) {
    signal input challenge;
    signal input claims[nClaim];
    signal output evaluated[nClaim];

    evaluated[nClaim - 1] <== claims[nClaim - 1];
    for (var i = nClaim - 2; i >= 0; i--) {
        evaluated[i] <== evaluated[i + 1] * challenge + claims[i];
    }
}
