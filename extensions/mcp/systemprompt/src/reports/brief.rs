//! The daily brief: one traffic light per project over the same Jira facts the
//! project report reads.
//!
//! The light is derived here rather than read from a Jira field on purpose. A
//! project's own RAG status is a claim someone typed; these three signals are
//! what the issues actually show. Where the two disagree that IS the finding,
//! so the brief carries both and never silently prefers one.

use super::atlassian::{Atlassian, identifier};
use super::shape::table;
use super::{ReportInput, ReportOutput, invalid};
use serde_json::{Value, json};
use std::collections::BTreeMap;

// Why: red if delivery is already breaking its own dates, amber if it is
// drifting, green otherwise. "unknown" is a real answer — a project whose
// issues carry no usable status is not green, and reporting it as green is the
// failure mode this whole dashboard exists to prevent.
#[must_use]
pub fn traffic_light(metrics: &BTreeMap<String, u64>) -> &'static str {
    let get = |key: &str| metrics.get(key).copied().unwrap_or_default();
    let sample = get("Issues in sample");
    if sample == 0 {
        return "unknown";
    }
    let overdue = get("Overdue unresolved");
    let stalled = get("Unchanged for 7 days");
    let unassigned = get("Unassigned unresolved");
    // Why: proportions, not counts. A hundred-issue project with three overdue
    // items is healthier than a five-issue project with three, and a threshold
    // on the raw count would rank them the other way round.
    let share = |count: u64| (count * 100) / sample;
    if overdue > 0 && share(overdue) >= 10 {
        return "red";
    }
    if share(stalled) >= 25 || share(unassigned) >= 25 || overdue > 0 {
        return "amber";
    }
    "green"
}

// Why: the worst light across the portfolio is what the header states. A brief
// that averaged its projects would report green while one of them burned.
fn worst(lights: &[&str]) -> &'static str {
    for level in ["red", "amber", "unknown", "green"] {
        if lights.contains(&level) {
            return match level {
                "red" => "red",
                "amber" => "amber",
                "unknown" => "unknown",
                _ => "green",
            };
        }
    }
    "unknown"
}

fn validate(cloud_id: &str, project_keys: &[String]) -> Result<(), rmcp::ErrorData> {
    identifier(cloud_id)?;
    if project_keys.is_empty() {
        return Err(invalid(
            "project_keys must name at least one project; call admin_project_catalog to list them",
        ));
    }
    if project_keys.len() > 20 {
        return Err(invalid("project_keys is limited to 20 projects per brief"));
    }
    for key in project_keys {
        identifier(key)?;
    }
    Ok(())
}

// Why: one project's contribution to the brief — its light, its table row, the
// metrics to fold into the portfolio totals, and any source warnings.
struct ProjectLight {
    light: &'static str,
    row: Value,
    metrics: BTreeMap<String, u64>,
    warnings: Vec<(String, Option<String>)>,
}

// Why: one project's failure must not take the brief down with it. A site the
// caller cannot read becomes a source warning and an unknown light, which is
// the honest rendering of "we could not look" — folding it into green would
// hide exactly the project most likely to need attention.
async fn one_project(client: &Atlassian, cloud_id: &str, key: &str, days: u16) -> ProjectLight {
    let single = ReportInput::Projects {
        cloud_id: cloud_id.to_owned(),
        project_key: key.to_owned(),
        days,
        space_id: None,
    };
    match super::projects::report(client, &single).await {
        Ok(project) => {
            let light = traffic_light(&project.metrics);
            let get = |name: &str| project.metrics.get(name).copied().unwrap_or_default();
            let row = json!({
                "project": key,
                "light": light,
                "issues": get("Issues in sample"),
                "overdue": get("Overdue unresolved"),
                "stalled": get("Unchanged for 7 days"),
                "unassigned": get("Unassigned unresolved"),
                "completed": get("Completed in period"),
            });
            let warnings = project
                .sources
                .into_iter()
                .filter(|source| !source.complete)
                .map(|source| (format!("{key}: {}", source.source), source.warning))
                .collect();
            ProjectLight {
                light,
                row,
                metrics: project.metrics,
                warnings,
            }
        },
        Err(error) => ProjectLight {
            light: "unknown",
            row: json!({"project": key, "light": "unknown"}),
            metrics: BTreeMap::new(),
            warnings: vec![(format!("{key}: Jira"), Some(error.message.to_string()))],
        },
    }
}

pub async fn report(
    client: &Atlassian,
    input: &ReportInput,
) -> Result<ReportOutput, rmcp::ErrorData> {
    let ReportInput::Brief {
        cloud_id,
        project_keys,
        days,
    } = input
    else {
        return Err(invalid("Expected brief input"));
    };
    validate(cloud_id, project_keys)?;

    let mut report = ReportOutput::new(
        "brief",
        "Daily Brief",
        format!(
            "Current project state; completion window {}",
            super::platform::period(*days)?
        ),
    );
    report.request = Some(input.clone());

    let mut lights = Vec::new();
    let mut rows: Vec<Value> = Vec::new();
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for key in project_keys {
        let project = one_project(client, cloud_id, key, *days).await;
        for (metric, value) in &project.metrics {
            *totals.entry(metric.clone()).or_default() += value;
        }
        for (source, warning) in project.warnings {
            report.source(&source, false, warning);
        }
        lights.push(project.light);
        rows.push(project.row);
    }

    report.metrics = totals;
    report
        .highlights
        .insert("Portfolio".into(), worst(&lights).into());
    report.tables.push(table("Project traffic lights", rows));
    Ok(report)
}
