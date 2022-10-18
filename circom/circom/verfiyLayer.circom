pragma circom 2.0.0;
include "./sumcheck/sumcheckVerify.circom"

template VerifyInitial
template VerifyLayer(inLength, outLength) {
    signal input in[inLength];
    signal input out[outLength];

    component sumcheck[]
}
