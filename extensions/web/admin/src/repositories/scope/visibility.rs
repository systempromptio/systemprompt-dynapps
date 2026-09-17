//! Resolving which users a listing may span for a given caller.
//!
//! Two layers. [`Visibility`] is what the caller is allowed to see and is a
//! pure function of their identity; [`ScopeRequest`] adds the group and
//! project filters a query string asked for. Resolving the pair against the
//! database yields a [`SubjectScope`] — either every user, or an explicit id
//! list — which is what scoped queries bind.
//!
//! Narrowing to a user id list rather than to a column keeps the rule in one
//! place: a listing joins whatever it likes and adds one `user_id = ANY($n)`,
//! instead of every query restating what membership means.

use crate::types::UserContext;

/// What a caller may see before any query filter is applied.
///
/// `Scoped` is a non-console caller: the groups and projects their marketplace
/// participation names, plus themselves. Empty containers still carry the
/// caller, so a person no marketplace reaches sees their own rows and nothing
/// else — never the instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    All,
    Scoped {
        groups: Vec<String>,
        projects: Vec<String>,
        self_id: String,
    },
}

impl Visibility {
    #[must_use]
    pub fn for_user(user_ctx: &UserContext) -> Self {
        if user_ctx.is_console {
            Self::All
        } else {
            Self::Scoped {
                groups: user_ctx.participation.group_ids.clone(),
                projects: user_ctx.participation.project_ids.clone(),
                self_id: user_ctx.user_id.as_str().to_owned(),
            }
        }
    }
}

/// A listing's resolved scope: what the caller may see, narrowed by what they
/// asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeRequest {
    pub visibility: Visibility,
    pub group: Option<String>,
    pub project: Option<String>,
}

// Why: a requested container the caller may not see is ignored rather than
// refused. The query string is not an authorisation surface, so a scoped
// caller asking for someone else's group or project gets their own view
// back, never an error that would tell them the container exists.
fn narrow(visible: Option<&[String]>, asked: Option<&str>) -> Option<String> {
    match (visible, asked) {
        (_, None) => None,
        (None, Some(v)) => Some(v.to_owned()),
        (Some(own), Some(v)) => own.iter().find(|o| o.as_str() == v).cloned(),
    }
}

impl ScopeRequest {
    #[must_use]
    pub fn from_query(user_ctx: &UserContext, group: Option<&str>, project: Option<&str>) -> Self {
        let visibility = Visibility::for_user(user_ctx);
        let (group, project) = match &visibility {
            Visibility::All => (narrow(None, group), narrow(None, project)),
            Visibility::Scoped {
                groups, projects, ..
            } => (narrow(Some(groups), group), narrow(Some(projects), project)),
        };
        Self {
            visibility,
            group,
            project,
        }
    }

    // Why: The group ids a query must narrow to, or `None` for every group.
    #[must_use]
    pub fn group_filter(&self) -> Option<Vec<String>> {
        match (&self.visibility, self.group.as_ref()) {
            (Visibility::All, None) => None,
            (Visibility::All, Some(group)) => Some(vec![group.clone()]),
            (Visibility::Scoped { groups, .. }, None) => Some(groups.clone()),
            (Visibility::Scoped { groups, .. }, Some(group)) => {
                Some(groups.iter().filter(|o| *o == group).cloned().collect())
            },
        }
    }

    // Why: The project ids a query must narrow to, or `None` for every project.
    #[must_use]
    pub fn project_filter(&self) -> Option<Vec<String>> {
        match (&self.visibility, self.project.as_ref()) {
            (Visibility::All, None) => None,
            (Visibility::All, Some(project)) => Some(vec![project.clone()]),
            (Visibility::Scoped { projects, .. }, None) => Some(projects.clone()),
            (Visibility::Scoped { projects, .. }, Some(project)) => {
                Some(projects.iter().filter(|o| *o == project).cloned().collect())
            },
        }
    }

    // Why: the caller's own id when their view is scoped, so their own rows
    // are always inside it; `None` for a console caller who sees everyone.
    #[must_use]
    pub fn self_id(&self) -> Option<&str> {
        match &self.visibility {
            Visibility::All => None,
            Visibility::Scoped { self_id, .. } => Some(self_id.as_str()),
        }
    }

    #[must_use]
    pub const fn is_scoped(&self) -> bool {
        matches!(self.visibility, Visibility::Scoped { .. })
    }
}

/// The users a scoped query may return.
///
/// `All` binds NULL, which every scoped predicate reads as "do not narrow".
/// `Users` binds the id list, and an empty list is a legitimate answer that
/// matches nothing — the least-attached caller must not widen to everything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectScope {
    All,
    Users(Vec<String>),
}

impl SubjectScope {
    #[must_use]
    pub const fn as_sql(&self) -> Option<&[String]> {
        match self {
            Self::All => None,
            Self::Users(ids) => Some(ids.as_slice()),
        }
    }
}

// Why: whether one owner's rows are inside the caller's view — the question a
// detail page asks before rendering a session, context, trace or request the
// header lookup landed on. A console caller sees every owner; a scoped caller
// resolves their unfiltered view once and checks membership. An unknown owner
// is outside every scoped view.
pub async fn may_view(
    pool: &sqlx::PgPool,
    user_ctx: &UserContext,
    owner: Option<&systemprompt::identifiers::UserId>,
) -> Result<bool, sqlx::Error> {
    if user_ctx.is_console {
        return Ok(true);
    }
    let Some(owner) = owner else {
        return Ok(false);
    };
    let request = ScopeRequest::from_query(user_ctx, None, None);
    let scope = super::membership::get_subject_scope(pool, &request).await?;
    Ok(match scope {
        SubjectScope::All => true,
        SubjectScope::Users(ids) => ids.iter().any(|id| id == owner.as_str()),
    })
}
