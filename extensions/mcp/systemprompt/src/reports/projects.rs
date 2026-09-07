//! Deterministic delivery metrics from paginated Jira issue facts.

use super::atlassian::{Atlassian, identifier, rows};
use super::shape::table;
use super::{ReportInput, ReportOutput, failure};
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[must_use]
pub fn project_metrics(issues: &[Value], now: DateTime<Utc>, days: u16) -> BTreeMap<String, u64> {
    let mut result = BTreeMap::from([
        ("Issues in sample".into(), 0),
        ("To do".into(), 0),
        ("In progress".into(), 0),
        ("Done".into(), 0),
        ("Overdue unresolved".into(), 0),
        ("Unassigned unresolved".into(), 0),
        ("Unchanged for 7 days".into(), 0),
        ("Completed in period".into(), 0),
    ]);
    let mut add = |key: &str| {
        *result.entry(key.into()).or_default() += 1;
    };
    for issue in issues {
        add("Issues in sample");
        let fields = &issue["fields"];
        let category = fields["status"]["statusCategory"]["key"].as_str();
        match category {
            Some("done") => add("Done"),
            Some("indeterminate") => add("In progress"),
            Some("new") => add("To do"),
            _ => {},
        }
        let done = category == Some("done");
        if matches!(category, Some("new" | "indeterminate")) {
            if fields.get("assignee").is_some_and(Value::is_null) {
                add("Unassigned unresolved");
            }
            if fields["duedate"]
                .as_str()
                .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .is_some_and(|d| d < now.date_naive())
            {
                add("Overdue unresolved");
            }
            if date(&fields["updated"]).is_some_and(|d| d < now - Duration::days(7)) {
                add("Unchanged for 7 days");
            }
        }
        if done
            && date(&fields["resolutiondate"])
                .is_some_and(|d| d >= now - Duration::days(i64::from(days)) && d <= now)
        {
            add("Completed in period");
        }
    }
    result
}

fn date(value: &Value) -> Option<DateTime<Utc>> {
    value
        .as_str()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
}

pub async fn report(
    client: &Atlassian,
    input: &ReportInput,
) -> Result<ReportOutput, rmcp::ErrorData> {
    let ReportInput::Projects {
        cloud_id,
        project_key,
        days,
        space_id,
    } = input
    else {
        return Err(super::invalid("Expected project input"));
    };
    identifier(cloud_id)?;
    identifier(project_key)?;
    if let Some(space) = space_id {
        identifier(space)?;
    }
    let mut report = ReportOutput::new(
        "projects",
        "Project Delivery",
        format!(
            "Current project state; completion window {}",
            super::platform::period(*days)?
        ),
    );
    report.request = Some(input.clone());
    let site_data = client
        .read(&["getAccessibleAtlassianResources"], json!({}))
        .await?;
    let sites = rows(&site_data, &["resources", "items"])?;
    let site = sites
        .iter()
        .find(|s| s["id"].as_str() == Some(cloud_id))
        .ok_or_else(|| failure("Selected Atlassian site is not accessible to this caller"))?;
    let base = site["url"]
        .as_str()
        .filter(|s| s.starts_with("https://"))
        .ok_or_else(|| failure("Atlassian site has no HTTPS URL"))?;
    let issues = collect_issues(client, cloud_id, project_key, &mut report).await?;
    report.metrics = project_metrics(&issues, Utc::now(), *days);
    let issue_rows = issues
        .iter()
        .map(|issue| {
            let f = &issue["fields"];
            let key = issue["key"].as_str().unwrap_or_default();
            json!({"key": key, "summary": f["summary"], "status": f["status"]["name"],
            "assignee": f["assignee"]["displayName"], "due": f["duedate"], "updated": f["updated"],
            "url": format!("{}/browse/{key}", base.trim_end_matches('/'))})
        })
        .collect();
    report.tables.push(table("Jira issues", issue_rows));
    if let Some(space) = space_id {
        add_pages(client, &mut report, cloud_id, space, base).await?;
    }
    Ok(report)
}

async fn collect_issues(
    client: &Atlassian,
    cloud_id: &str,
    project_key: &str,
    report: &mut ReportOutput,
) -> Result<Vec<Value>, rmcp::ErrorData> {
    let mut issues = Vec::new();
    let mut seen = BTreeSet::new();
    let mut cursor: Option<String> = None;
    let mut complete = false;
    for _ in 0..10 {
        let mut args = json!({"cloudId": cloud_id, "jql": format!("project = \"{project_key}\" ORDER BY key ASC"),
            "fields": ["summary","status","assignee","duedate","updated","resolutiondate"], "maxResults":100});
        if let Some(cursor) = &cursor {
            args["nextPageToken"] = json!(cursor);
        }
        let response = match client.read(&["searchJiraIssuesUsingJql"], args).await {
            Ok(response) => response,
            Err(e) if !issues.is_empty() => {
                report.source("Jira issues", false, Some(e.message.to_string()));
                break;
            },
            Err(e) => return Err(e),
        };
        let page = match rows(&response, &["issues", "items"]) {
            Ok(page) => page,
            Err(error) => {
                report.source("Jira response", false, Some(error.message.to_string()));
                break;
            },
        };
        let page_len = page.len();
        for issue in page {
            if let Some(key) = issue["key"].as_str() {
                if seen.insert(key.to_owned()) && issues.len() < 1000 {
                    issues.push(issue);
                }
            } else {
                report.source(
                    "Jira issue",
                    false,
                    Some("An issue without a key was omitted.".into()),
                );
            }
        }
        let next = response["nextPageToken"].as_str().map(str::to_owned);
        if response["isLast"].as_bool() == Some(true)
            || (next.is_none()
                && page_len < 100
                && response["isLast"].as_bool() != Some(false)
                && response["total"]
                    .as_u64()
                    .is_none_or(|total| total <= issues.len() as u64))
        {
            complete = true;
            break;
        }
        if next.is_none() || next == cursor {
            break;
        }
        cursor = next;
    }
    report.source(
        "Jira issues",
        complete,
        (!complete).then(|| {
            "Incomplete issue sample (maximum 1,000); totals describe fetched issues only.".into()
        }),
    );
    check_field_coverage(&issues, report);
    Ok(issues)
}

fn check_field_coverage(issues: &[Value], report: &mut ReportOutput) {
    if issues.iter().any(|issue| {
        issue["fields"]["status"]["statusCategory"]["key"]
            .as_str()
            .is_none()
    }) {
        report.source(
            "Jira field coverage",
            false,
            Some(
                "Some status categories are missing; status-dependent metrics omit those issues."
                    .into(),
            ),
        );
    }
}

async fn add_pages(
    client: &Atlassian,
    report: &mut ReportOutput,
    cloud_id: &str,
    space: &str,
    base: &str,
) -> Result<(), rmcp::ErrorData> {
    let result = client
        .read(
            &["listConfluenceContent", "getPagesInConfluenceSpace"],
            json!({"cloudId":cloud_id,"spaceId":space,"type":"page","limit":50}),
        )
        .await;
    match result {
        Ok(value) => {
            let pages = match rows(&value, &["results", "items", "pages"]) {
                Ok(pages) => pages,
                Err(error) => {
                    report.source("Confluence", false, Some(error.message.to_string()));
                    return Ok(());
                },
            };
            report.source(
                "Confluence",
                pages.len() < 50,
                Some("Pages from the selected space; not inferred project associations.".into()),
            );
            report.tables.push(table(
                "Confluence pages",
                pages
                    .into_iter()
                    .map(|p| {
                        json!({
                            "id":p["id"],"title":p["title"],"url":p["_links"]["webui"].as_str()
                                .map(|path| format!("{}/wiki{}",base.trim_end_matches('/'),path))
                        })
                    })
                    .collect(),
            ));
        },
        Err(e) => report.source("Confluence", false, Some(e.message.to_string())),
    }
    Ok(())
}
