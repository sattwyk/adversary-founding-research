# What is resolved, and what must still be demonstrated

The construction choices are resolved in the decision record. “All questions resolved” cannot honestly mean that pricing, adoption or every storage assumption has become a measured fact. This ledger turns the remaining uncertainty into specific evidence requirements; none blocks implementing the agreed native V1.

| Item | Present evidence | Required next evidence | Decision if it fails |
|---|---|---|---|
| Shared Rust/Go integration | Implemented SDKs and CLI; identical contract/replay/reduction/fix workflow on real libraries | Three independent supported application integrations | Narrow the supported boundary or improve the adapter UX |
| Failure diagnosis and regression | Introduced split-transaction bug reproduced and reduced to one operation; corrected atomic path passes | Real/historical customer defect and useful regression through two code changes | Do not claim differentiated production bug finding; revise workload/oracle workflow |
| Guest-loss fidelity | Real Linux/ext4 and bbolt runs show cache loss and synced recovery; separate reproducible probe with witness capture | Full fixed-profile litmus/failure suite and clean-runner qualification | Keep experimental; no substitute single-file approximation with a stronger label |
| Device-cache/torn-write support | Not implemented or qualified | A concrete customer requirement, sound profile, calibration and best-existing-tool comparison | Keep unsupported; this does not prevent narrower guest-loss testing |
| Portable execution environment | Local preview hashes the top-level executable and stores exact inputs; native schedules/dependency closure uncontrolled | N2 portable regression and guest/image/artifact manifest | Call it input replay, not deterministic execution or a portable certificate |
| Supported toolchain floor | Rust 1.90 workspace tests and Go 1.23.2 SDK race/example checks pass locally; normal validation used Rust 1.97.1/Go 1.26.4 | Keep the supplied minimum-version CI jobs green and qualify upgrades | Narrow the published matrix, not an unmaintained promise |
| Reduced customer integration effort | Small real-library wrappers and working demos; no independent customer-time study | Assisted paired comparison with customer labor recorded | Do not claim a measured DX advantage |
| Reusable domain knowledge | One preset and two libraries; no cross-customer contract reuse | Independent systems reuse contract machinery without extensive bespoke work | Narrow domain or price explicit integration labor |
| Repeated adoption and payment | Public category spending only | Repeat runs through real changes plus two paid pilots/continuations | Revisit the team offer and commercial investment |
| Managed execution security | Trusted local preview only | Isolated jobs, credentials/egress controls, quotas, clean teardown and artifact policy | Remain local/BYOC under supported trust boundary |

Selected defaults—price, support cap, initial persona, license, native-first delivery, explicit backend selection and no custom runtime—are decisions to execute, not assumptions left for the implementer to choose. Their commercial effectiveness still has to be tested.

The unresolved inputs before public release are operational: confirmed repository/package namespace, release credentials, supported runner image and customer-approved examples. No production deployment, public publishing or communication was performed. Development can continue locally from the supplied code without any of those inputs.
