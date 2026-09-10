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

fn cost_commands(days: u16) -> Vec<(&'static str, String, bool)> {
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
    ]
}

fn report_plan(input: &ReportInput) -> Result<ReportPlan, rmcp::ErrorData> {
    let days = input.days_or_default();
    let super::ReportKind::Costs = input.report;
    let (kind, title, range, commands) = (
        "costs",
        "AI Usage & Cost",
        period(days)?,
        cost_commands(days),
    );
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
    report.request = Some(*input);
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
