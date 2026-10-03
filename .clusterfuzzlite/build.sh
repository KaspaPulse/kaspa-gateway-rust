#!/bin/bash -eu

# OSS-Fuzz/ClusterFuzzLite requires a build.sh entrypoint.
# All project-owned build logic lives in the Rust xtask.
cd "$SRC/kaspa-gateway-rust"
# ClusterFuzzLite injects sanitizer RUSTFLAGS globally. Applying them while
# bootstrapping xtask also instruments host proc-macro crates, which Rust rejects.
# The Rust-owned fuzz-build stage requests AddressSanitizer from cargo-fuzz itself.
unset RUSTFLAGS
exec cargo +nightly run --locked -p xtask -- fuzz-build
