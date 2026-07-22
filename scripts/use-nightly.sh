#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 <nightly-date|nightly-channel>"
    echo "Example: $0 2024-12-01"
    echo "Example: $0 nightly-2024-12-01"
}

if [[ $# -ne 1 ]]; then
    usage
    exit 2
fi

channel="$1"
if [[ "${channel}" != nightly* ]]; then
    channel="nightly-${channel}"
fi

if [[ ! "${channel}" =~ ^nightly(-[0-9]{4}-[0-9]{2}-[0-9]{2})?$ ]]; then
    usage
    exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"

# shellcheck source=./rust-env.sh
source "${script_dir}/rust-env.sh"

toolchain_file="${repo_root}/rust-toolchain.toml"
rustup toolchain install "${channel}" \
    --profile minimal \
    --component rust-src \
    --component llvm-tools-preview \
    --component rustfmt \
    --component clippy

rustup target add riscv64gc-unknown-none-elf --toolchain "${channel}"

sed -i.bak -E "s/^channel = .*/channel = \"${channel}\"/" "${toolchain_file}"
rm -f "${toolchain_file}.bak"

echo "Switched ${toolchain_file} to ${channel}"
