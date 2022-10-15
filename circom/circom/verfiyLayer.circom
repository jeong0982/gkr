pragma circom 2.0.0;
include "./sumcheck.circom"

template VerifyLayer(inLength, outLength) {
    signal input in[inLength];
    signal input out[outLength];
}
