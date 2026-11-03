# Adversary V1: decisions to build against

Decision date: 3 November 2026. This document adopts the developer-first platform direction: free local tooling, thin language integrations, honest correctness evidence and a persistence/recovery workflow first. Paid integration is onboarding and discovery, not the product definition.

**Build one local Rust CLI with a common adapter protocol, small Rust/Go SDKs and versioned correctness contracts. Ship native process-crash testing first. Add a separately qualified whole-guest-loss backend next. Reuse in-process simulators through explicit adapters once the native workflow is usable.**

The architecture no longer requires choosing between one universal simulator and unrelated language tools. The common layer owns user intent, campaigns, evidence and regression workflow; execution adapters own the behavior they actually control.

## Closed construction decisions

| Question | Decision | Why / implementation consequence |
|---|---|---|
| What is the first user? | Engineer changing persistence or recovery in an infrastructure component/service | One expensive release question and an inspectable boundary; not every Rust/Go developer |
| First contract? | Acknowledged atomic batches survive recovery; interrupted batches may wholly commit or abort | Covers durability and application integration; common meaning in both languages |
| First workload shape? | Finite sequential operations on a small declared state, then crash/recover/observe | Makes the oracle trustworthy before adding concurrent histories |
| What constitutes success? | Client-visible accepted response, under the declared application contract | A transaction returning internally is not automatically a remote client's acknowledgment; adapters must identify that boundary |
| Default execution? | Actual native Linux processes and actual library/application code | Low integration cost, preserves production runtime behavior, no scheduler rewrite |
| Precise crash placement? | Optional cooperative SDK checkpoint with controller handshake | Repeats a chosen boundary without claiming global scheduling determinism; no guessing from sleep timing |
| Process crash vs machine failure? | Separate profiles and results; never fallback silently | Process kill preserves host page cache; guest kill removes guest page cache but is still a restricted failure model |
| Power-related backend? | Disposable QEMU/KVM Linux guest with fresh recovery boot, initially completed host writes retained | Now smoke-tested on real ext4 and bbolt. No host block devices or root needed. Device-cache loss/tears remain outside this profile. |
| `dm-log-writes`? | Later optional persistence exploration inside a supported guest, not a mandatory host dependency | Host kernel lacks it. Guest kernel supports it, but mark/ack/flush correlation and replay are separate qualification work. |
| Rust runtime strategy? | Keep native Tokio/threads untouched in the first path; optional Turmoil adapter for compatible components | Scheduler substitution across an arbitrary dependency graph is not required for the first contract |
| Go runtime strategy? | Keep native runtime untouched first; optional pinned Gosim adapter | Prior pinned Gosim worked and installed current Go failed; avoid owning a Go runtime translator |
| Common scheduler? | None | Native, Turmoil, Gosim and specialized concurrency tools have different control boundaries |
| Universal effect IR? | None in V1 | Common protocol and evidence envelope; retain native backend traces as typed extensions |
| Cross-language scenarios? | Native processes may share one external scenario; implement multi-node orchestration after single-boundary beta | The protocol already crosses languages. Joining independent in-process simulators into one virtual schedule is unsupported. |
| Specification language? | TOML orchestration metadata, normal code for workload/checker behavior, typed presets | Avoid a new behavioral DSL; reject unknown configuration fields |
| What do SDKs do? | Framing, operation identity, observation helpers and optional checkpoints | No replacement runtime; keep test-only integration separate from production library changes |
| How much code changes? | A test adapter is required; instrumentation only for precise internal boundaries | Zero upstream library changes demonstrated; zero integration work is not promised |
| Result vocabulary? | Violation, no violation observed, inconclusive; tool/config errors separately | Bounded proof-like absence is not available until an actually complete finite exploration profile exists |
| Unknown requests? | Preserve as may-have-committed; do not treat timeouts as rejection | Pending whole batch is allowed; a mixed partial batch is a counterexample |
| Missing values? | Explicit observed absence (`null`) can violate a contract; missing observation is inconclusive | Prevent observation gaps from being mistaken for either correctness or data loss |
| Initial search? | Small workloads, checkpoint/cut matrix and finite sampling with exact inputs saved | Debugging value before sophisticated search; a seed is not a coverage percentage |
| DPOR? | Excluded from V1 | Native events do not reveal sound independence; reuse Loom/Shuttle for fitting local components later |
| Failure reduction? | Greedy removal of earlier operations, then smaller payloads/fault choices as supported | Preserve target identity, contract and backend; report attempts and avoid global-minimum claims |
| Replay? | Capability-specific: native input replay, cooperative boundary replay, saved-image recovery, simulator replay | Every report states scope; changed-code replay is fix evaluation |
| Regression storage? | Checked-in scenario/contract plus exact failing workload and fault plan; large images referenced by hash | Rebuild compatible artifacts explicitly; no absolute temporary path as the long-term identity |
| CLI commands? | `init`, `doctor`, `test`, `inspect`, `replay`, `reduce` | First four user-facing commands are implemented; doctor/reduce make limits and debugging concrete |
| Local persistence? | Append-only NDJSON journals and immutable run directories; JSON report | No local service or database required for one developer |
| Report UI? | Terminal summary + JSON immediately; local static HTML next | A useful causal timeline precedes a hosted dashboard |
| CI? | Known regressions first, short sampled PR job, broader scheduled campaign | Nonzero inconclusive result is visible and never labeled pass; initial GitHub Actions workflow supplied |
| Deployment? | Free local execution first; BYOC execution for the first paid team offering | Avoid hosting untrusted builds and sensitive disk images before demand is demonstrated |
| Security boundary? | Trusted local adapters in preview; disposable job VM/cgroup ownership for managed release | Process groups control lifecycle, not security; daemonizing/escaping processes are unsupported in preview |
| Language/toolchain support? | Linux x86_64, a pinned tested CLI build, explicit SDK toolchain matrix | Native binaries need not share the CLI compiler. Simulator toolchain pins are stricter and separate. |
| Wasm/hypervisor? | No new hypervisor; no Wasm backend in V1; reuse QEMU where a guest is useful | Existing native code and filesystem behavior are the initial concern |
| License? | Apache-2.0 for owned code and open local evidence/replay interfaces | Do not hide truth, usable regressions or counterexample access behind payment |
| Paid value? | Maintained workflows, shared run/regression history, BYOC scheduling, triage and team controls | Local adoption is a distribution channel; conversion requires repeated team value |
| Initial price test? | $5,000 scoped onboarding/pilot; $1,500/month team continuation, BYOC compute paid by customer | Fixed commercial experiment with support cap, not an observed willingness to pay |
| When build more execution technology? | Only after a required customer behavior fails an assisted existing-tool comparison | Record the missing semantic capability and integration cost before approving a backend project |

These are engineering/product decisions, not assertions that the market has validated them. They are sufficient to implement V1 without repeatedly revisiting architectural alternatives.

## What has been built in this turn

A working Rust workspace and Go SDK implement the native portion above. Both actual-library adapters pass the same correct workloads, expose the same deliberately split-transaction bug, reproduce it, reduce it from three operations to one, and evaluate the atomic implementation under the unchanged contract. Failure guards cover missing checkpoints, malformed evidence, bad IDs, unsupported backends, timeout budgets, empty campaigns, immutable results and contract weakening.

The QEMU probe separately measured guest page-cache loss on real ext4 and bbolt. The CLI explicitly refuses that unintegrated backend today; this distinction is intentional. This is a tested construction foundation, not a claim that the complete V1 product has shipped.

## Decisions that require external evidence rather than more architecture

Willingness to pay, repeat use, comparative integration effort and how much domain logic can be reused cannot be resolved honestly by choosing a design. The release/validation document assigns tests and decision gates to them. We did not contact anyone, publish packages or deploy a service. Repository/package namespace and release credentials are administrative inputs before publication, not blockers to local implementation.
