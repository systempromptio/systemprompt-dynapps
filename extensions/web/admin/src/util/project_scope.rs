//! Resolving which project a listing may span for a given caller.
//!
//! An admin sees every project and may narrow with `?project=`; everyone else
//! is pinned to their own project whatever the query says. Pure, so the rule
//! is pinned by unit tests rather than by a database.

use crate::types::{Project, UserContext};

/// Whether a listing spans every project, is pinned to one, or belongs to a
/// caller who is in none.
///
/// `Nothing` is a non-admin with no project on file — an operator account
/// that never came through AD FS. They match no rows, never widen to `All`:
/// the least-attached caller must not get the widest view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectScope {
    All,
    Only(Project),
    Nothing,
}

impl ProjectScope {
    #[must_use]
    pub fn resolve(user_ctx: &UserContext, requested: Option<&str>) -> Self {
        if user_ctx.is_admin {
            return Project::parse_opt(requested).map_or(Self::All, Self::Only);
        }
        user_ctx.project.map_or(Self::Nothing, Self::Only)
    }

    // Why: The null-skip bind every scoped query takes. `All` becomes `NULL`,
    // which the `($n IS NULL OR upe.project = $n)` predicate reads as "do not
    // narrow"; `Nothing` binds the empty string, which no row can hold.
    #[must_use]
    pub const fn as_sql(&self) -> Option<&str> {
        match self {
            Self::All => None,
            Self::Only(project) => Some(project.as_str()),
            Self::Nothing => Some(""),
        }
    }

    #[must_use]
    pub const fn selected(&self) -> Option<Project> {
        match self {
            Self::Only(project) => Some(*project),
            Self::All | Self::Nothing => None,
        }
    }
}
