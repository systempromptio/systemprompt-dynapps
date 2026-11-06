//! Which class governs a document or folder, and whether a requested change
//! of class is allowed.
//!
//! A target's own class wins, then the nearest classed ancestor folder, then
//! the creator default. A change may only tighten: the requested class must
//! be at least as strict as both the target's current effective class and
//! the class it would inherit, unless an admin loosens it with a reason.
//!
//! A row naming a class the catalog does not declare (a class removed from
//! `classes.yaml`, a hand-edited row) is `ClassSource::Unknown`: it stops the
//! walk at the strictest class, since falling through to a looser ancestor
//! would widen a cap, and agents are capped at `none`. Resolution is `None`
//! only when the catalog declares no classes or no default.

use super::catalog::{ClassCatalog, DocumentClass};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassSource {
    Own,
    Folder(String),
    Default,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedClass<'a> {
    pub class: &'a DocumentClass,
    pub source: ClassSource,
}

/// The class rows that bear on one target: its own, then each ancestor
/// folder nearest first as `(folder_id, class_id)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClassChain {
    pub own: Option<String>,
    pub ancestors: Vec<(String, Option<String>)>,
    pub created_by_agent: Option<String>,
}

impl ClassChain {
    #[must_use]
    pub fn inherited(&self) -> Self {
        Self {
            own: None,
            ..self.clone()
        }
    }
}

fn unknown<'a>(catalog: &'a ClassCatalog, id: &str) -> Option<ResolvedClass<'a>> {
    tracing::error!(
        class_id = id,
        "document class row names an undeclared class; resolving to the strictest"
    );
    catalog.strictest().map(|class| ResolvedClass {
        class,
        source: ClassSource::Unknown(id.to_owned()),
    })
}

#[must_use]
pub fn resolve<'a>(catalog: &'a ClassCatalog, chain: &ClassChain) -> Option<ResolvedClass<'a>> {
    let own = chain.own.as_deref().map(|id| (id, ClassSource::Own));
    let folders = chain.ancestors.iter().filter_map(|(folder, class_id)| {
        class_id
            .as_deref()
            .map(|id| (id, ClassSource::Folder(folder.clone())))
    });
    own.into_iter().chain(folders).next().map_or_else(
        || {
            catalog
                .default_for(chain.created_by_agent.is_some())
                .map(|class| ResolvedClass {
                    class,
                    source: ClassSource::Default,
                })
        },
        |(id, source)| {
            catalog.find(id).map_or_else(
                || unknown(catalog, id),
                |class| Some(ResolvedClass { class, source }),
            )
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Tighten,
    AdminLoosen,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ClassChangeError {
    #[error("unknown document class `{0}`")]
    Unknown(String),
    #[error("`{requested}` is looser than `{floor}`; only an admin may loosen a class")]
    Looser { requested: String, floor: String },
    #[error("loosening a class needs a reason")]
    ReasonRequired,
    #[error("the class catalog declares no default class")]
    NoDefault,
}

#[derive(Debug)]
pub struct ChangeRequest<'r> {
    pub requested: &'r str,
    pub is_admin: bool,
    pub reason: Option<&'r str>,
}

pub fn check_change(
    catalog: &ClassCatalog,
    chain: &ClassChain,
    request: &ChangeRequest<'_>,
) -> Result<ChangeKind, ClassChangeError> {
    let requested = catalog
        .rank(request.requested)
        .ok_or_else(|| ClassChangeError::Unknown(request.requested.to_owned()))?;
    let current = resolve(catalog, chain)
        .ok_or(ClassChangeError::NoDefault)?
        .class;
    let inherited = resolve(catalog, &chain.inherited())
        .ok_or(ClassChangeError::NoDefault)?
        .class;
    let floor = if catalog.rank(&inherited.id) > catalog.rank(&current.id) {
        inherited
    } else {
        current
    };
    let floor_rank = catalog
        .rank(&floor.id)
        .ok_or_else(|| ClassChangeError::Unknown(floor.id.clone()))?;
    if requested >= floor_rank {
        return Ok(ChangeKind::Tighten);
    }
    if !request.is_admin {
        return Err(ClassChangeError::Looser {
            requested: request.requested.to_owned(),
            floor: floor.id.clone(),
        });
    }
    match request.reason.map(str::trim) {
        Some(r) if !r.is_empty() => Ok(ChangeKind::AdminLoosen),
        _ => Err(ClassChangeError::ReasonRequired),
    }
}
