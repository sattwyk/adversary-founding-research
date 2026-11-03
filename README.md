# Adversary development preview

Adversary makes advanced correctness testing easier to use on real Rust and Go systems. This repository implements a small native crash/recovery workflow and fixes the interfaces around which the first product can be built.

Read the [product decisions](docs/01-decisions.md), [architecture and interfaces](docs/02-system-and-interfaces.md), and [backend qualification plan](docs/03-backends-and-fault-semantics.md) for the V1 design. Earlier research and experimental evidence remain at the repository root.

**Implemented:** Rust CLI, Rust/Go adapter SDKs, a shared atomic-map contract, cooperative crash checkpoints, process termination, evidence journals, replay, greedy workload reduction, immutable run directories, explicit inconclusive results, and CI checks. Real redb and bbolt adapters exercise the same protocol.

**Experimental:** a separate QEMU/ext4 guest-loss probe. It is not selectable as a CLI backend. Turmoil, Gosim, Faultbox, generic models, networking faults, hosted collaboration and managed execution are not implemented in this preview. There is no implicit fallback when a requested backend is unsupported.

## Run the complete demonstration

Linux x86_64; tested with Rust 1.97.1, Go 1.26.4 and Python 3. Build/download access is required. No root, Docker or custom runtime is needed for native tests.

```bash
bash scripts/demo.sh
```

This builds the CLI and both real library adapters. The demonstration checks correct atomic implementations, finds an intentionally split-transaction application bug in both languages, reproduces it, reduces three operations to one, and checks the corrected implementation with the original contract. It also demonstrates the native process-kill blind spot of bbolt's relaxed durability configuration. These are deliberate example bugs/configurations, not new upstream defects.

Run all validation, including malformed/missing evidence and timeout checks:

```bash
bash scripts/check.sh
```

For a project integration:

```bash
cargo build --release --workspace --locked
./target/release/adversary init --language rust
# Point the manifest at your adapter and declare initial state and crash boundary.
./target/release/adversary doctor
./target/release/adversary test --output .adversary/run-1
./target/release/adversary inspect .adversary/run-1
```

After a failure:

```bash
./target/release/adversary replay .adversary/run-1 --output .adversary/replay-1
./target/release/adversary reduce .adversary/run-1 --output .adversary/reduced-1
./target/release/adversary replay .adversary/run-1 --output .adversary/fix-1 \
  --allow-changed --manifest fixed-implementation.toml
```

Fix evaluation must preserve the contract, fault specification and backend. It may change the adapter artifact/configuration. Ordinary replay checks the top-level adapter executable hash. It does not pin shared libraries, the complete dependency closure, kernel scheduling, runtime behavior or external services. Keep the manifest, workload and compatible build environment with a regression. Changing artifact location requires a replacement manifest; changed-artifact evaluation is labeled separately.

## Integrate an adapter

The SDK is a test adapter transport, not a replacement runtime. It communicates over bounded NDJSON stdin/stdout and obtains a fresh data directory through `ADVERSARY_DATA_DIR`. Diagnostic logs go to stderr. Expose:

- `initialize`: establish the declared baseline using the application's actual durability semantics.
- `apply`: perform one declared atomic map batch and return `{"accepted":true}` only after its success boundary.
- `observe`: return all declared keys as a JSON object; values are strings or explicit `null` for absence.

Rust's `adversary_sdk::serve` and Go's `adversary.Serve` handle framing. `Context.checkpoint` / `Context.Checkpoint` optionally pause a test operation until the controller continues it or terminates its process. See [redb adapter](examples/redb-adapter/src/main.rs) and [bbolt adapter](examples/bbolt-adapter/main.go). The Go module name `adversary.local/sdk` is a local development namespace, not a published package.

The default contract is a narrow first preset, not a universal application API. It checks a finite sequential batch workload. A fully acknowledged operation must survive; a batch interrupted at a checkpoint may take effect completely or have no effect. A partial batch is a counterexample. Missing/malformed observation is inconclusive; explicit absence is a valid observation that can violate the contract.

## Evidence and limits

Each run records the resolved scenario, exact workload, executable hash, per-case recovered state, acknowledgments/unknown outcome, request/event journals, fault confirmation and stderr. No-violation means no violation in this completed sampled workload. No result is a proof of whole-system correctness.

Exit codes: 0 = completed sampled scope without a violation; 1 = violation; 2 = inconclusive; 3 = invalid scenario/tool failure. A requested checkpoint that never occurs is inconclusive. Unsupported backends, empty campaigns, bad schemas and contract weakening are rejected. Campaigns default to a 60-second wall budget and enforce per-request deadlines; cleanup and host I/O add overhead. Frames are limited to 1 MiB.

Native replay repeats inputs and, where available, a named cooperative boundary. It does not repeat an entire thread/goroutine schedule. `SIGKILL` retains the host kernel page cache. Native results make no power-loss claim. Runs execute trusted local adapters without a security sandbox; do not run untrusted software or daemonizing adapters. The controller owns the adapter process group; applications that escape it require a different execution boundary.

Artifacts currently remain in local directories. They may contain application data. Cloud upload, team access control, OS/device profile qualification, retention enforcement, packaged distribution and a friendly report UI remain product work.

The [isolated guest experiment](experiments/guest-loss/README.md) explains the additional evidence for whole-guest page-cache loss and its narrower storage assumptions. The sibling build decision documents state the complete product/release decisions and validation gates.
