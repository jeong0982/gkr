pragma circom 2.0.4;

template evalUnivariate(n) {
    signal input x;
    signal input coeffs[n];

    signal evaluated[n];
    signal output result;

    evaluated[0] <== coeffs[0];
    for (var i = 1; i < n; i++) {
        evaluated[i] <== evaluated[i - 1] * x + coeffs[i];
    }
    result <== evaluated[n - 1];
}
