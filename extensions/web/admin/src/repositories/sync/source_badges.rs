//! The badges on a source row: the one-word state an operator scans for.
//!
//! `in step` is the absence of every other badge: pinned digest equals
//! active digest, the composition is reconciled, the profile names a digest
//! rather than a tag, and the kit carries no access block of its own.

use serde::Serialize;

/// A badge on a source row; `tone` is a design-system tone token.
#[derive(Debug, Clone, Serialize)]
pub struct SourceBadge {
    pub text: &'static str,
    pub tone: &'static str,
    pub detail: String,
}

pub(super) struct BadgeInputs<'a> {
    pub name: &'a str,
    pub pinned_digest: Option<&'a str>,
    pub active_digest: Option<&'a str>,
    pub following_tag: bool,
    pub restart_pending: bool,
    pub kit_access: Vec<&'a str>,
}

pub(super) fn source_badges(input: &BadgeInputs<'_>) -> Vec<SourceBadge> {
    let mut badges = Vec::new();
    match (input.pinned_digest, input.active_digest) {
        (Some(p), Some(a)) if p != a => badges.push(SourceBadge {
            text: "pinned ≠ active",
            tone: "warn",
            detail: format!("profile pins {p}; the active tree was fetched as {a}"),
        }),
        (_, None) => badges.push(SourceBadge {
            text: "not fetched",
            tone: "err",
            detail: "no state for this source — it was never fetched, or the fetch failed"
                .to_owned(),
        }),
        _ => {},
    }
    if input.following_tag {
        badges.push(SourceBadge {
            text: "following a tag",
            tone: "warn",
            detail: "a tag moves; pin @sha256:<digest> so a kit release is a deliberate act"
                .to_owned(),
        });
    }
    if input.restart_pending && input.active_digest.is_some() {
        badges.push(SourceBadge {
            text: "restart pending",
            tone: "info",
            detail: "the composition was refreshed; authz reconciles at the next boot".to_owned(),
        });
    }
    if !input.kit_access.is_empty() {
        badges.push(SourceBadge {
            text: "kit declares access",
            tone: "err",
            detail: format!(
                "the kit's sidecar carries an access block for {} — ignored as a declaration \
                 here; remove it from the kit and declare marketplace/<id> with owner: bundle:{} \
                 in rules.yaml",
                input.kit_access.join(", "),
                input.name
            ),
        });
    }
    if badges.is_empty() {
        badges.push(SourceBadge {
            text: "in step",
            tone: "ok",
            detail: "pinned digest is the active digest and the composition is reconciled"
                .to_owned(),
        });
    }
    badges
}
