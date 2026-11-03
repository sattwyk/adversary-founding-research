#![cfg(target_os = "linux")]
pub mod model;
pub mod runner;

use adversary_protocol::{Request, VERSION};
use anyhow::{Context, Result, ensure};
use model::{Case, Config, Observation, Operation, State};
use runner::{Call, Process};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub case: Case,
    pub outcome: String,
    pub detail: String,
    pub fault_confirmed: bool,
    pub history_complete: bool,
    pub acknowledged: Vec<String>,
    pub unknown: Option<String>,
    pub required_before_pending: State,
    pub observed: Option<Observation>,
    pub milliseconds: u128,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema: u32,
    pub config: Config,
    pub adapter_sha256: String,
    pub execution: Value,
    pub outcome: String,
    pub invocation: String,
    pub results: Vec<CaseResult>,
}

pub fn load_config(path: &Path) -> Result<Config> {
    let mut cfg: Config = toml::from_str(&fs::read_to_string(path)?)?;
    let base = path.canonicalize()?.parent().unwrap().to_path_buf();
    model::validate(&cfg)?;
    let executable = PathBuf::from(&cfg.adapter.command[0]);
    ensure!(
        executable.is_absolute() || executable.components().count() > 1,
        "use an explicit adapter path; PATH lookup is not a pinned artifact"
    );
    cfg.adapter.command[0] = if executable.is_absolute() {
        executable
    } else {
        base.join(executable)
    }
    .canonicalize()?
    .to_string_lossy()
    .into();
    cfg.adapter.cwd = Some(base.to_string_lossy().into());
    Ok(cfg)
}

fn request(id: &str, method: &str, params: Value) -> Request {
    Request {
        v: VERSION,
        id: id.into(),
        method: method.into(),
        params,
    }
}
fn response(call: Call) -> Result<Value> {
    match call {
        Call::Response(v) => Ok(v),
        Call::Interrupted => anyhow::bail!("unexpected interrupted protocol call"),
    }
}

fn execute_case(
    cfg: &Config,
    case: &Case,
    root: &Path,
    campaign_deadline: Instant,
) -> Result<CaseResult> {
    let mut process = Process::start(
        &cfg.adapter,
        &root.join("data"),
        &root.join("capture"),
        cfg.campaign.timeout_ms,
        campaign_deadline,
    )?;
    response(process.call(
        &request(
            "initialize",
            "initialize",
            json!({"state":cfg.contract.initial}),
        ),
        None,
    )?)?;
    let initial: Observation = serde_json::from_value(response(process.call(
        &request(
            "initial-observation",
            "observe",
            json!({"keys":cfg.contract.initial.keys().collect::<Vec<_>>()}),
        ),
        None,
    )?)?)?;
    ensure!(
        model::allowed_observation(&cfg.contract.initial, None, &initial),
        "initialization did not establish the declared state"
    );
    let mut before = cfg.contract.initial.clone();
    let mut acknowledged = Vec::new();
    let mut pending: Option<Operation> = None;
    let mut landed = false;
    for op in &case.operations {
        let target = op.id == case.target_id;
        let point = if target && cfg.fault.kind == "checkpoint" {
            cfg.fault.checkpoint.as_deref()
        } else {
            None
        };
        match process.call(&request(&op.id, "apply", json!({"batch":op.batch})), point)? {
            Call::Interrupted => {
                pending = Some(op.clone());
                landed = process.fault_landed;
                break;
            }
            Call::Response(value) => {
                ensure!(
                    value.get("accepted").and_then(Value::as_bool) == Some(true),
                    "operation response does not establish accepted success"
                );
                before.extend(op.batch.clone());
                acknowledged.push(op.id.clone());
                if target {
                    ensure!(
                        cfg.fault.kind == "after-ack",
                        "checkpoint target unexpectedly completed"
                    );
                    process.kill()?;
                    landed = true;
                    break;
                }
            }
        }
    }
    ensure!(landed, "requested fault did not land");
    drop(process);
    let mut recovery = Process::start(
        &cfg.adapter,
        &root.join("data"),
        &root.join("recovery"),
        cfg.campaign.timeout_ms,
        campaign_deadline,
    )?;
    let observed: Observation = serde_json::from_value(response(recovery.call(
        &request(
            "recover",
            "observe",
            json!({"keys":cfg.contract.initial.keys().collect::<Vec<_>>()}),
        ),
        None,
    )?)?)
    .context("recovery did not return a string-or-null observation")?;
    ensure!(
        observed.keys().eq(cfg.contract.initial.keys()),
        "recovery observation keys do not match contract keys"
    );
    let valid = model::allowed_observation(&before, pending.as_ref(), &observed);
    Ok(CaseResult {
        case: case.clone(),
        outcome: if valid {
            "no_violation_observed"
        } else {
            "violation"
        }
        .into(),
        detail: if valid {
            "recovered state is allowed by this finite workload"
        } else {
            "recovered state violates acknowledged durability or interrupted-batch atomicity"
        }
        .into(),
        fault_confirmed: true,
        history_complete: true,
        acknowledged,
        unknown: pending.map(|x| x.id),
        required_before_pending: before,
        observed: Some(observed),
        milliseconds: 0,
    })
}

pub fn run(cfg: Config, cases: Vec<Case>, output: &Path, invocation: &str) -> Result<Report> {
    model::validate(&cfg)?;
    ensure!(
        !output.exists(),
        "output path already exists; runs are immutable"
    );
    ensure!(
        !cases.is_empty(),
        "empty case list cannot produce a passing run"
    );
    for case in &cases {
        ensure!(!case.operations.is_empty(), "empty workload");
        ensure!(
            case.operations.iter().any(|x| x.id == case.target_id),
            "fault target missing"
        );
        let mut ids = std::collections::BTreeSet::new();
        for op in &case.operations {
            ensure!(ids.insert(&op.id), "duplicate operation ID");
            ensure!(
                op.batch
                    .keys()
                    .all(|k| cfg.contract.initial.contains_key(k)),
                "workload references undeclared keys"
            );
        }
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(output)?;
    let campaign_deadline = Instant::now() + Duration::from_millis(cfg.campaign.wall_time_ms);
    let mut results = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let start = Instant::now();
        let root = output.join(format!("case-{i:04}"));
        fs::create_dir(&root)?;
        let mut result = match execute_case(&cfg, case, &root, campaign_deadline) {
            Ok(r) => r,
            Err(e) => CaseResult {
                case: case.clone(),
                outcome: "inconclusive".into(),
                detail: format!("{e:#}"),
                fault_confirmed: false,
                history_complete: false,
                acknowledged: vec![],
                unknown: None,
                required_before_pending: cfg.contract.initial.clone(),
                observed: None,
                milliseconds: 0,
            },
        };
        result.milliseconds = start.elapsed().as_millis();
        fs::write(
            root.join("result.json"),
            serde_json::to_vec_pretty(&result)?,
        )?;
        results.push(result);
    }
    let outcome = if results.iter().any(|r| r.outcome == "violation") {
        "violation"
    } else if results.iter().any(|r| r.outcome == "inconclusive") {
        "inconclusive"
    } else {
        "no_violation_observed"
    };
    let report = Report {
        schema: 1,
        adapter_sha256: model::hash_file(Path::new(&cfg.adapter.command[0]))?,
        execution: json!({"backend":"native-process","fault":"SIGKILL","evidence":"native-observation","scheduler_control":false,"power_loss":false,"scope":"finite sequential atomic-map workloads","replay":"same inputs and named cooperative boundary when used; no instruction replay","exhaustive":false,"sandbox":"none; trusted local adapters only","request_timeout_ms":cfg.campaign.timeout_ms}),
        config: cfg,
        outcome: outcome.into(),
        invocation: invocation.into(),
        results,
    };
    fs::write(output.join("run.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(report)
}

pub fn load_report(path: &Path) -> Result<Report> {
    let report: Report = serde_json::from_slice(&fs::read(path.join("run.json"))?)?;
    ensure!(report.schema == 1, "unsupported report schema");
    model::validate(&report.config)?;
    Ok(report)
}

pub fn summary(report: &Report) -> String {
    format!(
        "{}: {} cases; {} violations; {} inconclusive. Native scheduling uncontrolled; no power-loss claim.",
        report.outcome,
        report.results.len(),
        report
            .results
            .iter()
            .filter(|r| r.outcome == "violation")
            .count(),
        report
            .results
            .iter()
            .filter(|r| r.outcome == "inconclusive")
            .count()
    )
}
pub fn exit_code(report: &Report) -> u8 {
    match report.outcome.as_str() {
        "violation" => 1,
        "inconclusive" => 2,
        _ => 0,
    }
}
