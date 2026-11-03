# Adversary founding investigation

**Recommendation:** build a recovery-contract qualification offer with an open regression kit and scoped paid integrations. Validate recurring demand and storage-model fidelity before investing in a broader verification platform.

The prototypes demonstrate native Rust/Go integration and compatible correctness evidence. They also show that an existing Go simulator can reproduce the demonstrated durability loss. No new upstream bug or customer demand was established. The experimental single-file storage model is not qualified for customer power-loss claims.

## Eight deliverables

1. [Executive decision memo](01-executive-decision-memo.md) — product choice, commercial rationale, counterevidence and investment gate.
2. [Technical research report](02-technical-research-report.md) — code observations, primary sources, language integration, fault fidelity and exploration.
3. [Architecture specification](03-architecture-specification.md) — module boundaries, contracts, execution profiles, evidence semantics, replay and isolation.
4. [Experiments and working prototypes](04-experiments.md) — measured results, limitations and reproduction; [prototype instructions](prototypes/README.md).
5. [Competitive comparison](05-competitive-comparison.md) — A–F alternatives and grounded comparisons with incumbents and newer tools.
6. [Product specification](06-product-specification.md) — first user, workflow, support boundary and acceptance criteria.
7. [Implementation roadmap and economics](07-roadmap-and-economics.md) — milestones, critical risks, cost/pricing scenarios and stop conditions.
8. [Assumption and falsification register](08-assumptions-and-falsification-register.md) — evidence, confidence and contingent decisions.

## Inspect or reproduce the evidence

- [Native experiment results](evidence/native-campaign/results.json), [compatible evidence artifacts](evidence/compatible-artifacts.ndjson), [environment and binary hashes](evidence/environment.json).
- [Gosim pinned/current toolchain results](evidence/adversary-gosim-results.json), [same-version native bbolt comparison](evidence/native-bbolt-1.3.11.json), [Faultbox results](evidence/faultbox-baseline/results.json).
- [Pinned repository revisions](evidence/repository_manifest.json), [code locations](evidence/code_inspection.json), [primary source index](evidence/source-index.md), [economics calculations](evidence/economics-scenarios.json).
- [Clean end-to-end reproduction log](evidence/clean-reproduction.log).

From this bundle directory on Linux, run `bash prototypes/run.sh`. For the independent Go simulation baseline, run `bash prototypes/run-gosim.sh`. Requirements and exact scope are in the prototype instructions. Build caches and upstream repository checkouts are intentionally excluded; pinned dependencies and source/replay scripts are included.

Measured facts, reported vendor capabilities, proposed product behavior and unvalidated commercial assumptions are distinguished throughout. Read the decision memo first, then the experiments before relying on the architecture.

