pragma circom 2.0.0;
include "./verifyLayer.circom"

template Fold(len, r) {
    assert((len \ 2) * 2 == len);
    signal input l[len];
    signal output out[len \ 2];

    for (var i = 0; i < len \ 2; i++) {
        out[i] <== l[i] + r * (l[i + len \ 2] - l[i]);
    }
}

template GetInitialClaim(npowtwo) {
    signal input qPrime[npowtwo];
    signal input out[npowtwo ** 2];

    signal output claim;
    var len = npowtwo ** 2;

    component folded[npowtwo];
    for (var i = 0; i <= npowtwo; i++) {
        var r = qPrime[i];
        folded[i] = Fold(len, r);
        len = len \ 2;
    }
    claim <== folded[npowtwo - 1];
}

template VerfiyGKR(nLayer, npowtwo) {
    signal input claims[nLayer - 1];
    signal input sumcheckProof[nLayer][]

    signal input circuit[nLayer][3];
    signal input layerNumber;
    signal input out[npowtwo ** 2];
    signal input qPrime[npowtwo];

    signal output isValid;

    component initial_claim = GetInitialClaim(npowtwo);
    component layerVerifier[layerNumber - 1];

    initial_claim.qPrime = qPrime;
    initial_claim.out = out;

    for (var layer = layerNumber - 1; layer >= 1; layer--) {
        assert(circuit[layer][1] == 0);
        layerVerifier[layer] = VerifyLayer();
    }
}
