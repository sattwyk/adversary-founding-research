# Delivery plan and commercial decisions

The product direction is fixed. The remaining work is making a supported workflow reliable and showing that independent engineers prefer it to their best available alternative.

## Delivery backlog in dependency order

| Work item | Owner role | Deliverable / acceptance | Dependency |
|---|---|---|---|
| N1: native end-to-end core | Systems founder | Implemented preview: SDK protocol, real adapters, crash/check/inspect/replay/reduce, explicit outcomes | Complete; validation evidence supplied |
| N2: portable regression format | Systems founder | A checked-in regression can be rerun on a clean runner against a rebuilt current artifact; contract/fault/profile cannot silently change | N1; move beyond temporary absolute binary paths |
| D1: report and integration UX | Developer-experience founder | `init` templates, useful `doctor`, local HTML timeline, one copyable reproduction command, SDK examples with normal project builds | N1; first external usability test |
| N3: native lifecycle/resource hardening | Systems founder | Job-owned process/cgroup boundary, escaped-process detection or explicit refusal, wall/disk/log quotas, interrupted-controller cleanup, bounded artifact collection | N1 before offering untrusted/managed execution |
| G1: guest-loss adapter | Systems founder | Integrate the already executed VM approach behind the common scenario/evidence interface; explicit capability checks and fresh recovery clones | N2 and backend qualification |
| G2: guest profile qualification | Systems founder | Complete the litmus/failure suite in the backend document, clean-runner reproduction, real application integration | G1; no broad power-loss marketing before this |
| F1: existing syscall-fault integration | Systems founder | Pinned Faultbox invocation, confirmed injected events, no dependence on its supervisor exit status alone | N2/N3; a customer needing I/O failure behavior |
| C1: CI workflows | DX founder | Known regressions first, short PR job, periodic exploration, visible inconclusive status; example GitHub Actions workflow supplied | N2/D1 |
| U1: independent integrations | Both founders | Three supported applications attempted; record customer labor, founder assistance and alternative-tool baseline | D1; no claim that library demos are customer adoption |
| T1: team/BYOC layer | DX founder | Shared run/regression history, signed runner jobs, access/retention controls and scheduling | Repeated use by independent teams; N3 |
| S1: simulation adapters | Systems founder | One qualified Turmoil or Gosim integration selected by real user demand; backend-specific replay recorded | Stable common evidence and funded need; not parallel runtime projects |

Planning budget for two founders: weeks 1–2 finish portable native regressions, UX and lifecycle; weeks 2–4 integrate and qualify guest loss; weeks 3–6 run external integrations and repeat-change tests; team/BYOC productization follows paid recurring demand. Qualification overruns narrow the advertised profile rather than being hidden. This is a proposed work budget, not a committed delivery date.

The next merge-sized task is **N2**: extract a stable regression document from a failed run, resolve executable locations at invocation, record artifact/dependency requirements, and execute it from a clean temporary project. It should be followed by D1's report because successful debugging is the initial advantage the company is claiming.

## Default product boundaries

Free local use includes supported execution, all correctness evidence, local replay/reduction and normal CI use. Do not charge for access to a user's own failures or make offline regressions depend on a paid server. The initial project is Apache-2.0; backend/dependency licenses remain their own.

First paid team offer: **$1,500/month**, one named system/workflow, shared history/regression queue, BYOC scheduling and up to two hours/month of maintenance/triage support. Customer pays execution infrastructure. This is the initial price to test, not a validated market price or a production service already available. Extra bespoke model/backend work is a separately scoped engagement.

Offer a **$5,000 onboarding/pilot** for one supported system, one or two guarantees and at most two Adversary engineering days. It delivers a checked-in workflow, readable evidence and a continuation decision. Do not offer unlimited correctness audits at that price. A paid onboarding engagement alone does not establish subscription demand.

At an assumed $125/hour loaded support cost, two monthly hours plus a $100 allocation for coordination/storage cost $350 against $1,500 revenue, leaving $1,150 before R&D, sales and general overhead. Eight hours support instead leaves $400. These are planning scenarios, not observed margins. BYOC avoids hiding execution cost in an unlimited plan; it introduces installation/support work that must be measured.

This price is below some broad-platform offers, but lower price is not the differentiation claim. Antithesis has a small-founder offering; Workers IO publishes commercial capacity for smaller teams. The choice must be justified by integration, diagnosis and ongoing maintenance outcomes. [Antithesis founder offer](https://antithesis.com/blog/2025/antithesis_for_founders/), [Workers IO pricing](https://workers.io/pricing/).

## Customer and acquisition decisions

Start with persistence owners at database, durable queue/broker and stateful infrastructure teams with a recent storage/recovery change. The champion is a staff/senior engineer or persistence owner; the buyer is the technical founder or engineering leader. Small team size is a prospecting filter, not a TAM estimate. Avoid ordinary stateless applications and teams that cannot express or observe their guarantees.

Use founder introductions, maintainers and a concrete recovery-regression demonstration. Do not present disabled sync or the deliberately split example as a newly found production defect. Ask prospects to bring a real historical failure or release-risky persistence change and compare against the best existing tool with equivalent assistance.

The paired task measures: customer engineering time to a meaningful check, total founder assistance, time to understand/reproduce a counterexample, repeated effort after a real change and whether the buyer pays to keep the workflow maintained. Record oracle design separately from execution setup so an apparent DX improvement does not just move work into founder services.

No outreach or customer messages were sent in this work. Publication, a public package namespace and customer contact require their own concrete authorized actions; they are not implied completed milestones.

## Continue, narrow or stop gates

After three independent attempted integrations, require two useful supported checks within four customer engineering hours, with a material improvement over each team's best alternative. Four hours is a selected target, not a current measurement. Require a real/historical application defect, a comprehensible regression across two changes and two paid pilots with a reason beyond discounted engineering labor.

If ten substantive qualified conversations yield no paid pilot, revisit the offer/target rather than increasing backend scope. If every oracle is bespoke, narrow to a recurring domain or accept a services business consciously. If existing tools meet the same outcome with less effort, recommend them. If repeat users only value local free execution and never the team layer, revise monetization before hosting more compute.

Autonomous agents remain an interface consumer. The V1 workflow must justify itself for engineers first; the shared CLI/results make later agent use possible without a separate execution architecture.
