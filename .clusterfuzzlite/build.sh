#!/bin/bash -eu

cd "$SRC/kaspa-gateway-rust"

# cargo-fuzz needs nightly; the OSS-Fuzz Rust builder provides it.
export RUSTUP_TOOLCHAIN=nightly

cargo fuzz build -O --debug-assertions gateway_config_json

# OSS-Fuzz/ClusterFuzzLite can influence Cargo's target layout. Do not assume
# a fixed target directory; locate the exact release binary that cargo-fuzz built.
FUZZ_BIN="$(
    find "$PWD" \
        -type f \
        -name gateway_config_json \
        -path '*/release/gateway_config_json' \
        -perm -111 \
        -print \
        -quit
)"

if [[ -z "$FUZZ_BIN" ]]; then
    echo "gateway_config_json was built but its release binary was not found" >&2
    find "$PWD" -type f -name 'gateway_config_json*' -print >&2 || true
    exit 1
fi

echo "FUZZ_BINARY=$FUZZ_BIN"
cp "$FUZZ_BIN" "$OUT/gateway_config_json"
