#!/usr/bin/env bash
set -euo pipefail
# Downloads a separate Go toolchain through GOTOOLCHAIN; requires a C compiler.
task_root="$(cd "$(dirname "$0")" && pwd)"
task_run="$(mktemp -d -t adversary-gosim-XXXXXXXX)"
git clone https://github.com/jellevandenhooff/gosim.git "$task_run/gosim"
git -C "$task_run/gosim" checkout ffd3a613542675755e4cbf8186b5edaf404ed95c
cp "$task_root/gosim-contract/adversary_test.go" "$task_run/gosim/examples/bolt/"
cd "$task_run/gosim"
GOTOOLCHAIN=go1.23.2 CGO_ENABLED=1 go run ./cmd/gosim test -v -seeds 1-5 -run TestAdversaryDurability ./examples/bolt
printf 'Checkout: %s\n' "$task_run/gosim"
