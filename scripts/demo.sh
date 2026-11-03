#!/usr/bin/env bash
set -euo pipefail
task_root="$(cd "$(dirname "$0")/.." && pwd)"
task_run="$(mktemp -d -t adversary-build-demo-XXXXXXXX)"
export CARGO_TARGET_DIR="$task_run/target"
cargo build --release --locked --manifest-path "$task_root/Cargo.toml" --workspace
(cd "$task_root/examples/bbolt-adapter" && go build -mod=readonly -o "$task_run/bbolt-adapter" .)
python3 "$task_root/scripts/demo.py" --cli "$CARGO_TARGET_DIR/release/adversary" --rust "$CARGO_TARGET_DIR/release/redb-adapter" --go "$task_run/bbolt-adapter" --output "$task_run/results"
