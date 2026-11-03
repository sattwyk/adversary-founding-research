# First product specification

Working name: **Adversary Recovery**. Status: proposed, with research prototypes delivered separately. This document specifies a first sellable version; it does not describe an already implemented platform.

The job: **qualify and preserve a customer's declared recovery guarantees across releases of a stateful Linux component or service.** The first commercial deliverable is one maintained contract and an executable regression campaign, backed by a scoped integration engagement.

## Customer and first use case

An infrastructure team with roughly 5–50 engineers is a prospecting assumption, not a measured market segment. Its persistence owner is changing a WAL, checkpoint replacement, write batching, acknowledgment policy, or embedded database integration. Its technical founder, engineering director or infrastructure lead approves the purchase. A recent durability incident or an upcoming persistence change is the buying trigger to test.

The first use case is a finite sequence of acknowledged atomic batches against a local durable store, interrupted during writing or recovery, then compared with permitted recovered states. Target the application integration and release process. Merely repeating the underlying database's own durability tests is usually insufficient value.

## Workflow and interface

1. `adversary init` creates a manifest and example driver/checker protocol for an existing binary or wrapper.
2. The engineer identifies the durable boundary, acknowledgment semantics, restart procedure, initial state and allowed uncertain outcomes.
3. `adversary qualify` runs capability and fault-profile checks. Unsupported effects produce an inconclusive result with the offending boundary.
4. `adversary run --budget 10m` performs the selected native or qualified storage campaign. The report states what actually ran and which faults landed.
5. `adversary inspect bundle/` shows the smallest retained witness, relevant client history, fault/persistence timeline, recovered state, assumptions and replay command.
6. `adversary replay bundle/` recreates the supported evidence scope and records reproduction frequency.
7. The engineer reviews the contract and saves a regression. CI reruns known witnesses before new sampling and tests the proposed corrective change under the same contract plus nearby cases.

These commands are proposed UX. The supplied research tools use `probe.py`, `replay.py`, and the reproduction scripts instead.

Example report language: “No violation observed: 200 selected recovery images, 4 workloads, profile X. Native schedules are uncontrolled. This is not an exhaustive execution check.” A supported counterexample states “Acknowledged batch 17 absent after recovery” and links the exact image and operation. An unqualified profile states “Inconclusive: writable shared mapping is outside the selected capture profile.” Never label either a general correctness proof.

## User stories and acceptance criteria

| Story | Acceptance criterion for a first sellable version |
|---|---|
| I need a useful check without rebuilding my runtime | Two independent pilot integrations reach a meaningful customer-specific check in at most one customer engineer-day on a supported profile; record Adversary support time separately |
| I need to know what success promises | Scenario includes explicit boundary, initial state, acknowledgment/unknown-outcome rules, checker identity and fault profile; missing fields prevent a conclusive campaign |
| I need real recovery behavior | Recovery runs the declared production binary or library on a fresh copy; report native versus substituted effects and all build/configuration hashes |
| I need power-loss evidence I can trust | Storage profile passes its calibration suite on a pinned guest/filesystem; completed flushes are preserved; acknowledgment marks do not add persistence; unsupported profiles remain inconclusive |
| I need to understand a failure | Bundle contains a focused failed predicate, relevant history, confirmed fault, exact input/snapshot and plain replay instructions; two pilot engineers can explain the observed violation without a founder interpreting it |
| I need regressions that remain useful | A fixed version passes the saved contract, witness replay and a small surrounding campaign; old known-bad version still fails under the supported replay scope |
| I need honest CI gating | All missing capture/history, checker timeouts and unsupported effects yield inconclusive/error; no green result is produced from absent observations |
| I need to bound cost | Every campaign has wall time, image count, memory/storage and artifact-retention limits; report consumed resources and unfinished checks |

Integration/time criteria are proposed product gates, not achieved customer measurements. The intentionally weakened configurations in the delivered probes do not satisfy the known-bad real-application-defect gate.

## Supported first-version behavior

Linux x86_64; source-available or supplied trusted binaries; Rust and Go example adapters through ordinary commands; one local persistent volume; bounded sequential client workloads; atomic batch durability and permitted recovered-state checks; process termination; selected syscall errors; selected crash cuts using one qualified filesystem/block profile; short recovery with a declared deadline; portable local bundles and basic CI execution.

Use existing tools for fault mediation. Contracts can be ordinary checker programs, with reusable helpers for durable prefixes, atomic batches, duplicate request IDs and referential consistency only as pilots justify them. Retry/idempotency support requires a stable logical operation ID and application queryability; it is not inferred from HTTP status codes. Preserve unknown operations as unknown. The first retry contract is an extension gate, not a day-one general financial-ledger claim.

Backend selection is explicit. Process-crash results and filesystem power-loss results are different tests. A storage-model qualification failure cannot silently fall back to process killing while retaining the requested power-loss label.

## Deliberately unsupported

- Arbitrary Rust/Go scheduling determinism, unmodified in-process simulation of every dependency, or instruction-level replay.
- General linearizability/serializability of arbitrary distributed transactions, external cloud storage correctness, multi-volume atomicity or Byzantine faults.
- Unqualified filesystem/device combinations, arbitrary persistent mmap/DAX, io_uring/direct-I/O paths until qualified, silent corruption after a successful flush, and SSD firmware guarantees.
- General liveness/fairness proof, a new formal specification language, autonomous oracle approval, and a certificate of whole-system correctness.
- Production fault injection, multi-tenant hosted execution, Windows/macOS-native fault mediation, and a large SDK/plugin ecosystem in the initial offer.

Unsupported behavior is identified at intake and qualification. A customer requiring it should receive an explicit scope decision or an existing-tool recommendation, not an implicit approximation.

## Packaging and commercial offer

Publish the local runner interface, artifact schema, basic contract helpers, checker execution and replay support under a permissive license, provisionally Apache-2.0 subject to dependency review. Keep semantic truth and access to failure evidence outside a paid gate. Potential paid value: integration, maintained profiles/contracts, scheduled release campaigns, shared history, regression triage, BYOC operation and enterprise controls. Hosted compute comes after recurring usage is demonstrated.

First pilot scope: one system, one supported profile, one or two guarantees, a bounded integration effort, a private evidence handoff, and a documented renewal decision. Exclude unlimited custom model writing and an implied correctness warranty. Price hypotheses and margin gates are in the [roadmap and economics](07-roadmap-and-economics.md).

