//! The contract every sync plane meets, and the generic drift shape the page
//! renders for any of them.
//!
//! A plane owns one declaration (a file in `services/`) and one projection
//! (tables in the database). The page never learns what either looks like;
//! it asks for the declared hash, the drift as rows, and offers the three
//! directions — insert only, overwrite from code, export — through this
//! trait alone, so adding a plane is one `impl` and one
//! [`super::registry::PlaneRegistration`].
//!
//! Boot goes through the same trait ([`super::boot`]): it seeds an empty
//! projection with *overwrite from code* as [`Actor::Boot`] and
//! otherwise only reads the drift, so every plane behaves the same way on a
//! restart and none of them can quietly rewrite what the console changed.
//!
//! A declaration normally comes from the composed services root; an import
//! preview hands the plane the text of one uploaded file instead
//! ([`DeclarationSource::Text`]). The `_from` methods take that choice and
//! default to refusing text, so a plane that has not opted in still
//! compiles and still syncs from disk. A `Text` preview must leave no trace
//! in `sync_state`, and `apply_scoped` defaults to refusing so a scoped
//! caller can never reach a plane-wide write. A plane `id` is a URL segment
//! and the `sync_state.plane` key, so it is a stable token.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};

pub use super::drift_shape::{ActionCounts, DriftKpi, DriftRow, Export, PlaneDrift, resolution};

pub const SYNC_TAB: &str = "sync";

/// Where a plane reads its declaration: the composed services root (every
/// page and the boot seed), or the text of one uploaded file (an import
/// preview, and the apply chosen from it).
#[derive(Debug, Clone, Copy, Default)]
pub enum DeclarationSource<'a> {
    #[default]
    Disk,
    Text(&'a str),
}

impl DeclarationSource<'_> {
    #[must_use]
    pub const fn is_disk(&self) -> bool {
        matches!(self, Self::Disk)
    }
}

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

/// Who is applying: the boot seed, or a signed-in administrator.
///
/// The boot seed is the one write nobody presses, so it carries a fixed
/// label instead of a person and is recorded as mode `seed` whatever
/// direction it ran in.
#[derive(Debug, Clone, Copy)]
pub enum Actor<'a> {
    Boot,
    User(&'a UserId),
}

impl Actor<'_> {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Boot => super::state::BOOT_ACTOR,
            Self::User(id) => id.as_str(),
        }
    }

    #[must_use]
    pub const fn recorded_mode(&self, mode: SyncMode) -> Option<SyncMode> {
        match self {
            Self::Boot => None,
            Self::User(_) => Some(mode),
        }
    }
}

/// Which entities an apply may touch: the whole plane, or the named
/// `<kind>/<id>` entities alone.
///
/// A marketplace participant syncs their own marketplaces and nothing else,
/// so their apply is scoped; an administrator's is not. A plane whose rows
/// have no entity dimension refuses a scoped apply outright.
#[derive(Debug, Clone, Copy, Default)]
pub enum EntityScope<'a> {
    #[default]
    All,
    Entities(&'a [String]),
}

impl EntityScope<'_> {
    #[must_use]
    pub fn admits(&self, entity_type: &str, entity_id: &str) -> bool {
        match self {
            Self::All => true,
            Self::Entities(keys) => keys
                .iter()
                .any(|k| k.split_once('/') == Some((entity_type, entity_id))),
        }
    }

    #[must_use]
    pub const fn keys(&self) -> &[String] {
        match self {
            Self::All => &[],
            Self::Entities(keys) => keys,
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
    pub entities_retired: usize,
}

#[async_trait]
pub trait SyncPlane: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn source_file(&self) -> &'static str;
    fn projection(&self) -> &'static str;
    fn runtime_note(&self) -> Option<&'static str> {
        None
    }
    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift>;
    async fn apply(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome>;
    async fn apply_scoped(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
        scope: EntityScope<'_>,
    ) -> AdminResult<ApplyOutcome> {
        match scope {
            EntityScope::All => self.apply(pool, mode, actor).await,
            EntityScope::Entities(_) => Err(AdminError::Unprocessable(format!(
                "the {} plane cannot be applied for single entities",
                self.label()
            ))),
        }
    }
    async fn export(&self, pool: &PgPool) -> AdminResult<Option<Export>>;

    fn owner_url(&self) -> &'static str {
        "/admin/sync"
    }

    fn sync_url(&self) -> String {
        format!("{}?tab={SYNC_TAB}", self.owner_url())
    }

    async fn drift_from(
        &self,
        pool: &PgPool,
        from: DeclarationSource<'_>,
    ) -> AdminResult<PlaneDrift> {
        match from {
            DeclarationSource::Disk => self.drift(pool).await,
            DeclarationSource::Text(_) => Err(self.text_unsupported()),
        }
    }

    async fn apply_from(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
        from: DeclarationSource<'_>,
    ) -> AdminResult<ApplyOutcome> {
        match from {
            DeclarationSource::Disk => self.apply(pool, mode, actor).await,
            DeclarationSource::Text(_) => Err(self.text_unsupported()),
        }
    }

    fn text_unsupported(&self) -> AdminError {
        AdminError::Unprocessable(format!(
            "the {} plane can only sync from the services tree, not from an uploaded file",
            self.label()
        ))
    }
}
