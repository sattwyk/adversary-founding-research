# Implementation roadmap and startup economics

Planning basis: two experienced founders, six weeks of validation work before a broader product commitment. Timing is a budget, not a completed schedule or an assurance that storage calibration fits it. Public evidence establishes demand for correctness work, not demand for this startup. No outreach, sale or customer commitment was made during this investigation.

## Roadmap with decision gates

| Milestone | Work and deliverable | Dependencies and go/no-go gate |
|---|---|---|
| **First technical milestone, week 1** | Rust CLI schema/contract validation, native command runner, known witness replay, explicit inconclusive paths; reuse a maintained syscall injector | Validate old known-bad and corrected examples; no pass from missing history; avoid reimplementing Faultbox's mediation layer |
| **First useful end-to-end demo, weeks 1–2** | One production-code workload produces a recovery witness, explanation, saved regression, then evaluates a corrective configuration/change | Research probes provide a starting point. A nontrivial historical app defect must replace the sync-disabled demonstration before claiming distinctive value. |
| **Storage qualification, weeks 1–3, critical path** | Disposable guest, pinned real filesystem, existing block capture/replay, acknowledgment correlation, calibration suite | Completed flush/FUA preservation; directory create/rename and length-growth cases; fresh recovery cache; no artifact/capture gaps. If calibration fails or overruns, limit the offer to the narrower supported native contract. |
| **First real-world integration, weeks 2–4** | A stateful application's real persistence path, workload, unknown-operation semantics and recovery checker; publish a private engineering assessment | Use maintainer/customer authorization; do not mistake downloading an OSS library for an external integration. At most one customer engineer-day target and at most two Adversary engineer-days for a supported profile. |
| **First external user, weeks 3–5** | User installs and runs a meaningful check, reproduces/understands the witness, reruns after a real change | Two users should complete interpretation without founder narration. Track both customer and support labor; reject flattering demo feedback as adoption. |
| **First paid pilots, weeks 4–6** | Two scoped paid qualification engagements and an explicit monthly continuation option | Real budget owner; identified release trigger; bounded delivery obligations; reason for choosing Adversary versus an assisted existing tool |
| **Productization, after gate** | Reduce repeated integration work, semantic fault coverage, workload reduction, curated contract helpers, CI summaries | Repeat use across releases; most new work reusable; support cost within unit-economics envelope |
| **Recurring infrastructure, conditional** | Scheduling, shared artifacts, BYOC workers, resource metering, team access, regression queue | At least several customers use it recurrently; avoid a hosted control plane as a substitute for proving demand |

Immediate builds: manifest/evidence validation, native orchestration wrapper, checker protocol, durable-prefix/batch helper, fresh-image replay, known-regression CI, and storage-profile calibration. Keep simulation adapters, generic history-checker integrations, hosted scheduling, native process checkpointing, DPOR, Wasm, and autonomous repair as interfaces or future projects.

Critical risks are oracle design, storage fidelity, acknowledgment/capture correlation, reproduction drift, toolchain/dependency compatibility, and integration services expanding beyond reusable work. A two-language custom scheduler is not on the critical path because the recommended product does not need it.

## Public commercial evidence

Antithesis' founder offer describes small customers paying a few thousand dollars per month, with early services giving way to self-service. This both supports category willingness to spend and challenges Adversary's down-market premise. It is not a current individualized quote. [Primary source](https://antithesis.com/blog/2025/antithesis_for_founders/).

Workers IO lists $2,500/month for 2,500 actual simulation runtime hours, $3.60/hour overage and custom enterprise deployment/support. We did not purchase or benchmark it; simulated application hours are not a comparable billing unit. [Primary pricing](https://workers.io/pricing/).

WarpStream's engineering account demonstrates actual use of commercial correctness infrastructure and continuing workload investment. Jepsen's published analyses demonstrate demand for specialized correctness engagements. Neither source publishes a comparable Adversary conversion rate, integration cost or contract value. [WarpStream](https://www.warpstream.com/blog/deterministic-simulation-testing-for-our-entire-saas), [Jepsen](https://jepsen.io/analyses).

## Customer economics: explicit scenarios

Assume loaded engineering time costs **$125/hour** solely for planning. Replace this with customer-specific opportunity cost during a pilot. Saving four hours monthly is worth $500; saving twelve is worth $1,500 before compute and switching effort. Avoided incident value may be large, but cannot be estimated responsibly without the customer's incident frequency, impact and existing defenses.

Illustrative DIY harness: 40 initial hours plus four hours/month maintenance = $5,000 initial and $500/month. If a customer already has that harness and it works, a $1,500/month subscription may have poor economics. If a regression costs an owner two days to reproduce every release, maintained witnesses could be worth more. These are scenarios to test, not observed averages.

The one-day integration target itself costs the customer about $1,000 under this assumption. Two founder days cost another $2,000 in delivery labor. A $5,000 qualification pilot with those delivery hours leaves $3,000 before compute, acquisition and product development; broad bespoke work can consume that quickly. Ask for a paid scope before extending the integration surface.

## Recurring delivery economics

Illustrative execution cost: **$0.05 per provisioned vCPU-hour**, not a researched cloud quote. Include all billed execution, capture and recovery time, not virtual application time. Memory, idle reservations, I/O, guest boot and storage may dominate; measure them in pilots before quoting hosted capacity. Engineering support remains the most dangerous cost.

| Scenario | Actual vCPU-hours/month | Compute | Artifact/control allocation | Support at $125/h | Total direct cost | Test subscription | Contribution before R&D/sales |
|---|---:|---:|---:|---:|---:|---:|---:|
| Small | 100 | $5 | $75 | 1 h = $125 | $205 | $750 | $545 / 72.7% |
| Base | 1,000 | $50 | $100 | 2 h = $250 | $400 | $1,500 | $1,100 / 73.3% |
| High-touch/heavy | 10,000 | $500 | $250 | 8 h = $1,000 | $1,750 | $3,000 | $1,250 / 41.7% |

These are contribution scenarios, not company gross-margin forecasts. They omit founder R&D, acquisition, taxes and general overhead. At the base subscription, support rising from two to eight hours reduces contribution to $350/23.3%, even with unchanged compute. If the fully loaded execution rate rises to $0.20/vCPU-hour, base direct cost rises to $550 and contribution falls to $950/63.3%; the heavy scenario reaches $3,250 and loses money at $3,000. [Inspectable calculation](evidence/economics-scenarios.json).

Use a capped compute budget or customer-paid execution initially. Pricing per simulated year or promising unlimited custom scenarios hides the cost driver. The small native probes' subsecond runtimes cannot predict these economics.

## Monetization choices

| Model | Assessment |
|---|---|
| Open-core | Useful distribution and trust; open the local contract/evidence/replay path, charge for maintained service/team operation. No automatic conversion assumed. |
| Paid correctness integration | Best initial willingness-to-pay test; fixed scope, named guarantees, known support cap. One-off audits alone do not prove recurring demand. |
| Per-project maintained subscription | Best initial recurring hypothesis: a named service's release contracts and regression campaign, with explicit capacity/support boundaries |
| Hosted compute pricing | Later; clear actual-runtime accounting, quotas and minimum subscription. Competes directly with established platforms. |
| BYOC execution | Attractive when code/data cannot leave the customer. Reduces Adversary compute exposure, adds installation/upgrade/support burden. |
| Enterprise licensing | Potential for qualified profiles, private deployment and controls, but premature before repeatability; do not build a second deployment product for a single unpaid prospect |

Test a $5,000 fixed pilot and a $750–$1,500/month continuation with limited support/execution. A higher-touch/private installation needs a separate quote tied to delivery costs. These numbers are proposed experiments, not claims of accepted pricing.

Acquire initial users through founders' relevant infrastructure relationships, maintainer introductions, and a concrete published recovery-regression example after authorized disclosure. Qualify around recent persistence changes or incidents. Offer a paired comparison against the customer's best available tool. No unsolicited outreach was sent as part of this work.

Stop or pivot if qualified teams do not allocate engineering time; if two paid pilots cannot be obtained after ten substantive qualified conversations; if users only value a one-time audit; if most integrations require custom runtime work; or if an existing platform supplies the same maintained outcome with less effort. These are chosen investment discipline thresholds, not statistically justified market estimates.

## Longer-term path

Grow reusable contracts and calibrated effect boundaries first. Add concurrent histories only when needed, then simulation adapters for customers who already fit them. The same artifact bundle can be consumed by a human, CI or a development agent. An autonomous loop can request a bounded campaign, inspect counterexamples, evaluate a proposed fix and preserve a regression without changing the execution architecture. Generality should follow repeated customer demand and a demonstrated ability to maintain honest evidence.

