//! Deterministic document checks; semantic quality is measured by the
//! evaluation service.

use crate::model::{Phase, Validation, Workflow};
use anyhow::Result;
use serde_json::json;

pub fn validate(workflow: &Workflow, phase: Phase) -> Result<Validation> {
    workflow.require_predecessors(phase)?;
    let artifact = workflow
        .artifacts
        .get(&phase)
        .ok_or_else(|| anyhow::anyhow!("Submit the artifact first"))?;
    let body = artifact
        .files
        .values()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Artifact has no document"))?;
    let mut findings = Vec::new();
    if body.trim().len() < 100 {
        findings.push("Document is too short to review".to_owned());
    }
    if !workflow
        .evidence
        .iter()
        .any(|source| body.contains(&source.uri))
    {
        findings.push("Cite the frozen evidence using its source URLs".to_owned());
    }
    if body.contains("TODO") || body.contains("<placeholder>") {
        findings.push("Resolve draft placeholders before review".to_owned());
    }
    let report = json!({
        "kind":"deterministic", "passed":findings.is_empty(), "findings":findings,
        "semantic_status":"not_scored", "validator_version":workflow.validator_version,
        "limitations":"Checks do not establish factual support, completeness or acceptance-criteria quality. Human review remains required."
    });
    Ok(Validation {
        artifact_hash: artifact.hash.clone(),
        dependency_hash: workflow.dependency_hash(phase)?,
        passed: findings.is_empty(),
        report: report.to_string(),
        validator_version: workflow.validator_version.clone(),
    })
}
