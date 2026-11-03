use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

pub type State = BTreeMap<String, String>;
pub type Observation = BTreeMap<String, Option<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    pub name: String,
    pub backend: String,
    pub adapter: Adapter,
    pub contract: Contract,
    pub campaign: Campaign,
    pub fault: Fault,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adapter {
    pub command: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub cwd: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub kind: String,
    pub initial: State,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub seed: u64,
    pub cases: usize,
    pub operations: usize,
    pub timeout_ms: u64,
    #[serde(default = "default_wall_time")]
    pub wall_time_ms: u64,
}
fn default_wall_time() -> u64 {
    60_000
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fault {
    pub kind: String,
    pub checkpoint: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub id: String,
    pub batch: State,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub seed: u64,
    pub operations: Vec<Operation>,
    pub target_id: String,
}

pub fn hash_file(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

pub fn validate(cfg: &Config) -> Result<()> {
    ensure!(cfg.schema == 1, "unsupported scenario schema");
    ensure!(
        cfg.backend == "native-process",
        "backend {} is not implemented; no fallback is allowed",
        cfg.backend
    );
    ensure!(cfg.contract.kind == "atomic-map", "unsupported contract");
    ensure!(
        !cfg.name.is_empty() && !cfg.adapter.command.is_empty(),
        "scenario name and adapter command required"
    );
    ensure!(
        !cfg.contract.initial.is_empty() && cfg.contract.initial.len() <= 100,
        "contract requires 1..100 keys"
    );
    ensure!(
        cfg.campaign.cases > 0 && cfg.campaign.cases <= 1000,
        "cases must be 1..1000"
    );
    ensure!(
        cfg.campaign.operations > 0 && cfg.campaign.operations <= 100,
        "operations must be 1..100"
    );
    ensure!(
        (10..=60000).contains(&cfg.campaign.timeout_ms),
        "per-request timeout must be 10..60000 ms"
    );
    ensure!(
        (10..=600_000).contains(&cfg.campaign.wall_time_ms),
        "campaign wall time must be 10..600000 ms"
    );
    match cfg.fault.kind.as_str() {
        "after-ack" => ensure!(
            cfg.fault.checkpoint.is_none(),
            "after-ack does not take a checkpoint"
        ),
        "checkpoint" => ensure!(
            cfg.fault.checkpoint.as_ref().is_some_and(|s| !s.is_empty()),
            "checkpoint name required"
        ),
        _ => bail!("unsupported fault kind"),
    }
    Ok(())
}

pub fn generate(cfg: &Config) -> Vec<Case> {
    (0..cfg.campaign.cases)
        .map(|i| {
            let seed = cfg.campaign.seed.wrapping_add(i as u64);
            let operations: Vec<_> = (0..cfg.campaign.operations)
                .map(|j| Operation {
                    id: format!("op-{}", j + 1),
                    batch: cfg
                        .contract
                        .initial
                        .keys()
                        .map(|k| (k.clone(), format!("seed-{seed}/op-{}", j + 1)))
                        .collect(),
                })
                .collect();
            let target_id = operations.last().unwrap().id.clone();
            Case {
                seed,
                operations,
                target_id,
            }
        })
        .collect()
}

/// Complete prior acknowledgments are required; one interrupted atomic batch may
/// either take effect completely or have no effect. Key partitioning is forbidden.
pub fn allowed_recovery(before: &State, pending: Option<&Operation>, observed: &State) -> bool {
    if observed == before {
        return true;
    }
    if let Some(op) = pending {
        let mut after = before.clone();
        after.extend(op.batch.clone());
        return &after == observed;
    }
    false
}

pub fn allowed_observation(
    before: &State,
    pending: Option<&Operation>,
    observed: &Observation,
) -> bool {
    let concrete = observed
        .iter()
        .map(|(key, value)| value.clone().map(|v| (key.clone(), v)))
        .collect::<Option<State>>();
    concrete.is_some_and(|state| allowed_recovery(before, pending, &state))
}
