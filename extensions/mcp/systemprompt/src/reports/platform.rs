//! Fixed read-only CLI adapters; no caller-controlled commands or connection
//! flags.

use super::shape::table;
use super::{ReportInput, ReportOutput, failure, invalid};
use crate::cli::{self, CliLocation};
use chrono::{Duration, Utc};
// JSON: upstream report envelopes carry heterogeneous table cells.
use serde_json::{Value, json};
use systemprompt::models::artifacts::ChartArtifact;

pub fn normalize_cli(value: &Value) -> Result<Vec<Value>, rmcp::ErrorData> {
    if let Some(items) = value.get("items").and_then(Value::as_array) {
        return Ok(items.clone());
    }
    if let Some(sections) = value.get("sections").and_then(Value::as_array) {
        return Ok(sections.clone());
    }
    if value.get("labels").is_some() && value.get("datasets").is_some() {
        return chart_rows(value);
    }
    if let Some(text) = value.get("content").and_then(Value::as_str) {
        let parsed: Value = serde_json::from_str(text)
            .map_err(|_error| failure("CLI returned text instead of structured report data"))?;
        return normalize_cli(&parsed);
    }
    if let Some(items) = value.as_array() {
        return Ok(items.clone());
    }
    Err(failure("Unsupported CLI report envelope"))
}

fn chart_rows(value: &Value) -> Result<Vec<Value>, rmcp::ErrorData> {
    let chart: ChartArtifact =
        serde_json::from_value(value.clone()).map_err(|_error| failure("Malformed CLI chart"))?;
    if chart
        .datasets
        .iter()
        .any(|dataset| dataset.data.len() != chart.labels.len())
    {
        return Err(failure("CLI chart labels and data lengths disagree"));
    }
    Ok(chart
        .labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let mut row = serde_json::Map::from_iter([("period".into(), json!(label))]);
            for dataset in &chart.datasets {
                row.insert(dataset.label.clone(), json!(dataset.data[index]));
            }
            Value::Object(row)
        })
        .collect())
}

pub(crate) fn period(days: u16) -> Result<String, rmcp::ErrorData> {
    if !(1..=90).contains(&days) {
        return Err(invalid("days must be between 1 and 90"));
    }
    let end = Utc::now();
    Ok(format!(
        "{} to {}",
        (end - Duration::days(i64::from(days))).to_rfc3339(),
        end.to_rfc3339()
    ))
}

struct ReportPlan {
    report: ReportOutput,
    commands: Vec<(&'static str, String, bool)>,
}

fn operation_commands() -> Vec<(&'static str, String, bool)> {
    vec![
        ("Providers", "admin config provider list".into(), false),
        (
            "Gateway routes",
            "admin config gateway route list".into(),
            false,
        ),
        ("Rate limits", "admin config rate-limits show".into(), false),
        ("Agents", "admin agents list".into(), false),
        ("MCP health", "plugins mcp status".into(), false),
        ("Scheduled jobs", "infra jobs list".into(), false),
        (
            "Evaluation runs",
            "admin evals list --limit 100".into(),
            true,
        ),
    ]
}

fn report_plan(input: &ReportInput) -> Result<ReportPlan, rmcp::ErrorData> {
    let (kind, title, range, commands) = match input {
        ReportInput::Activity { days } => (
            "activity",
            "Activity & Governance",
            period(*days)?,
            vec![
                (
                    "Recent requests",
                    format!("infra logs request list --since {days}d --limit 100"),
                    true,
                ),
                (
                    "Request statistics",
                    format!("analytics requests stats --since {days}d"),
                    false,
                ),
            ],
        ),
        ReportInput::Costs { days } => (
            "costs",
            "AI Cost & Adoption",
            period(*days)?,
            vec![
                (
                    "Spend summary",
                    format!("analytics costs summary --since {days}d"),
                    false,
                ),
                (
                    "Models",
                    format!("analytics requests models --since {days}d --limit 100"),
                    true,
                ),
                (
                    "Cost trends",
                    format!("analytics costs trends --since {days}d"),
                    false,
                ),
                (
                    "Sessions",
                    format!("analytics sessions stats --since {days}d"),
                    false,
                ),
            ],
        ),
        ReportInput::Users { offset } => (
            "users",
            "Users & Access",
            "Current accounts; directory page of 100".into(),
            vec![
                (
                    "Directory",
                    format!("admin users list --limit 100 --offset {offset}"),
                    true,
                ),
                ("Account statistics", "admin users stats".into(), false),
            ],
        ),
        ReportInput::Operations {} => (
            "operations",
            "AI Operations",
            "Current configuration and runtime state".into(),
            operation_commands(),
        ),
        ReportInput::Projects { .. } => return Err(invalid("Use the project report adapter")),
    };
    Ok(ReportPlan {
        report: ReportOutput::new(kind, title, range),
        commands,
    })
}

pub(crate) async fn run(
    input: &ReportInput,
    cli: &CliLocation,
    token: &str,
) -> Result<ReportOutput, rmcp::ErrorData> {
    let ReportPlan {
        mut report,
        commands,
    } = report_plan(input)?;
    for (label, command, limited) in commands {
        match read(cli, token, &command).await {
            Ok(rows) => {
                for row in &rows {
                    if let (Some(heading), Some(content)) =
                        (row["heading"].as_str(), row["content"].as_str())
                        && report.highlights.len() < 6
                        && content.len() < 120
                    {
                        report
                            .highlights
                            .insert(heading.to_owned(), content.to_owned());
                    }
                }
                if label == "Directory" {
                    let signed_in = rows
                        .iter()
                        .filter(|row| {
                            row["roles"].as_array().is_some_and(|roles| {
                                roles.iter().any(|role| {
                                    role.as_str().is_some_and(|role| role != "anonymous")
                                })
                            })
                        })
                        .count();
                    report
                        .metrics
                        .insert("Signed-in accounts on this page".into(), signed_in as u64);
                    report
                        .metrics
                        .insert("Directory page size".into(), rows.len() as u64);
                }
                let complete = !limited || rows.len() < 100;
                report.source(
                    &command,
                    complete,
                    limited.then(|| {
                        "Listed rows are a sample/page; use aggregate statistics for totals.".into()
                    }),
                );
                report.tables.push(table(label, rows));
            },
            Err(error) => report.source(&command, false, Some(error.message.to_string())),
        }
    }
    report.request = Some(input.clone());
    Ok(report)
}

async fn read(
    cli: &CliLocation,
    token: &str,
    command: &str,
) -> Result<Vec<Value>, rmcp::ErrorData> {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        cli::execute(cli, command, token),
    )
    .await
    .map_err(|_error| failure("Report source timed out"))??;
    if !result.success {
        return Err(failure(format!(
            "CLI source failed with exit {}",
            result.exit_code
        )));
    }
    let value =
        serde_json::from_str(&result.stdout).map_err(|_error| failure("Malformed CLI JSON"))?;
    normalize_cli(&value)
}
