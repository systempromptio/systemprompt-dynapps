//! Runs the pinned structural and semantic evaluator without a shell.
use crate::model::{Phase, Validation, Workflow};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Serialize)]
struct Input<'a> {
    phase: Phase,
    operation: &'a str,
    workflow: &'a Workflow,
}
#[derive(Deserialize)]
struct Verdict {
    passed: bool,
}
#[derive(Deserialize)]
struct Scaffold {
    files: BTreeMap<String, String>,
}

async fn run(workflow: &Workflow, phase: Phase, operation: &str) -> Result<String> {
    let dir = tempfile::tempdir()?;
    let input = dir.path().join("input.json");
    let output = dir.path().join("output.json");
    tokio::fs::write(
        &input,
        serde_json::to_vec(&Input {
            phase,
            operation,
            workflow,
        })?,
    )
    .await?;
    let mut command = tokio::process::Command::new("node");
    command
        .arg("scripts/requirements-workflow/validate.mjs")
        .arg(input)
        .arg(&output)
        .kill_on_drop(true);
    if let Ok(secrets) = systemprompt::config::SecretsBootstrap::require() {
        if let Some(key) = secrets.get("bedrock_api_key") {
            command.env("AWS_BEARER_TOKEN_BEDROCK", key);
        }
    }
    let result = tokio::time::timeout(Duration::from_secs(900), command.output()).await??;
    ensure!(
        result.status.success(),
        "Validation runner failed: {}",
        String::from_utf8_lossy(&result.stderr)
            .chars()
            .take(1600)
            .collect::<String>()
    );
    Ok(tokio::fs::read_to_string(output).await?)
}

pub async fn validate(workflow: &Workflow, phase: Phase) -> Result<Validation> {
    workflow.require_predecessors(phase)?;
    let artifact = workflow
        .artifacts
        .get(&phase)
        .ok_or_else(|| anyhow::anyhow!("Submit the artifact first"))?;
    let report = run(workflow, phase, "validate").await?;
    let verdict: Verdict = serde_json::from_str(&report)?;
    Ok(Validation {
        artifact_hash: artifact.hash.clone(),
        dependency_hash: workflow.dependency_hash(phase)?,
        passed: verdict.passed,
        report,
        rubric_commit: workflow.rubric_commit.clone(),
    })
}

pub async fn scaffold(workflow: &Workflow) -> Result<BTreeMap<String, String>> {
    workflow.require_predecessors(Phase::Fsd)?;
    Ok(serde_json::from_str::<Scaffold>(&run(workflow, Phase::Fsd, "scaffold").await?)?.files)
}
