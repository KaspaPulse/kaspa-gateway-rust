#!/bin/bash -eu

cd "$SRC/kaspa-gateway-rust"

# The repository pins stable Rust for normal builds. cargo-fuzz requires nightly,
# which is provided by the official OSS-Fuzz Rust builder image.
export RUSTUP_TOOLCHAIN=nightly

cargo fuzz build -O --debug-assertions gateway_config_json
cp fuzz/target/x86_64-unknown-linux-gnu/release/gateway_config_json "$OUT/"
