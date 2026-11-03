# Backend selection and qualification

**A backend is selected by required semantics, not by language or the strongest replay label available.** An unsupported requirement stops the run. Native testing and simulated execution can coexist in the product without pretending to provide identical evidence.

## Fixed support progression

| Backend/profile | Status | Supported meaning | Important exclusions |
|---|---|---|---|
| `native-process` / native-process-linux-v1 | Implemented preview | Actual adapter/SUT process group; terminate after acknowledgment or named checkpoint; fresh process recovery | Host page-cache loss, schedule determinism, escaped daemons, external persistent dependencies |
| `linux-guest-loss` / qemu-guest-loss-v1 | Separate executed experiment; next CLI integration | Lose a Linux guest's memory/page cache; retain host-backed disk writes; recover in fresh kernel | Volatile device-cache loss/reordering, arbitrary tears, host power outage, deterministic instruction stream |
| Native syscall faults using pinned Faultbox | Integration backlog; prior direct injection test succeeded | Selected real error/delay/hold semantics, confirmed from execution events and application observations | Denied fsync is not successful fsync then power loss; supervisor exit alone is not the oracle |
| Turmoil profile | Optional Rust adapter after beta | Compatible async code with declared network/filesystem shims and pinned dependency closure | Bypassed native I/O/threads/FFI; real filesystem guarantee from an abstract model |
| Gosim profile | Optional pinned Go adapter after beta | Translated Go and supported simulated Linux effects under a qualified toolchain | Arbitrary current Go compatibility, unsupported cgo/unsafe/syscalls; unmodified native binary execution |
| Loom/Shuttle/Porcupine/Elle/Kani | Targeted checker/integration extensions | Each tool's declared problem and evidence class | Automatic whole-application proof |

Rust and Go use the same native path. Native binaries can retain their existing runtime; an adapter supplies application operations and observations. Do not promise that adding the SDK controls every existing dependency.

The release should package native support first, then a pinned guest image profile with a small control agent. The experimental minimal initramfs establishes feasibility; it is not the final arbitrary-application guest packaging. A release guest needs the application's loader/libraries, reproducible image construction, lifecycle/secret isolation, quotas and reliable artifact export.

## New evidence: actual guest-state loss

The experiment booted a real Linux kernel and ext4 filesystem under QEMU/KVM, waited for a guest acknowledgment, then sent host `SIGKILL` to QEMU and waited for it to terminate. It booted a fresh guest for recovery. No host disk devices, loop devices or root changes were involved. This improves on the previous single-file simulated reconstruction by exercising the real filesystem and kernel.

| Case | First campaign | Reproducible-script campaign | Meaning |
|---|---:|---:|---|
| Unsynced overwrite of existing file | Previous value in 3/3 | Previous value in 1/1 | Guest page-cache loss was exercised |
| File sync before acknowledgment | New value in 3/3 | New value in 1/1 | This synchronized overwrite survived sampled guest failures |
| Write temp, sync, rename, sync directory | New value in 3/3 | New value in 1/1 | This complete replacement protocol survived sampled guest failures |
| bbolt 1.4.3, `NoSync` | Previous pair in 3/3 | Previous pair in 1/1 | Real acknowledged data loss under deliberately relaxed durability |
| bbolt 1.4.3, durable default | New pair in 3/3 | New pair in 1/1 | Acknowledged data survived these samples |

The earlier native process-kill test retained bbolt's relaxed update. The difference comes from losing guest kernel state, not from changing the library or replacing filesystem semantics. These are documented relaxed settings, not new bbolt defects. Raw results and logs are included in the evidence directory.

Tested setup: Alpine kernel 6.18.58-0-virt, QEMU 11.0.1, one vCPU, 256 MiB memory, 64 MiB raw test disk, ext4 defaults, `cache=none,aio=threads`. Boot assets have fixed SHA256 values. The standalone reproducible script retains pre-recovery witness images. Both campaigns used fresh recovery kernels. Repetition counts describe observations, not the full set of allowed filesystem/device outcomes.

QEMU documents that its cache modes separately configure writeback, direct I/O and flush handling. Its graceful QMP `quit` also flushes caches, so the experiment used abrupt host termination. Neither this configuration nor a seed provides full execution determinism. [Cache modes](https://www.qemu.org/docs/master/system/invocation.html), [graceful quit behavior](https://www.qemu.org/docs/master/interop/live-block-operations.html).

## Exact crash/fault distinctions

- **Process termination:** loses process memory, skips cleanup, preserves kernel/device state and other processes. Owned process group must terminate before recovery starts.
- **Guest volatile-state loss:** loses the entire guest's volatile state and page cache. The host-backed virtual disk survives; writes reaching it may survive. This is a concrete machine-failure subset, not all hardware power failures.
- **I/O error:** caller observes an error or partial result at a declared boundary. The operation's logical effect may be uncertain. A simulated `EIO` is not a disk-cache model.
- **Device-cache loss:** requires an independently modeled volatile writeback layer, including completed flush/FUA obligations. Not implemented.
- **Torn writes/corruption:** require a declared atomicity granularity and specific permitted target/version semantics. Not enabled by inventing arbitrary byte corruption.
- **Network fault:** must identify endpoints, messages/connections and failure direction; client timeout can leave an operation committed. Later backend work.

Keep environment guarantees and application promises separate. A successful sync must not be discarded in a profile that promises honest persistence. If a profile models lying hardware, label that different assumption and evaluate an application contract that actually promises tolerance of it.

## Qualification requirements before advertising guest support

The following is the release gate, not a claim that three smoke tests complete it:

1. Record kernel, filesystem features/mount options, e2fsprogs, QEMU, guest image, binary/library closure, virtual device/cache settings and host constraints.
2. Run litmus cases for overwrite, append/extend/truncate, new file plus parent directory, rename replacement, unlink, repeated crash during recovery, successful sync and interrupted sync/error.
3. Verify complete baseline initialization and externally recorded acknowledgment without adding an application sync at the failure boundary.
4. Confirm capture termination before freezing the disk. Preserve an immutable original and recover only clones. Validate artifact hashes and crash confirmation.
5. Compare against known allowed/forbidden outcomes; absence of a particular unsynced loss is not an error because unsynced data may survive.
6. Detect missing recovery observations, guest boot failure, dropped serial/control frames, unconfirmed crash, disk-full host and timeout. These must be inconclusive/tool failure.
7. Test the same profile on a clean runner and through a real application persistence change. Separate host failures from SUT failures.

Guest support is qualified only for its finite documented profile. Adding another kernel, mount mode, device cache policy, persistent mmap path or direct-I/O use creates qualification work. The filesystem literature shows why one abstract file-drop rule cannot cover them all. [ALICE/BOB](https://www.usenix.org/system/files/conference/osdi14/osdi14-paper-pillai.pdf), [Ferrite](https://dl.acm.org/doi/10.1145/2980024.2872406).

The Linux `dm-log-writes` target remains a useful future way to explore persistence cuts without treating writes as independent. Its completed-write, flush and FUA ordering must be respected. It is absent from this host kernel but enabled in the downloaded guest kernel; that fact resolves deployment feasibility, not end-to-end replay semantics. [Kernel semantics](https://docs.kernel.org/admin-guide/device-mapper/log-writes.html).

## Simulator qualification policy

One supported version combination per adapter initially. Pin compiler, simulator revision, dependency graph, build flags, seeds and test entry points. Maintain behavioral conformance tests for timers, randomness, synchronization, network, storage and reboot semantics actually claimed. Unsupported effects stop the campaign rather than escaping silently.

The prior experiment's Gosim checkout worked with Go 1.23.2 and failed to translate under installed Go 1.26.4. Use that known combination as experimental support, not a promise of current native-Go compatibility. Turmoil's inspected workspace includes unstable storage simulation, while the inspected upstream MadSim power-failure routine is incomplete. Adapter support follows tested code paths, not README category labels. The earlier research report retains exact source revisions and evidence.

Do not adopt Wasm to avoid these questions. It changes compilation/dependency boundaries and still requires qualified host effects. Do not add a custom scheduler, DPOR or VM implementation until a paid customer requirement and a controlled comparison justify it.
