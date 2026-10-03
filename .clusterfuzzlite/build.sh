#!/bin/bash -eu

# OSS-Fuzz/ClusterFuzzLite requires a build.sh entrypoint.
# All project-owned build logic lives in the Rust xtask.
cd "$SRC/kaspa-gateway-rust"
# ClusterFuzzLite injects sanitizer flags globally. Applying them while
# bootstrapping xtask instruments host proc-macros and native build dependencies,
# which either Rust rejects or leaves the bootstrap linker without the ASan runtime.
# Preserve native sanitizer flags for the Rust-owned cargo-fuzz child, while the
# xtask bootstrap itself stays unsanitized. cargo-fuzz owns Rust ASan/cfg(fuzzing).
export KGW_CFL_CFLAGS="${CFLAGS-}"
export KGW_CFL_CXXFLAGS="${CXXFLAGS-}"
unset RUSTFLAGS CFLAGS CXXFLAGS
exec cargo +nightly run --locked -p xtask -- fuzz-build
