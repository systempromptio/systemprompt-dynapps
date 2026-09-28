//! View types for the sync-plane component, shared by every page that
//! renders one: an owner page's Sync tab, the import preview, and the
//! configuration page's state column.
//!
//! Boot seeds a plane only while its projection is empty, and access control
//! counts band rules rather than entities, so entities a later release declares
//! never reach an existing database by themselves; the recommendation names
//! "Apply all new" first for them, since it adds without overwriting.

use serde::Serialize;

use crate::repositories::sync::plane::{ActionCounts, DriftKpi, DriftRow};

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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncRecommendation {
    pub text: String,
    pub card: &'static str,
}

impl SyncRecommendation {
    pub(crate) fn for_actions(actions: &ActionCounts, is_clean: bool) -> Option<Self> {
        if is_clean {
            return None;
        }
        let code = actions.code_side();
        let console = actions.kept + actions.delete_console;
        if let Some(rec) = Self::for_unseedable_entities(actions) {
            return Some(rec);
        }
        let moves = format!(
            "+{} · ~{} · −{}",
            actions.insert + actions.insert_entities,
            actions.update,
            actions.delete
        );
        let text = match (code, console) {
            (true, 0) => {
                format!("Replace database with code brings this database in step ({moves}).")
            },
            (false, n) => format!(
                "{n} row(s) exist only in this database. Export it and commit the file to keep them; nothing here is deleted by code."
            ),
            (true, n) => format!(
                "Replace database with code settles the code side ({moves}); {n} row(s) written here need Export to reach the file first if they are to be kept."
            ),
        };
        Some(Self {
            text,
            card: if code && actions.delete_console == 0 {
                "overwrite"
            } else {
                "export"
            },
        })
    }

    fn for_unseedable_entities(actions: &ActionCounts) -> Option<Self> {
        if actions.insert_entities == 0 {
            return None;
        }
        let deletes = if actions.delete > 0 {
            format!(
                " Replace database with code would also delete {} row(s), so use it only if that is what you want.",
                actions.delete
            )
        } else {
            String::new()
        };
        Some(Self {
            text: format!(
                "{} entity(s) declared in code are absent from this database; Apply all new adds them, with {} row(s), and changes nothing written here.{deletes}",
                actions.insert_entities, actions.insert
            ),
            card: "insert",
        })
    }
}

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
    pub recommendation: Option<SyncRecommendation>,
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
