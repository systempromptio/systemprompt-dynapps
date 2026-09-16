//! The contract every sync plane meets, and the generic drift shape the page
//! renders for any of them.
//!
//! A plane owns one declaration (a file in `services/`) and one projection
//! (tables in the database). The page never learns what either looks like;
//! it asks for the declared hash, the drift as rows, and offers the three
//! directions — insert only, overwrite from code, export — through this
//! trait alone, so adding a plane is one `impl` and one registry line.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::AdminResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    InsertOnly,
    Overwrite,
}

impl SyncMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::InsertOnly => "insert_only",
            Self::Overwrite => "overwrite",
        }
    }

    #[must_use]
    pub const fn human(self) -> &'static str {
        match self {
            Self::InsertOnly => "insert only",
            Self::Overwrite => "overwrite from code",
        }
    }
}

/// What one apply moved.
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct ApplyOutcome {
    pub inserted: usize,
    pub updated: usize,
    pub deleted: usize,
    pub entities_inserted: usize,
    pub entities_updated: usize,
}

/// One tile above the drift table.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DriftKpi {
    pub label: &'static str,
    pub value: usize,
    pub note: &'static str,
    pub tone: &'static str,
}

/// One line of a plane's diff table. `kind` is a stable token the page and
/// the tests key on; the labels are for people.
#[derive(Debug, Clone, Serialize)]
pub struct DriftRow {
    pub kind: &'static str,
    pub kind_label: &'static str,
    pub kind_tone: &'static str,
    pub entity_type: String,
    pub entity_type_label: String,
    pub entity_id: String,
    pub band: String,
    pub band_label: &'static str,
    pub subject: String,
    pub in_code: String,
    pub in_db: String,
    pub origin: &'static str,
    pub origin_tone: &'static str,
    pub governed: bool,
    pub insert_applies: bool,
    pub overwrite_applies: bool,
    pub overwrite_effect: &'static str,
}

/// The numbers the action buttons and their confirm dialog quote: what
/// *Insert only* adds (rows, entity defaults), what *Overwrite* corrects and
/// deletes, and how many of the deletions the console wrote.
#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct ActionCounts {
    pub insert: usize,
    pub insert_entities: usize,
    pub update: usize,
    pub delete: usize,
    pub delete_console: usize,
}

/// A plane's declared-versus-database picture, ready to render. `unreadable`
/// carries the message when the declaration could not be read; it is the
/// card's headline, not an error.
#[derive(Debug, Default, Clone, Serialize)]
pub struct PlaneDrift {
    pub declared_hash: String,
    pub declared_count: usize,
    pub in_db: usize,
    pub kpis: Vec<DriftKpi>,
    pub actions: ActionCounts,
    pub rows: Vec<DriftRow>,
    pub is_clean: bool,
    pub unreadable: Option<String>,
}

/// A declaration rendered back out of the database as its file.
#[derive(Debug, Clone, Serialize)]
pub struct Export {
    pub filename: &'static str,
    pub content_type: &'static str,
    pub body: String,
    pub row_count: usize,
}

#[async_trait]
pub trait SyncPlane: Send + Sync {
    // Why: `id` is a URL segment and the `sync_state.plane` key, so it is a
    // stable token; `source_file` is relative to `services/`; `projection`
    // names the table(s) for the card.
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn source_file(&self) -> &'static str;
    fn projection(&self) -> &'static str;
    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift>;
    async fn apply(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: &UserId,
    ) -> AdminResult<ApplyOutcome>;
    // Why: `None` for a plane whose declaration has no file form.
    async fn export(&self, pool: &PgPool) -> AdminResult<Option<Export>>;
}
