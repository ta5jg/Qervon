#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary_dir=${QERVON_BINARY_DIR:-"$root/bin"}

mkdir -p "$binary_dir"
cd "$root/backend"
cargo build --locked --release -p qervon-api-gateway -p qervon-migration-runner -p qervon-worker
install -m 0755 target/release/qervon-api-gateway "$binary_dir/qervon-api-gateway"
install -m 0755 target/release/qervon-migration-runner "$binary_dir/qervon-migration-runner"
install -m 0755 target/release/qervon-worker "$binary_dir/qervon-worker"
echo "Qervon release binaries installed in $binary_dir."
