#!/bin/bash -eu

# OSS-Fuzz/ClusterFuzzLite requires a build.sh entrypoint.
# All project-owned build logic lives in the Rust xtask.
cd "$SRC/kaspa-gateway-rust"
exec cargo +nightly run --locked -p xtask -- fuzz-build
