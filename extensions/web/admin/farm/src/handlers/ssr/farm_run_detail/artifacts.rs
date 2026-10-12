//! The Artifacts tab: the workspace diff the runner recorded after exit, and
//! how the process ended.

use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::ssr::format::format_bytes;

use crate::repositories::rows::RunRow;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunArtifactsView {
    pub has_diff: bool,
    pub diff_unreadable: bool,
    pub added: i64,
    pub modified: i64,
    pub deleted: i64,
    pub files: Vec<DiffFileView>,
    pub fixture: Option<String>,
    pub exit_code: Option<i32>,
    pub exit_display: String,
    pub result_subtype: Option<String>,
    pub error: Option<String>,
    pub run_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DiffFileView {
    pub path: String,
    pub kind: String,
    pub tone: &'static str,
    pub bytes: String,
}

#[derive(Debug, Default, Deserialize)]
struct DiffDoc {
    #[serde(default)]
    added: i64,
    #[serde(default)]
    modified: i64,
    #[serde(default)]
    deleted: i64,
    #[serde(default)]
    files: Vec<DiffFile>,
}

#[derive(Debug, Deserialize)]
struct DiffFile {
    path: String,
    #[serde(alias = "kind")]
    change: String,
    #[serde(default)]
    bytes: Option<i64>,
}

pub(crate) fn artifacts_view(run: &RunRow) -> RunArtifactsView {
    let parsed = run
        .diff_summary
        .as_ref()
        .map(|v| serde_json::from_value::<DiffDoc>(v.clone()));
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
            .files
            .into_iter()
            .map(|f| DiffFileView {
                tone: match f.change.as_str() {
                    "added" => "ok",
                    "deleted" => "err",
                    _ => "info",
                },
                bytes: f.bytes.map_or_else(|| "—".to_owned(), format_bytes),
                path: f.path,
                kind: f.change,
            })
            .collect(),
        fixture: run.fixture.clone(),
        exit_code: run.exit_code,
        exit_display: run
            .exit_code
            .map_or_else(|| "—".to_owned(), |c| c.to_string()),
        result_subtype: run.result_subtype.clone(),
        error: run.error.clone(),
        run_dir: run.run_dir.clone(),
    }
}
