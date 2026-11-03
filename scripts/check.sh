#!/usr/bin/env bash
set -euo pipefail
task_root="$(cd "$(dirname "$0")/.." && pwd)"
task_run="$(mktemp -d -t adversary-check-XXXXXXXX)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$task_run/target}"
cd "$task_root"
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
(cd sdk/go && go test -race ./...)
(cd examples/bbolt-adapter && go test ./... && go build -mod=readonly -o "$task_run/bbolt-adapter" .)
python3 scripts/demo.py --cli "$CARGO_TARGET_DIR/debug/adversary" --rust "$CARGO_TARGET_DIR/debug/redb-adapter" --go "$task_run/bbolt-adapter" --output "$task_run/demo"
python3 scripts/check_fail_closed.py --cli "$CARGO_TARGET_DIR/debug/adversary" --demo "$task_run/demo" --out "$task_run/negative"
printf 'Validation evidence: %s\n' "$task_run"
