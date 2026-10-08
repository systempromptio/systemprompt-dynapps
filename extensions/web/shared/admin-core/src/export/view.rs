//! What a page hands its template so the export button and dialog can render.
//!
//! The button is a plain link that works without JavaScript (default columns,
//! the page's own window, CSV); the dialog reads the rest of this context.

use serde::Serialize;
use systemprompt::identifiers::ContextId;

use super::format::Format;
use super::model::{Column, Window};
use super::registry;
use super::transcripts::{DocumentExportView, MAX_CONVERSATIONS, TranscriptSource};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ExportFormatView {
    value: &'static str,
    label: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ExportColumnGroupView {
    name: &'static str,
    columns: Vec<Column>,
}

#[derive(Debug, Serialize)]
pub struct ExportDatasetView {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    window: Window,
    columns: &'static [Column],
    groups: Vec<ExportColumnGroupView>,
}

fn group_columns_in_table_order(columns: &'static [Column]) -> Vec<ExportColumnGroupView> {
    let mut groups: Vec<ExportColumnGroupView> = Vec::new();
    for column in columns {
        match groups.iter_mut().find(|g| g.name == column.group) {
            Some(group) => group.columns.push(*column),
            None => groups.push(ExportColumnGroupView {
                name: column.group,
                columns: vec![*column],
            }),
        }
    }
    groups
}

#[derive(Debug, Serialize)]
pub struct ExportView {
    href: String,
    query: String,
    multiple: bool,
    datasets: Vec<ExportDatasetView>,
    formats: Vec<ExportFormatView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document: Option<DocumentExportView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transcripts: Option<TranscriptsView>,
    filters: Vec<ExportFilterView>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TranscriptsView {
    source: &'static str,
    window: Window,
    cap: i64,
}

#[derive(Debug, Serialize)]
pub struct ExportFilterView {
    key: String,
    label: String,
    value: String,
    pinned: bool,
}

const DIALOG_AND_ORDERING_KEYS: &[&str] = &[
    "preset", "from", "to", "days", "start", "end", "month", "since", "range", "format", "columns",
    "ids", "page", "sort", "dir", "tab", "source",
];
const PAGE_SUBJECT_KEYS: &[&str] = &["context_id", "session_id", "skill", "marketplace", "server"];

fn filter_label(key: &str) -> &str {
    match key {
        "q" | "search" => "Search",
        "user_id" | "user" => "Person",
        "agent_id" => "Agent",
        "agent_scope" => "Agent scope",
        "model" => "Model",
        "provider" => "Provider",
        "status" => "Status",
        "tool" => "Tool",
        "group" => "Group",
        "project" => "Project",
        "scope" => "Scope",
        "side" => "Side calls",
        "error_only" => "Errors only",
        "deny_only" => "Denials only",
        "include_lifecycle" => "Lifecycle events",
        "category" => "Category",
        "outcome" => "Outcome",
        "judged" => "Judged",
        "client" => "Client",
        "flag" => "Flag",
        "policy" => "Policy",
        "decision" => "Decision",
        "attention" => "Needs attention",
        "blocked" => "Blocked",
        "action" => "Action",
        "kind" => "Kind",
        "state" => "State",
        "server" => "Server",
        "context" | "context_id" => "Conversation",
        "session" | "session_id" => "Session",
        "artifact" => "Artifact",
        "marketplace" => "Marketplace",
        "skill" => "Skill",
        "by" => "Breakdown",
        "view" => "View",
        "attr" => "Attribution",
        "axis" => "Axis",
        "audience" => "Audience",
        "filter" => "Filter",
        "role" => "Role",
        "per_skill" => "Per skill",
        other => other,
    }
}

fn filter_chips(query: &str) -> Vec<ExportFilterView> {
    url::form_urlencoded::parse(query.as_bytes())
        .filter(|(k, v)| !v.trim().is_empty() && !DIALOG_AND_ORDERING_KEYS.contains(&k.as_ref()))
        .map(|(k, v)| ExportFilterView {
            label: filter_label(&k).to_owned(),
            pinned: PAGE_SUBJECT_KEYS.contains(&k.as_ref()),
            key: k.into_owned(),
            value: v.into_owned(),
        })
        .collect()
}

impl ExportView {
    #[must_use]
    pub fn new(ids: &[&str], query: &str) -> Self {
        let datasets: Vec<ExportDatasetView> = ids
            .iter()
            .filter_map(|id| registry::find(id))
            .map(|d| ExportDatasetView {
                id: d.id(),
                title: d.title(),
                description: d.description(),
                window: d.window(),
                columns: d.columns(),
                groups: group_columns_in_table_order(d.columns()),
            })
            .collect();
        let first = datasets.first().map_or("", |d| d.id);
        let query = query.trim_start_matches('?').trim_matches('&').to_owned();
        let href = if query.is_empty() {
            format!("/admin/export/{first}?format=csv")
        } else {
            format!("/admin/export/{first}?{query}&format=csv")
        };
        Self {
            href,
            filters: filter_chips(&query),
            query,
            multiple: datasets.len() > 1,
            datasets,
            formats: Format::ALL
                .iter()
                .map(|f| ExportFormatView {
                    value: f.param(),
                    label: f.label(),
                })
                .collect(),
            document: None,
            transcripts: None,
        }
    }

    #[must_use]
    pub fn with_transcripts(mut self, source: TranscriptSource) -> Self {
        if self.datasets.is_empty() {
            let sep = if self.query.is_empty() { "" } else { "&" };
            self.href = format!(
                "/admin/export/transcripts?source={}{sep}{}",
                source.as_str(),
                self.query
            );
        }
        let session_pins_conversations = self.filters.iter().any(|f| f.key == "session_id");
        self.multiple = true;
        self.transcripts = Some(TranscriptsView {
            source: source.as_str(),
            window: if session_pins_conversations {
                Window::None
            } else {
                source.window()
            },
            cap: MAX_CONVERSATIONS,
        });
        self
    }

    #[must_use]
    pub fn conversation(context_id: &ContextId, with_ledger: bool) -> Self {
        let query = format!("context_id={}", urlencoding::encode(context_id.as_str()));
        let ids: &[&str] = if with_ledger {
            &["analysis-conversation-turns"]
        } else {
            &[]
        };
        let mut view = Self::new(ids, &query);
        let document = DocumentExportView::conversation(context_id);
        if !with_ledger {
            view.href.clone_from(&document.json_href);
        }
        view.multiple = true;
        view.document = Some(document);
        view
    }

    #[must_use]
    pub fn single(id: &str, query: &str) -> Self {
        Self::new(&[id], query)
    }
}

#[must_use]
pub fn query_string(pairs: &[(&str, Option<&str>)]) -> String {
    crate::ssr::list_view::query_string_dropping(pairs, &[], &[])
}
