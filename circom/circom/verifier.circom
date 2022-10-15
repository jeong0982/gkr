pragma circom 2.0.0;
include "./verifyLayer.circom"

template VerfiyGKR(nLayer) {
    signal input claims[nLayer];
    signal input circuit[nLayer][3];
    signal input layerNumber;
    signal output isValid;

    component layerVerifier[layerNumber - 1];

    for (var layer = layerNumber - 1; layer >= 1; layer--) {
        assert(circuit[layer][1] == 0);
        layerVerifier[layer] = VerifyLayer();
    }
}
