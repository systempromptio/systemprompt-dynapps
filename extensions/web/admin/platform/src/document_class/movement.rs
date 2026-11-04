//! What a move or a copy does to a class.
//!
//! A move must never loosen what agents may do: when the destination would
//! govern a target more loosely than its current class, that class is pinned
//! onto the target as its own row (a folder's pin covers everything that
//! inherits through it). An admin who gives a reason may let the move loosen.
//! A copy is never looser than its source, so copying never launders a
//! restricted document into an agent workspace: the copy keeps its source's
//! class wherever it would otherwise resolve more loosely, and a copy into a
//! stricter folder stays under that folder. A move's destination arrives as
//! its folders nearest first, each with its class.

use super::catalog::ClassCatalog;
use super::resolve::{ClassChain, ClassSource, resolve};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveOutcome {
    Keep,
    Pin(String),
    AdminLoosen,
}

#[must_use]
pub fn on_move(
    catalog: &ClassCatalog,
    current: &ClassChain,
    to: Vec<(String, Option<String>)>,
    admin_reason: Option<&str>,
) -> MoveOutcome {
    let Some(now) = resolve(catalog, current) else {
        return MoveOutcome::Keep;
    };
    if now.source == ClassSource::Own {
        return MoveOutcome::Keep;
    }
    let moved = ClassChain {
        ancestors: to,
        ..current.clone()
    };
    let after = resolve(catalog, &moved);
    let looser = after.is_none_or(|a| {
        matches!(now.source, ClassSource::Unknown(_))
            || catalog.rank(&a.class.id) < catalog.rank(&now.class.id)
    });
    if !looser {
        return MoveOutcome::Keep;
    }
    if admin_reason.is_some_and(|r| !r.trim().is_empty()) {
        return MoveOutcome::AdminLoosen;
    }
    MoveOutcome::Pin(now.class.id.clone())
}

#[must_use]
pub fn on_copy(catalog: &ClassCatalog, source: &ClassChain, copy: &ClassChain) -> Option<String> {
    let from = resolve(catalog, source)?;
    let landed = resolve(catalog, copy);
    let stricter = landed.is_none_or(|l| {
        matches!(from.source, ClassSource::Unknown(_))
            || catalog.rank(&from.class.id) > catalog.rank(&l.class.id)
    });
    stricter.then(|| from.class.id.clone())
}
