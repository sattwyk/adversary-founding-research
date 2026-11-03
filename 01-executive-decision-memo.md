# Adversary: founding product decision

Decision date: 3 November 2026. Research and prototypes, not customer validation.

**Build a narrowly scoped recovery-contract qualification product for teams shipping stateful infrastructure. Deliver the first version as a paid integration plus an open local regression kit. Invest in a recurring platform only after customers demonstrate recurring use and willingness to pay.**

The first job is concrete: before releasing a change to a WAL, checkpoint, batching policy, storage configuration, or embedded database integration, establish whether acknowledged operations survive the declared crashes, whether recovery exposes an allowed state, and whether a previously discovered failure stays fixed.

Start with Linux systems whose durability boundary is local persistent storage and whose recovered state can be queried. Rust and Go are useful initial integration examples, not a permanent market boundary. A language-neutral command protocol can cover both without controlling either language's scheduler.

## Who adopts, and who buys

The user is the engineer owning persistence or recovery in a database, broker, durable queue, or infrastructure service. The buyer is its engineering leader or technical founder. The most promising customer has recently changed its persistence path, has an explicit durability promise, and lacks a maintained recovery harness. A team with only ordinary stateless APIs is a poor prospect.

Adoption must yield one useful release check within one engineer-day after the product supports the customer's execution profile. The engineer supplies a short workload, the acknowledgment boundary, and an application-specific recovery checker. The product supplies fault campaigns, evidence capture, failure reduction, and CI regression preservation. Zero modifications to an underlying library are feasible; zero application-specific test work is not.

Payment would buy avoided harness maintenance and faster diagnosis of release-blocking failures. Neither the value nor a price has been validated with customers in this investigation. Sell a defined integration outcome before selling compute capacity.

## Evidence that changed the decision

1. **A shared contract works on actual Rust and Go code without a common runtime.** The native probes use redb 4.4.0 and bbolt 1.4.3, with zero upstream source changes and 38/59 lines of wrapper code. The shared requirement is an atomic pair update that must survive recovery after acknowledgment. The prototype emits the same evidence envelope for both. These small libraries do not establish the integration cost of a production service. See [experiments](04-experiments.md) and [raw results](evidence/native-campaign/results.json).
2. **Process-crash testing can miss durability loss.** bbolt's deliberately enabled `NoSync` returned the update after 10/10 native kills. Reconstructing a storage image without unsynced writes recovered the previous value, violating the stronger application contract. Gosim independently reproduced this loss using bbolt 1.3.11; a native probe of that same version confirmed the process-kill blind spot. This is expected behavior under an intentionally weakened durability setting, not a newly discovered bbolt bug.
3. **The execution technology is substantially available.** Faultbox injected `fdatasync` errors into both native binaries without source changes. Gosim reproduced the Go recovery outcome across five seeds. Its pinned toolchain worked; the same checkout failed under the installed Go 1.26.4 toolchain. See [baseline logs](evidence/adversary-gosim-results.json).
4. **The broad competitive premise is weak.** Antithesis already offers full-system deterministic execution, assertions, exploration, and debugging. Its founder package explicitly targets smaller startups. Workers IO publishes a $2,500/month plan with 2,500 actual simulation runtime hours; Faultbox provides local fault injection and portable reports. Cheap access, cross-language support, or an evidence bundle alone is insufficient differentiation. [Antithesis founder package](https://antithesis.com/blog/2025/antithesis_for_founders/), [Workers IO pricing](https://workers.io/pricing/), [Faultbox implementation](https://github.com/faultbox/Faultbox).

## What to build first

An open CLI called `adversary` with a checked-in scenario manifest, ordinary executable workload/checker programs, a small set of versioned recovery contracts, and portable failure artifacts. The commercial offer is maintaining those contracts and campaigns through customer releases, initially with a scoped engineering engagement.

Support two explicitly different evidence paths. Native process termination and syscall-error tests operate on actual binaries. A calibrated Linux storage replay runner handles selected power-loss outcomes using existing block capture/replay infrastructure. Qualification of that runner is a prerequisite for selling power-loss results. The supplied single-file reconstruction prototype resolves an architectural question; it is not that qualified runner.

Use Rust for the CLI, artifact validation, campaign policy, and local execution supervisor. Invoke existing execution tools through process boundaries. Keep scheduling in each backend. Leave hosted orchestration, broad distributed consistency, DPOR, Wasm execution, and a custom deterministic runtime out of the first build. The interface should allow later backend additions without implementing them now.

## Why an existing tool is insufficient—and when it is sufficient

An existing tool is sufficient for the experimental bugs demonstrated here. Adversary has not established a superior bug-finding engine. The commercial hypothesis is narrower: a customer will pay for a maintained, domain-specific statement of its recovery promises, trustworthy fault-profile qualification, and small regressions that survive application changes. etcd's substantial validation and history-reconciliation code shows the engineering work involved, but does not prove customers will outsource it. [etcd validation](https://github.com/etcd-io/etcd/blob/4b0b745025716b893c64dbee443e3f969931a16b/tests/robustness/validate/validate.go).

If Antithesis, Workers IO, or an existing internal harness gives the customer that outcome at acceptable effort and cost, recommend that tool. If most qualified prospects do so, abandon the standalone product thesis. A thin dashboard over their results is not a defensible company.

## Six-week investment gate

Authorize a six-week engineering and commercial validation phase, with two founders as a planning assumption. Spend the first two weeks on storage-model calibration and on qualifying prospective users through existing founder access; no customer outreach was performed during this investigation.

Continue only if, by the end of the phase:

- Three independent teams attempt an integration; at least two achieve a useful check within one customer engineer-day on a supported profile.
- At least one nontrivial historical or real application recovery defect is reproduced; deliberately disabling sync is insufficient evidence of differentiated value.
- The storage runner passes the declared litmus suite against real Linux filesystems, including a directory-durability case, and preserves successful persistence barriers.
- Two customers agree to paid pilots with a defined route to recurring use. Suggested test offer: $5,000 for a scoped integration, then $750–$1,500/month for maintained release campaigns, with execution paid separately or capped. These are pricing experiments, not market observations.
- Customers identify a reason to choose Adversary beyond a lower price and available engineering labor.

**Verdict:** the technical path is credible; the standalone commercial opportunity remains unproven. Build the smallest recovery qualification offer that can test it. Do not fund a general autonomous verification platform on the present evidence.

