//! View types for the sync-plane component, shared by every page that
//! renders one: an owner page's Sync tab, the import preview, and the
//! configuration page's state column.

use serde::Serialize;

use crate::repositories::sync::plane::{ActionCounts, DriftKpi, DriftRow};

// Why: A hash as the page prints it: the first twelve characters, with the full
// value in a title attribute.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct HashView {
    pub short: String,
    pub full: String,
}

impl HashView {
    pub(crate) fn of(full: &str) -> Option<Self> {
        (!full.is_empty()).then(|| Self {
            short: full.chars().take(12).collect(),
            full: full.to_owned(),
        })
    }
}

// Why: "last applied ab12… by Ed on 16 Sep (overwrite)"; `declared_changed`
// says the declaration's hash differs from the one last applied.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AppliedView {
    pub hash: Option<HashView>,
    pub mode: String,
    pub at: String,
    pub by: String,
    pub base_tree_hash: Option<HashView>,
    pub composed_hash: Option<HashView>,
    pub declared_changed: bool,
}

// Why: `apply_url` is where the buttons post — the plane's own apply, or a
// staged import's — and `show_export` is off on a preview, where the
// database is not the thing being read.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PlaneCardView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub projection: &'static str,
    pub owner_url: &'static str,
    pub runtime_note: Option<&'static str>,
    pub declared_hash: Option<HashView>,
    pub declared_count: usize,
    pub in_db: usize,
    pub kpis: Vec<DriftKpi>,
    pub actions: ActionCounts,
    pub rows: Vec<DriftRow>,
    pub row_count: usize,
    pub is_clean: bool,
    pub unreadable: Option<String>,
    pub applied: Option<AppliedView>,
    pub export_url: String,
    pub apply_url: String,
    pub show_export: bool,
    pub entity_filter: String,
    pub clear_filter_url: String,
    pub source_label: &'static str,
}
