use adversary::{exit_code, load_config, load_report, model, run, summary};
use anyhow::{Result, ensure};
use clap::{Parser, Subcommand};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(version, about = "Adversary native recovery testing preview")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long, default_value = "rust")]
        language: String,
        #[arg(long, default_value = "adversary.toml")]
        output: PathBuf,
    },
    Doctor {
        #[arg(long, default_value = "adversary.toml")]
        manifest: PathBuf,
    },
    Test {
        #[arg(long, default_value = "adversary.toml")]
        manifest: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    Inspect {
        run_id: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Replay {
        failure_id: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        allow_changed: bool,
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
    Reduce {
        failure_id: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = 10)]
        budget: usize,
    },
}

fn execute(cli: Cli) -> Result<u8> {
    match cli.command {
        Commands::Init { language, output } => {
            ensure!(
                language == "rust" || language == "go",
                "language must be rust or go"
            );
            ensure!(
                !output.exists(),
                "refusing to overwrite an existing manifest"
            );
            let binary = if language == "rust" {
                "./target/release/redb-adapter"
            } else {
                "./work/bbolt-adapter"
            };
            let text = format!(
                "schema = 1\nname = \"my-recovery-check\"\nbackend = \"native-process\"\n\n[adapter]\ncommand = [\"{binary}\"]\n\n[contract]\nkind = \"atomic-map\"\n[contract.initial]\na = \"0\"\nb = \"0\"\n\n[campaign]\nseed = 1\ncases = 3\noperations = 3\ntimeout_ms = 5000\n\n[fault]\nkind = \"after-ack\"\n"
            );
            fs::write(output, text)?;
            println!(
                "Created scenario. Supply an adapter implementing initialize/apply/observe; then run doctor and test."
            );
            Ok(0)
        }
        Commands::Doctor { manifest } => {
            let cfg = load_config(&manifest)?;
            println!(
                "native-process available; adapter SHA256={}; SDK protocol=1; no sandbox or deterministic schedule; power loss unsupported",
                model::hash_file(Path::new(&cfg.adapter.command[0]))?
            );
            Ok(0)
        }
        Commands::Test { manifest, output } => {
            let cfg = load_config(&manifest)?;
            let cases = model::generate(&cfg);
            let report = run(cfg, cases, &output, "campaign")?;
            println!("{}\nArtifacts: {}", summary(&report), output.display());
            Ok(exit_code(&report))
        }
        Commands::Inspect { run_id, json } => {
            let report = load_report(&run_id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", summary(&report));
                for r in &report.results {
                    println!("seed {}: {} — {}", r.case.seed, r.outcome, r.detail);
                }
            }
            Ok(0)
        }
        Commands::Replay {
            failure_id,
            output,
            allow_changed,
            manifest,
        } => {
            let old = load_report(&failure_id)?;
            ensure!(
                manifest.is_none() || allow_changed,
                "a replacement manifest requires --allow-changed"
            );
            let cfg = if let Some(path) = manifest {
                let cfg = load_config(&path)?;
                ensure!(
                    serde_json::to_value(&cfg.contract)?
                        == serde_json::to_value(&old.config.contract)?
                        && serde_json::to_value(&cfg.fault)?
                            == serde_json::to_value(&old.config.fault)?
                        && cfg.backend == old.config.backend,
                    "fix evaluation must preserve contract, fault, and backend"
                );
                cfg
            } else {
                old.config.clone()
            };
            let hash = model::hash_file(Path::new(&cfg.adapter.command[0]))?;
            ensure!(
                allow_changed || hash == old.adapter_sha256,
                "adapter changed: use --allow-changed for fix evaluation, not identical-artifact replay"
            );
            let cases = old
                .results
                .iter()
                .filter(|r| r.outcome == "violation")
                .map(|r| r.case.clone())
                .collect::<Vec<_>>();
            ensure!(
                !cases.is_empty(),
                "run contains no failed contract to replay"
            );
            let report = run(
                cfg,
                cases,
                &output,
                if allow_changed {
                    "fix-evaluation"
                } else {
                    "failure-reproduction"
                },
            )?;
            println!("{}", summary(&report));
            Ok(exit_code(&report))
        }
        Commands::Reduce {
            failure_id,
            output,
            budget,
        } => {
            ensure!(!output.exists(), "reduction output already exists");
            let old = load_report(&failure_id)?;
            ensure!(
                model::hash_file(Path::new(&old.config.adapter.command[0]))? == old.adapter_sha256,
                "reduction requires original adapter artifact"
            );
            let mut case = old
                .results
                .iter()
                .find(|r| r.outcome == "violation")
                .context("no violation to reduce")?
                .case
                .clone();
            fs::create_dir_all(&output)?;
            let mut attempts = 0;
            let mut i = 0;
            while i < case.operations.len() && attempts < budget {
                if case.operations[i].id == case.target_id {
                    i += 1;
                    continue;
                }
                let mut trial = case.clone();
                trial.operations.remove(i);
                let r = run(
                    old.config.clone(),
                    vec![trial.clone()],
                    &output.join(format!("trial-{attempts}")),
                    "reduction-trial",
                )?;
                attempts += 1;
                if r.outcome == "violation" {
                    case = trial;
                } else {
                    i += 1;
                }
            }
            let report = run(
                old.config,
                vec![case],
                &output.join("minimal"),
                "greedy-reduction; not globally minimal",
            )?;
            println!("{}; reduction attempts={attempts}", summary(&report));
            Ok(exit_code(&report))
        }
    }
}
use anyhow::Context;
fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let success = matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            );
            let _ = error.print();
            return ExitCode::from(if success { 0 } else { 3 });
        }
    };
    match execute(cli) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("adversary error: {e:#}");
            ExitCode::from(3)
        }
    }
}
