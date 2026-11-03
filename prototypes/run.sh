#!/usr/bin/env bash
set -euo pipefail
# Linux x86_64, Rust >=1.90, Go >=1.26, Python 3, git and strace.
task_root="$(cd "$(dirname "$0")" && pwd)"
task_run="$(mktemp -d -t adversary-repro-XXXXXXXX)"
export CARGO_TARGET_DIR="$task_run/rust-target"
cargo build --release --locked --manifest-path "$task_root/rust-worker/Cargo.toml"
(cd "$task_root/go-worker" && go build -mod=readonly -o "$task_run/go-worker" .)
python3 "$task_root/probe.py" --rust "$CARGO_TARGET_DIR/release/adversary-redb-probe" --go "$task_run/go-worker" --out "$task_run/results"
(cd "$task_root" && python3 -m unittest -v test_fidelity)
(cd "$task_root/synctest-probe" && go test -v -count=1 .)
printf 'Results: %s\n' "$task_run/results"

