pragma circom 2.0.0;

template evalUnivariate(n) {
    signal input x;
    signal input coeffs[n];
    signal output evaluated[n];

    evaluated[n - 1] <== coeffs[n - 1];
    for (var i = n - 2; i >= 0; i--) {
        evaluated[i] <== evaluated[i + 1] * x + coeffs[i];
    }
}
