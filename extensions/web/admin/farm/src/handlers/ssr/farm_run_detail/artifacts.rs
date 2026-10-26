//! The Artifacts tab: the workspace diff the executor recorded after exit,
//! how the host process ended, and its structured answer. All of it is read
//! from the run's `agent_runs` row.

use serde::Serialize;

use systemprompt::agent_runs::{AgentRunRow, DiffKind, DiffSummary};

use crate::repositories::rows::RunRow;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunArtifactsView {
    pub has_diff: bool,
    pub diff_unreadable: bool,
    pub added: usize,
    pub modified: usize,
    pub deleted: usize,
    pub files: Vec<DiffFileView>,
    pub fixture: Option<String>,
    pub exit_code: Option<i32>,
    pub exit_display: String,
    pub error_class: Option<String>,
    pub error: Option<String>,
    pub run_dir: Option<String>,
    pub has_structured_output: bool,
    pub structured_output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DiffFileView {
    pub path: String,
    pub kind: String,
    pub tone: &'static str,
}

pub(crate) fn artifacts_view(run: &RunRow, agent_run: &AgentRunRow) -> RunArtifactsView {
    let parsed = agent_run
        .diff_summary
        .as_ref()
        .map(|v| serde_json::from_value::<DiffSummary>(v.clone()));
    let diff_unreadable = matches!(parsed, Some(Err(_)));
    let doc = parsed.and_then(Result::ok);
    let has_diff = doc.is_some();
    let doc = doc.unwrap_or_default();
    RunArtifactsView {
        has_diff,
        diff_unreadable,
        added: doc.added,
        modified: doc.modified,
        deleted: doc.deleted,
        files: doc
            .entries
            .into_iter()
            .map(|entry| {
                let (kind, tone) = match entry.kind {
                    DiffKind::Added => ("added", "ok"),
                    DiffKind::Modified => ("modified", "info"),
                    DiffKind::Deleted => ("deleted", "err"),
                };
                DiffFileView {
                    path: entry.path,
                    kind: kind.to_owned(),
                    tone,
                }
            })
            .collect(),
        fixture: run.fixture.clone(),
        exit_code: agent_run.exit_code,
        exit_display: agent_run
            .exit_code
            .map_or_else(|| "—".to_owned(), |c| c.to_string()),
        error_class: run
            .error_class
            .clone()
            .or_else(|| agent_run.error_class.clone()),
        error: agent_run.error.clone(),
        run_dir: agent_run.run_dir.clone(),
        has_structured_output: agent_run.structured_output.is_some(),
        structured_output: agent_run.structured_output.as_ref().map(pretty_json),
    }
}

// JSON: protocol boundary — the host's structured answer, shaped by the
// caller's output schema, shown verbatim.
fn pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}
