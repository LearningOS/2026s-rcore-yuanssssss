#!/usr/bin/env bash

# Source this file before running rustup/cargo commands when network access to
# the default Rust servers is slow.
export RUSTUP_DIST_SERVER="${RUSTUP_DIST_SERVER:-https://rsproxy.cn}"
export RUSTUP_UPDATE_ROOT="${RUSTUP_UPDATE_ROOT:-https://rsproxy.cn/rustup}"

echo "RUSTUP_DIST_SERVER=${RUSTUP_DIST_SERVER}"
echo "RUSTUP_UPDATE_ROOT=${RUSTUP_UPDATE_ROOT}"
