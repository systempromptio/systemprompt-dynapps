//! Every kind of configuration the instance loads from `services/`, as one
//! static list.
//!
//! The configuration page shows what this instance is running, and "what"
//! has to be enumerated somewhere: core discovers most of these by path
//! rather than by a registry, so nothing in the running process can list
//! them. This is that list. A kind is either *projected* — a sync plane
//! owns it and the database is what is enforced — or *served from code*:
//! the composed tree is read at boot or per request and the database never
//! holds a copy. The distinction is the honest one and the page prints it.
//!
//! An import archive is classified against the same list, so an entry the
//! instance cannot project is named for what it is rather than ignored. A
//! path resolves to an exact file first, then to the nearest directory that
//! contains it, so `web/config/groups.yaml` is the groups plane and not "web".

mod kinds;

pub use kinds::CONFIG_KINDS;

/// One kind of configuration under `services/`.
///
/// `path` is relative to `services/` and names a directory when `is_dir`;
/// `plane` is set only for a projected kind, `page_url` only where the
/// console has a page for it.
#[derive(Debug, Clone, Copy)]
pub struct ConfigKind {
    pub id: &'static str,
    pub label: &'static str,
    pub purpose: &'static str,
    pub path: &'static str,
    pub is_dir: bool,
    pub plane: Option<&'static str>,
    pub page_url: Option<&'static str>,
}

const fn file(
    id: &'static str,
    label: &'static str,
    purpose: &'static str,
    path: &'static str,
) -> ConfigKind {
    ConfigKind {
        id,
        label,
        purpose,
        path,
        is_dir: false,
        plane: None,
        page_url: None,
    }
}

const fn dir(
    id: &'static str,
    label: &'static str,
    purpose: &'static str,
    path: &'static str,
) -> ConfigKind {
    ConfigKind {
        id,
        label,
        purpose,
        path,
        is_dir: true,
        plane: None,
        page_url: None,
    }
}

const fn paged(kind: ConfigKind, page_url: &'static str) -> ConfigKind {
    ConfigKind {
        page_url: Some(page_url),
        ..kind
    }
}

const fn projected(kind: ConfigKind, plane: &'static str, page_url: &'static str) -> ConfigKind {
    ConfigKind {
        plane: Some(plane),
        ..paged(kind, page_url)
    }
}

#[must_use]
pub fn kind_by_id(id: &str) -> Option<&'static ConfigKind> {
    CONFIG_KINDS.iter().find(|k| k.id == id)
}

#[must_use]
pub fn kind_for_path(rel: &str) -> Option<&'static ConfigKind> {
    let rel = rel.trim_start_matches("services/").trim_start_matches('/');
    if let Some(k) = CONFIG_KINDS.iter().find(|k| !k.is_dir && k.path == rel) {
        return Some(k);
    }
    CONFIG_KINDS
        .iter()
        .filter(|k| k.is_dir)
        .filter(|k| rel.starts_with(k.path) && rel[k.path.len()..].starts_with('/'))
        .max_by_key(|k| k.path.len())
}
