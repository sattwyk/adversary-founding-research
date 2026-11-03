# Technical research report

Research date: 3 November 2026. The investigation used Exa for 81 requested search-result entries across seven workstreams: engineering practice, Rust integration, Go integration, persistence research, exploration, competitors, and economics. This count includes overlapping search entries, not 81 independent validated sources. Repository inspection covered 17 pinned checkouts. The [repository manifest](evidence/repository_manifest.json) records exact revisions; [code inspection evidence](evidence/code_inspection.json) records selected inspected functions. Some repositories used sparse checkouts. None received a comprehensive audit.

Evidence labels used throughout: **observed** means inspected source or an executed experiment; **reported** means an attributed primary-source statement; **inference** means a proposed conclusion from that evidence. Public documentation describes capability, not independently measured performance or customer willingness to pay. There were no customer interviews, vendor trials requiring private access, paid benchmarks, or new upstream bug claims.

## The expensive work is maintaining the relationship between promises and failures

Engineers repeatedly build six connected pieces: controllable effect boundaries, realistic dependency/fault models, workloads that reach interesting states, application-specific oracles, reconciliation of incomplete operations, and preserved debugging/regression artifacts. A simulator addresses part of the first two. The rest remains even with an excellent execution environment.

**Observed:** etcd separates traffic, failure injection, history reporting, model validation, persisted-request reconciliation, and visualization. Its validator explicitly checks assumptions before linearizability and other checks. A failed client request can require evidence from persistence or watches to determine whether it took effect. This is more specific than an ordinary reference map. [Validator](https://github.com/etcd-io/etcd/blob/4b0b745025716b893c64dbee443e3f969931a16b/tests/robustness/validate/validate.go#L32), [history reconciliation](https://github.com/etcd-io/etcd/blob/4b0b745025716b893c64dbee443e3f969931a16b/tests/robustness/validate/patch_history.go#L34).

**Inference:** the most promising product surface is that maintained relationship between a customer's guarantees, its test workloads, and interpretable recovery evidence. There is no evidence here that generic fault injection is an unserved need, or that this relationship is necessarily a separately purchasable product.

## What production-oriented implementations actually control

| System | Inspected implementation and observation | Integration and trust implications |
|---|---|---|
| FoundationDB | `Sim2FileSystem::open` composes `SimpleFile`, `AsyncFileNonDurable`, detachable handles and optional chaos. It asserts restrictions on ordinary file creation. Machine termination has explicit kill types. [Source](https://github.com/apple/foundationdb/blob/8ddaf495d9eb5efe101a7554547bd51df3504e10/fdbrpc/sim2.cpp#L3055) | Real database logic runs against modeled effects in a purpose-designed runtime. Its simulator is not a library that makes an arbitrary existing database deterministic. The layering matters: a `SimpleFile::sync` comment in isolation does not describe the full durability model. |
| TigerBeetle | Testing storage separates read/write/crash faults, latency and fault-atlas restrictions. `Storage.reset` cancels pending operations and can corrupt their target sectors; writes reject concurrent overlapping operations. [Source](https://github.com/tigerbeetle/tigerbeetle/blob/0450c32b2c3c3ab6114f3402cc601341cc8192aa/src/testing/storage.zig#L274) | Strong, protocol-specific fault assumptions deliberately preserve recoverability conditions. Copying the corruption probability without those assumptions would be misleading. Production logic and the environment adapter are different things. |
| etcd | Native cluster robustness harness plus specialized history/model validation and persisted-request evidence. [Harness](https://github.com/etcd-io/etcd/tree/4b0b745025716b893c64dbee443e3f969931a16b/tests/robustness) | Real processes are practical; scheduler determinism is unnecessary for useful violations. Custom operations and response uncertainty account for substantial code. |
| RisingWave | Simulation cluster setup and restart orchestration are application-specific. Workspace dependencies pin MadSim packages and forks, including Tokio, tonic and Kafka-related integration. [Dependency configuration](https://github.com/risingwavelabs/risingwave/blob/f5a3e57a60263595a950678c5c5566dbd5dc3166/Cargo.toml#L204), [cluster](https://github.com/risingwavelabs/risingwave/blob/f5a3e57a60263595a950678c5c5566dbd5dc3166/src/tests/simulation/src/cluster.rs#L403) | Runtime substitution has a dependency-maintenance cost. The SQL/materialized-view tests inspect real application behavior, but modeled external services do not validate those services' implementations. |
| WarpStream | Customer engineering account describes packaging agents, control plane, clients, KV store, Postgres and object-store substitute together, plus custom workloads and assertions. [Account](https://www.warpstream.com/blog/deterministic-simulation-testing-for-our-entire-saas) | Reported use of Antithesis directly contradicts a thesis that existing tools cannot test a whole SaaS stack. The reported speedup is workload-specific and was not benchmarked here. Replacing a cloud service with localstack changes the boundary. |

FoundationDB's paper describes deterministic database code, modeled network/disk/time/randomness, composable workloads, assertions, and recovery checks after healing failures. That is a coherent architecture designed around testing, not evidence that every existing threaded system can be dropped into the same runtime. [FoundationDB paper](https://www.foundationdb.org/files/fdb-paper.pdf).

TigerBeetle's newer discussion adds protocol-aware checks and physical state convergence. This reinforces an important limitation of generic checkers: a system-specific invariant can reveal failures that a black-box key/value history cannot. [Protocol-aware DST](https://tigerbeetle.com/blog/2026-08-20-protocol-aware-dst).

## Rust integration reality

**Tokio:** async task scheduling, timer advancement, networking and filesystem effects are distinct control surfaces. A current-thread executor alone does not control `std::thread`, `spawn_blocking`, native synchronization, OS entropy, external processes, or a storage library's synchronous file calls. Even seeded task scheduling does not model Rust's relaxed atomic behaviors.

**Turmoil:** the inspected 0.7.2 workspace has per-host Tokio runtimes/`LocalSet`s and separate network/filesystem crates. The filesystem feature is explicitly unstable. Its implementation distinguishes file data and directory durability and supports configurable torn writes. This is materially more capable than an old description of Turmoil as network-only. Integration still requires using its shim types or an application-supported backend; transitive `std::fs` calls do not automatically switch. [Runtime](https://github.com/tokio-rs/turmoil/blob/6a1f50223b5a1884bf8890053c85eec04a8af267/crates/turmoil/src/rt.rs), [filesystem crash](https://github.com/tokio-rs/turmoil/blob/6a1f50223b5a1884bf8890053c85eec04a8af267/crates/turmoil-fs/src/lib.rs#L1104), [sync semantics](https://github.com/tokio-rs/turmoil/blob/6a1f50223b5a1884bf8890053c85eec04a8af267/crates/turmoil-fs/src/lib.rs#L1636).

**MadSim:** conditional exports substitute selected Tokio-shaped APIs. In the inspected upstream checkout, `madsim-tokio` still reexports real `tokio::fs`, while the separate simulated filesystem has an empty `power_fail` implementation. This particular code cannot establish a local-disk crash-consistency claim merely by enabling `cfg(madsim)`. RisingWave uses a separately pinned fork and external-service models; do not transfer this upstream finding to every fork. [Conditional API](https://github.com/madsim-rs/madsim/blob/519950efb4711464f300ed7edf2967ed62d5f502/madsim-tokio/src/lib.rs), [power-failure implementation](https://github.com/madsim-rs/madsim/blob/519950efb4711464f300ed7edf2967ed62d5f502/madsim/src/sim/fs.rs#L50).

**Shuttle:** an explicit scheduler chooses runnable tasks and random values; replay schedules are first-class. Its standard-library wrappers make the supported instrumentation boundary visible. It is useful for production algorithms compiled against controlled primitives, with harness effort and an escape boundary. It is not general storage or network virtualization. [Scheduler](https://github.com/awslabs/shuttle/blob/c98a83644919ad94c0e45be9ec4d72a5c2bfecaa/shuttle-engine/src/scheduler/mod.rs#L75).

**Loom:** the runtime maintains dependency accesses, vector clocks and backtracking for DPOR; atomic modeling is separate from merely scheduling threads. It is appropriate for small concurrent components compiled with Loom primitives, subject to configured bounds and its memory-model scope. Using it as an end-to-end database runner would expand state space and fail to control external effects. [DPOR implementation](https://github.com/tokio-rs/loom/blob/948c8cc78b178ede6eeff3afc7d97f2f4ea08559/src/rt/execution.rs#L135).

**Measured redb integration:** no source changes to redb; ordinary file-backed API and a 38-line wrapper. Its public `StorageBackend` also offers a natural optional white-box boundary. Existing redb tests already model a crash during file growth, including durable length versus header ordering—evidence that mature engines already build specialized recovery tests. [Backend interface](https://github.com/cberner/redb/blob/edbbbe3aef08f8ab45786ba7442247d8fd7dfae7/src/db.rs#L72), [growth regression](https://github.com/cberner/redb/blob/edbbbe3aef08f8ab45786ba7442247d8fd7dfae7/tests/crash_consistency.rs). The crate release used experimentally is pinned separately by Cargo.lock; a checkout revision is not a claim of byte identity with that release.

**Wasmtime/WASI:** a plausible controlled host-call boundary for components already targeting Wasm, but it changes the compilation target and dependency environment. Host clocks, entropy, filesystem, networking and scheduling still need deliberate control. Native threads, mmap-heavy storage, FFI and OS-specific behavior require a compatibility investigation. No Wasm prototype or performance result was obtained here. It does not minimize initial integration complexity for the selected native libraries. [Wasmtime interface overview](https://docs.wasmtime.dev/).

## Go integration reality

**Native Go:** goroutine scheduling includes runtime preemption and synchronization behavior beyond a process supervisor. `GOMAXPROCS=1` does not eliminate map iteration, `select` choice, timers, entropy, syscalls or asynchronous events. Production atomics, mutexes and channels remain native in the probes; no schedule replay claim is made.

**Gosim:** source translation replaces goroutines, maps, synchronization and standard-library/runtime hooks; a simulated Linux layer supplies network and filesystem behavior. The filesystem explicitly supports read-only mmap, enough for bbolt, and retains a persistence dependency graph for partial crash states. This is actual Go component code with a substituted environment, not an untouched production binary. Pointer-dependent behavior remains a concern. [Design](https://github.com/jellevandenhooff/gosim/blob/ffd3a613542675755e4cbf8186b5edaf404ed95c/docs/design.md), [filesystem](https://github.com/jellevandenhooff/gosim/blob/ffd3a613542675755e4cbf8186b5edaf404ed95c/internal/simulation/fs/filesystem.go#L21), [machine crash/restart API](https://github.com/jellevandenhooff/gosim/blob/ffd3a613542675755e4cbf8186b5edaf404ed95c/machine.go#L73).

**Measured:** the recovery contract ran across seeds 1–5 with Go 1.23.2 and Gosim's bbolt 1.3.11 dependency. Its durable configuration recovered `(1,1)`; `NoSync` recovered `(0,0)`. The same checkout failed under Go 1.26.4 at `internal/runtime/syscall/linux.Syscall6` translation. Disabling cgo also failed because this CLI uses sqlite3. These are version/environment observations, not claims that every Gosim fork or future release has the same limitations. No runtime repair was attempted.

**`testing/synctest`:** fake time and durable-blocking detection improve ordinary concurrency tests. The official API explicitly distinguishes syscalls/I/O and mutex blocking from operations that can advance its fake clock. It supplies no power-loss model. [Official package documentation](https://pkg.go.dev/testing/synctest). In the executed probe, 50 identical 16-goroutine timer workloads produced 50 distinct receive orders. This shows uncontrolled ordering in this setup; it is not a statistical characterization of all Go scheduling.

**Source instrumentation:** useful when it reuses a maintained translator such as Gosim. A startup-owned rewrite of every Go/runtime/standard-library boundary would turn toolchain compatibility into its core engineering burden. The measured version failure is a concrete reason to avoid that commitment.

## Checkers and formal tools: reuse their good solutions

| Tool | Observation | Correct use and limit |
|---|---|---|
| Jepsen | `run-case!` composes clients, nemesis, generators and a stored history; analysis is a distinct phase. [Code](https://github.com/jepsen-io/jepsen/blob/08e3e33964808876c2c9bbe4f3f9f948295a108e/jepsen/src/jepsen/core.clj#L236) | Tests actual systems with specialized workloads. Native execution remains nondeterministic; contracts and setup are real integration work. The framework is not limited to one-off consultancy even though commercial analyses are often engagements. |
| Elle | An analyzer constructs dependency graphs, strongly connected components, cycle explanations and anomalies. [Code](https://github.com/jepsen-io/elle/blob/4e359e597c52938b0afebb8bc42273914d68a424/src/elle/core.clj#L519) | Use supported observable transaction histories. Missing visibility limits what can be inferred; this is not a checker for arbitrary internal state. |
| Porcupine | Models use pure `Init`/`Step` functions, operation intervals and optional justified partitioning. Results include `Unknown`, `Ok`, `Illegal`. [Model/API](https://github.com/anishathalye/porcupine/blob/97cd067defd99df172053ba7f4ca5bbae57fe446/model.go#L101) | Reuse for finite-history linearizability. `Ok` for that history does not prove all executions; `Unknown` must stay inconclusive in Adversary. Partitioning transactions by individual keys can be unsound. |
| Stateright | Explicit model state/actions/transitions plus BFS/DFS/simulation checkers and discoveries. [Model](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/lib.rs#L158), [search](https://github.com/stateright/stateright/blob/ab8c8be9341505e0f71edbe5dd88ed275bd976a4/src/checker.rs) | Reuse for protocols or model-driven components. A model matching production implementation is a separate obligation. |
| Kani | Rust verification library exposes proof-related APIs, modeled futures and concrete playback; it requires a specialized compiler/toolchain. [Library](https://github.com/model-checking/kani/blob/c5e65fdfc3e15ec32293f2a4c07509aebec033c9/library/kani/src/lib.rs) | Good candidate for bounded helpers and storage-model primitives. No Kani proof was executed here. Do not equate a bounded harness result with an OS/storage durability proof. |

## Persistence fidelity determines whether a violation is meaningful

The ALICE/BOB study shows that application crash-consistency protocols depend on filesystem persistence properties that differ by implementation. Ferrite formalizes crash-consistency models using allowed/forbidden behaviors and explicit ordering assumptions. These sources directly undermine a generic rule that every unsynced operation may be independently dropped in every environment. [ALICE/BOB](https://www.usenix.org/system/files/conference/osdi14/osdi14-paper-pillai.pdf), [Ferrite](https://dl.acm.org/doi/10.1145/2980024.2872406).

CrashMonkey and ACE demonstrate bounded black-box crash testing with an existing block record/replay mechanism and short workload generation. Their results motivate small workloads and systematic cuts, not a claim that a small bound covers all application bugs. The approach tests filesystem behavior; adapting it to a customer's durability promise still requires an application oracle. [OSDI paper](https://www.usenix.org/conference/osdi18/presentation/mohan), [ACE workload bounds and adapter](https://github.com/utsaslab/crashmonkey/blob/master/docs/Ace.md).

Pathfinder's representative application crash testing and persistence graphs are relevant later work. Its premise makes a simple uniqueness claim about persistence-aware exploration untenable. We reviewed the primary paper description, not its implementation or performance independently. [Pathfinder paper](https://dl.acm.org/doi/full/10.1145/3720431).

The executed file reconstruction is deliberately narrower: a pre-existing single file, successful `pwrite64` effects, completed sync barriers, selected whole-effect and 512-byte edge samples, and actual library recovery. It excludes file creation/rename/unlink durability, general writable mappings, direct I/O and device-level behavior. It does not model the real filesystem. Its full-retain reconstructions agreed with native recovered values in all four primary configurations. That is a useful consistency check, not physical calibration.

Potential false positives: overpermissive reordered writes or sector tears that the chosen filesystem/device forbids; unsupported effects silently outside the trace; wrong acknowledgment contract; oracle error. Potential false negatives: preserving too much data; ignoring directory updates, mid-syscall failure, mmap or io_uring effects; testing one short path; omitting realistic dependencies, recovery failures, or repeated crashes. Zero safe-configuration failures in this campaign do not resolve those risks.

**Conclusion:** sufficient for an architecture experiment; insufficient for a production power-loss claim. A calibrated real-filesystem/block runner is a required next gate.

## Exploration: start with informative small failures

| Strategy | Initial decision | Reason and prerequisite |
|---|---|---|
| Seeded workload generation | Build | Cheap, useful for short batches, checkpoints, deletes, reopen and retry shapes; preserve exact operations as well as seed |
| Failure-oriented sampling | Build | Bias cuts around persistence boundaries and recovery transitions; verify faults actually landed |
| Bounded enumeration | Add for a genuinely finite qualified profile | Can support a bounded claim only if workload/effect/outcome domain and completeness are explicit |
| Image hashing | Build narrowly | Avoid recovering identical byte images under the same contract/profile; hidden runtime-state hashing is a separate problem |
| Counterexample reduction | Build early | User value comes from a small causal failure; budget replays and retain a reproduction frequency |
| Coverage-guided search | Defer until basic feedback exists | Prefer semantic milestones first; code-coverage novelty is useful but not itself correctness coverage |
| DPOR and sleep sets | Defer | Need sound independence and enabling information, controlled scheduling and replay; raw black-box operation names are inadequate |
| Checkpointed branching | Reuse storage snapshots; defer arbitrary process checkpoints | Disk-state replay is simpler than controlling the full runtime, but does not replay hidden concurrency |

DPOR dynamically records dependencies and introduces backtracking points to avoid equivalent schedules; the original work and subsequent optimal/source-set algorithms make explicit assumptions about the transition system. They do not justify pruning transactions because they touch different logical keys. [Original DPOR](https://users.soe.ucsc.edu/~cormac/papers/popl05.pdf), [source sets and wakeup trees](https://dl.acm.org/doi/10.1145/2535838.2535845). Loom's real dependency tracking illustrates the implementation burden. For a native storage product, wrong pruning is worse than redundant testing. Persistence dependencies deserve their own graph rather than being mislabeled concurrency DPOR.

Use a staged budget: deterministic known regressions first; short generated workloads and persistence cuts second; a small reserve for new shapes and rare recovery states. Measure distinct recovery outcomes, validated persistence boundaries and minimized failures. Do not market a number of seeds as a percentage of correctness coverage.

## Answers to the product's high-risk questions

| Question | Finding |
|---|---|
| What do teams rebuild? | Models, uncertain-operation handling, effect adapters, failure workloads, fault restrictions, replay/debugging, and regressions |
| Greatest engineering effort? | Inference from source: maintaining faithful boundaries and application oracles across code/dependency changes. No cross-company labor study was performed. |
| Which approaches are intrusive? | Retrofitting complete in-process simulation into code using unmediated threads/I/O/dependencies. Wrappers can be small when a suitable interface already exists. |
| Can actual production code run? | Yes: unchanged native libraries measured; production logic under adapters observed. Identical native execution and modeled-environment execution are different evidence. |
| What remains uncontrolled natively? | Scheduling, clocks/entropy, runtime implementation, memory ordering, kernel/device behavior, and undeclared dependencies |
| What is already well served? | Small-component concurrency checking, finite-history checking, controlled Go simulation, black-box native fault injection, and broad commercial DST |
| Is a custom simulator necessary? | No for the selected workflow. A qualified storage runner is still integration/validation work; it is not free. |
| What advantage is plausible? | Reduced time to a maintained customer recovery contract and an actionable regression. Not demonstrated as a commercial advantage yet. |

## Commercial counterevidence

Antithesis' founder offering prices small engagements in the few-thousand-dollar monthly range and describes a transition from high-touch services toward self-service. This weakens a strategy based solely on serving smaller teams. [Founder package](https://antithesis.com/blog/2025/antithesis_for_founders/).

Workers IO offers current documentation for block, syscall, network and service faults triggered at named occurrences. Its pricing page lists $2,500/month, 2,500 included actual simulation runtime hours and $3.60/hour overage; a third-party listing returned a different price and was rejected in favor of the primary source. Its deterministic-VM and reproduction claims were not tested here. [Fault models](https://workers.io/docs/fault-models), [pricing](https://workers.io/pricing/).

Faultbox's inspected implementation and actual native injection test show a capable open alternative. Its determinism documentation caps current behavior at mediated-event level, including under its netstack mode, and explicitly notes filesystem observation versus fault-datapath limits. Do not compare it to a toy chaos script. [Determinism contract](https://github.com/faultbox/Faultbox/blob/9ed11f9b90036904c069a4e82e04503e56842f51/docs/determinism.md), [filesystem observation example](https://github.com/faultbox/Faultbox/blob/9ed11f9b90036904c069a4e82e04503e56842f51/poc/gvisor-rfc056/faultbox.star).

Jepsen's published analysis record and customer-authored WarpStream account establish demand for serious correctness work. They reveal neither Adversary demand nor comparable contract prices. No defensible TAM estimate follows from them. [Jepsen analyses](https://jepsen.io/analyses).

The appropriate commercial experiment is a scoped paid integration, benchmarked against using the best existing tool with competent assistance. If that tool is sufficient and cheaper to maintain, the correct decision is to use it.

