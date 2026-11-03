# Executed experiments and reproduction

The experiments tested real open-source library implementations through small drivers. They establish feasibility and expose limitations. They did not discover a new upstream defect, integrate a complete customer application, validate market demand, prove concurrency correctness, or calibrate a real power-loss model.

## Environment and subjects

Linux x86_64 under WSL2, kernel `6.18.54.1-microsoft-standard-WSL2`; Rust/Cargo 1.97.1; Go 1.26.4; Python 3; syscall tracing with strace. Data files were under Linux `/tmp`, not on the Windows-mounted workspace. Exact tool output and binary hashes are in [environment.json](evidence/environment.json).

| Subject | Execution | Integration change |
|---|---|---|
| redb 4.4.0 | Release-mode native Rust binary, default file backend | 38-line wrapper; zero upstream implementation edits |
| bbolt 1.4.3 | Native Go binary, ordinary API | 59-line wrapper, 57 nonblank lines; zero upstream implementation edits |
| bbolt 1.3.11 | Additional native binary for same-version baseline comparison | Same wrapper; dependency version changed in an isolated temporary module |
| Gosim `ffd3a613…` + bbolt 1.3.11 | Translated code, Go 1.23.2 | 86-line test, 83 nonblank lines; copied into Gosim's existing bolt example; zero bbolt edits |
| Faultbox `9ed11f9b…` | Native syscall-error injection into both primary binaries | CLI command and baseline driver; zero implementation edits |

Line counts are reproducible file-size observations, not estimates of integration labor. Dependencies and compiler downloads, prototype development and debugging happened within this investigation; no reliable stopwatch log of human-equivalent integration effort exists. Cold builds and customer service integration time were not benchmarked.

## Contract used by A, B and C

Start from a durably initialized `a=0,b=0`. In one transaction update both to `1`, then emit success to the external controller. A recovered state after acknowledged success must be `(1,1)`. An interrupted operation before success may expose `(0,0)` or `(1,1)`, never a mixed pair or unrecoverable store under the declared profile. This is a deliberately small test of acknowledged durability and transaction atomicity, not a general database model.

The workers' unsafe modes use documented durability relaxations: redb `Durability::None`, bbolt `NoSync=true`. The test contract deliberately still demands durable acknowledgment. A failure therefore demonstrates detection of an application-contract/configuration mismatch. It does not imply the libraries violate their own documented promises.

The Go simulation uses the same pair contract and a durable initial directory entry. Its test explicitly expects unsafe mode to lose the update, so `PASS` in its log means the experiment reproduced that loss as expected, not that unsafe mode met the stronger contract.

## A — Rust integration

Executed the real file-backed redb API with durable and relaxed durability settings. For each setting, performed ten independent native runs, waited for the success response, killed the process group without destructors, and reopened the file in a fresh process.

Separately captured positional writes and sync completions from a complete transaction. Reconstructed selected file images at operation boundaries and ran redb's actual recovery code on fresh copies. No Tokio replacement, thread substitution, custom Rust runtime or Wasm compilation was required.

Result: the durable setting survived all ten native kills and all sampled recovery-image checks. The relaxed setting lost the update after all ten native kills. In this version, unsynced transaction data remained in process memory; its trace contained only the opening marker write and sync, rather than transaction data writes. The post-ack storage witness also recovered `(0,0)` and repeated that outcome five times.

Evidence quality: exact effect bytes, file images, operation boundary and recovered values are preserved. A small mismatch is easy to understand. Native thread interleavings and instruction-level replay are uncontrolled; neither the workload nor this experiment covers asynchronous or concurrent application behavior.

## B — Go integration

Repeated the same native-kill and storage-reconstruction workflow against bbolt 1.4.3. Its mmap was read-only; writes used positional I/O. The durable trace had three data/meta writes and two sync completions. The relaxed trace had three writes and zero sync completions.

Result: both settings returned `(1,1)` after ten native process kills each. The relaxed configuration lost the acknowledged update under the drop-unsynced reconstruction; that witness recovered `(0,0)` in five of five replays. A selected pre-ack image retaining the metadata write without its unsynced data/freelist writes triggered a recovery panic. This is an expected consequence of relaxed sync assumptions under this model, not an upstream bug report.

Native goroutine scheduling, preemption, mutexes/channels/atomics, map iteration, timers, entropy and kernel scheduling were not controlled. The serialized workload reduces their relevance to this contract but does not remove them in general. No result establishes deterministic replay of a Go application.

The bbolt 1.3.11 native comparison also retained relaxed-mode data after ten process kills while losing it under reconstruction. This removes a dependency-version difference as the explanation for Gosim's contrasting result.

## Primary measured results

| Library/configuration | Native loss after success | Captured writes / syncs | Unique image + acknowledgment-context checks | Pre-ack violations | Post-ack violations |
|---|---:|---:|---:|---:|---:|
| redb 4.4.0 durable | 0 / 10 | 6 / 2 | 12 | 0 | 0 |
| redb 4.4.0 relaxed | 10 / 10 | 1 / 1 | 3 | 0 | 1 |
| bbolt 1.4.3 durable | 0 / 10 | 3 / 2 | 6 | 0 | 0 |
| bbolt 1.4.3 relaxed | 0 / 10 | 3 / 0 | 8 | 1 | 1 |

The context count deduplicates `(image hash, acknowledged)` pairs; the same bytes checked before and after acknowledgment are different contract checks. It is not a count of all possible crash states. Safe configurations completed 18 such sampled checks without a violation. Results include duplicated candidate records pointing back to a checked context.

The saved primary campaign took 0.315 seconds for both redb settings and 0.189 seconds for both bbolt settings on a warm local environment, excluding builds. Other runs took longer. These tiny-workload observations are not representative service benchmarks, cloud-cost inputs or comparative speed claims. [Raw campaign](evidence/native-campaign/results.json), [same-version native baseline](evidence/native-bbolt-1.3.11.json).

## C — Shared verification model

One Python predicate checks the same allowed pair states for both languages. The capture parser emits storage effects without requiring a task/goroutine abstraction. Four exported artifacts share contract identity, subject/configuration, execution profile, search limits, outcome, witness and replay scope. [Compatible artifacts](evidence/compatible-artifacts.ndjson).

Measured conclusion: sharing a contract and evidence envelope is straightforward at this boundary. A universal runtime-effect language is unnecessary. This does not establish common scheduling, arbitrary transaction models, or that a Rust service and Go service can share in-process deterministic simulation. Native multi-service scenarios are architecturally compatible but were not executed.

## D — Existing-tool baseline

**Gosim:** used the project's existing bbolt integration, adding only the pair/recovery scenario. Across seeds 1–5, the durable configuration recovered `(1,1)` and relaxed mode recovered `(0,0)`. The full five-seed command took 8.073 seconds in the saved run, including CLI/build/translation overhead; reported test execution was 0.018 seconds. That is not comparable to the native probe's campaign time. [Pinned log](evidence/adversary-gosim-pinned.log).

The same checkout under installed Go 1.26.4 failed with `missing function body pkg=internal/runtime/syscall/linux name=Syscall6` in 3.895 seconds. A cgo-disabled attempt failed because this CLI uses go-sqlite3. No conclusion about other Gosim forks follows. Initial cold compilation attempts hit experimental time budgets; subsequent cached pinned runs succeeded. [Toolchain comparison](evidence/adversary-gosim-results.json).

**Faultbox:** compiled the current pinned implementation and denied every `fdatasync` in each primary native binary. Interception logs showed `EIO`; neither worker emitted success. This proves useful native syscall fault injection is already available. It tests error handling, not an fsync-honoring power-loss outcome. One Go invocation printed a panic while the Faultbox supervisor returned exit code 0 in this environment; the experiment therefore uses explicit worker output and the trace as evidence instead of treating supervisor exit status as the behavioral oracle. This observation was not investigated as an upstream defect. [Results and logs](evidence/faultbox-baseline/results.json).

**Go standard library:** the separate synctest probe starts sixteen timer-driven goroutines in each of fifty otherwise identical bubbles. It observed fifty distinct receive orders. Fake time is valuable but did not yield schedule replay in this experiment. [Source](prototypes/synctest-probe/probe_test.go).

Antithesis and Workers IO were assessed from primary documentation and customer evidence, not executed. The best demonstrated free baseline for the Go durability outcome is Gosim. Adversary has shown no bug-finding advantage over it.

## E — Fault-model fidelity

Executed three kinds of check:

1. Native process kill versus storage-loss reconstruction demonstrated an actual semantic distinction, particularly for both bbolt versions.
2. Reconstructing every captured write in order produced the same recovered values as the corresponding native execution for all four primary configurations.
3. Four focused fidelity/contract tests passed: synced bytes are preserved; pending and acknowledged operations differ; empty/truncated traces are rejected; writable shared mappings are rejected. A recovery timeout is represented as inconclusive.

These checks do not establish physical fidelity. The probe models one existing file and treats successful syncs as committing all prior captured file effects. Its sector samples retain the first or last 512 bytes of an unsynced write; they are selected examples, not full sector-subset enumeration. It excludes namespace durability, file-descriptor lifecycle generalization, arbitrary mmap writes, io_uring, direct I/O, partial syscalls and cross-file dependencies. An empty trace guard cannot establish observation completeness for an arbitrary application whose effects bypass tracing.

A first parser attempt mistakenly produced empty effect lists because of strace string-length and escaped-path handling. Those results were discarded. The delivered parser decodes matching path representation, captures full buffers, rejects malformed effects and fails on empty traces. This correction is why observation completeness must be a product concern, not an implicit assumption.

**Fidelity verdict: not yet qualified for customer power-loss claims.** The demonstrated mismatch is useful under the stated model, and independently consistent with Gosim. No real VM power-cut or filesystem-level block replay campaign was executed. The environment has QEMU/KVM and device-mapper utilities, but no prepared disposable guest/profile/replay-log setup was established; no host block devices were changed. This was an unperformed validation step, not a fabricated environment impossibility.

The next experiment is executable after provisioning a disposable Linux guest: install a kernel with `CONFIG_DM_LOG_WRITES`, build the kernel-documented `replay-log` utility, attach two blank job-owned virtual disks, then perform the following inside that guest. Paths below deliberately refer only to those disposable guest disks; verify their identity before running.

```bash
# Prerequisite: /dev/vdb is the blank test disk, /dev/vdc the blank log disk.
# Never run against host or production disks. Use the kernel documentation's
# log-writes setup/replay commands in a fresh disposable guest.
dmsetup targets                         # verify log-writes support
blockdev --getsz /dev/vdb               # use this verified size in the table
dmsetup create advlog --table '0 VERIFIED_SECTORS log-writes /dev/vdb /dev/vdc'
mkfs.ext4 -F /dev/mapper/advlog
mkdir -p /mnt/advtest
mount -o data=ordered /dev/mapper/advlog /mnt/advtest
dmsetup message advlog 0 mark baseline
# Run the driver with its data directory on /mnt/advtest and its acknowledgment
# journal outside the test volume. Mark selected operation milestones without
# syncing application files. Complete capture and export the log.
# Replay fresh baseline images to selected marks/prefixes using replay-log;
# boot/mount each fresh recovery image and run the same recovery checker.
```

`VERIFIED_SECTORS` is an explicit provisioning input, not a ready-to-run constant. The complete command forms, replay flags and ordering semantics come from the [kernel's executable examples](https://docs.kernel.org/admin-guide/device-mapper/log-writes.html). The unresolved engineering work is correlating acknowledgment cuts and flush/FUA completions without adding persistence, and producing a fresh cache-free recovery environment. Script that as a separate runner, then calibrate: unsynced overwrite, successful file sync, file creation plus directory sync, rename/checkpoint replacement, truncation/growth, partial write/error, and crash during recovery. Compare permitted image/outcome sets, not one sample. Fail qualification if the model admits an impossible violation or omits a known allowed failure.

## Reproduction

Use [prototype instructions](prototypes/README.md). The primary end-to-end command is:

```bash
bash prototypes/run.sh
```

The Gosim baseline is independent:

```bash
bash prototypes/run-gosim.sh
```

The scripts build pinned dependencies and create fresh temporary result directories. Dependency downloads require network access. Saved witness images can be replayed using `replay.py`; replay copies the image before each attempt because recovery may modify it. No prototype performs privileged block-device operations.

