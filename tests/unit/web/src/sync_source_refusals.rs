//! The Sources panel and the activity row over core's refusal record: a source
//! held back on its previous bundle is named with core's reason, and a record
//! that cannot be read is never taken for an empty one.

use std::path::Path;

use systemprompt::identifiers::UserId;
use systemprompt::loader::bundle::{BundleCache, RefusalKind, SourceRefusal};
use systemprompt::manifest::profile::{Profile, ServicesSource};
use systemprompt::manifest::services::ServicesConfig;
use systemprompt_web_admin_core::activity::{NewActivity, SourcesRefresh};
use systemprompt_web_admin_core::repositories::sync::sources::{SourcesView, build_sources};

use crate::boot_fixture::Fixture;

const KIT: &str = "sfcore";
const REASON: &str = "verification failed: bundle requires core >=0.99, this build is 0.87.0";

fn refusal() -> SourceRefusal {
    SourceRefusal {
        source: KIT.to_owned(),
        kind: RefusalKind::RequiresCore,
        reason: REASON.to_owned(),
        attempted_digest: Some("sha256:2222".to_owned()),
        serving_version: "v4.2.0".to_owned(),
        serving_digest: "sha256:1111".to_owned(),
        recorded_at: chrono::Utc::now(),
    }
}

fn served_cache(root: &Path) -> BundleCache {
    let cache = BundleCache::new(root);
    let state = serde_json::from_value(serde_json::json!({
        "composed_hash": "c0de",
        "last_reconciled_hash": "c0de",
        "sources": { KIT: {
            "digest": "sha256:1111",
            "version": "v4.2.0",
            "content_hash": "a".repeat(64),
            "fetched_at": "2026-11-20T10:45:00Z",
        }},
    }))
    .expect("bundle state document");
    cache.write_state(&state).expect("write bundle state");
    cache
}

fn pin(profile: &mut Profile, root: &Path) {
    profile.services.cache_dir = Some(root.display().to_string());
    profile.services.sources = vec![ServicesSource {
        name: KIT.to_owned(),
        https: None,
        oci: None,
    }];
}

fn view(root: &Path) -> SourcesView {
    let boot = Fixture::new().extension_boot_with(|profile| pin(profile, root));
    build_sources(&boot, &ServicesConfig::default())
}

fn badge_texts(view: &SourcesView) -> Vec<&'static str> {
    view.bundles[0].badges.iter().map(|b| b.text).collect()
}

#[test]
fn a_refused_source_is_reported_with_cores_reason() {
    let dir = tempfile::tempdir().expect("cache directory");
    let cache = served_cache(dir.path());
    cache.write_refusals(&[refusal()]).expect("write refusals");

    let view = view(dir.path());
    assert_eq!(view.refusals.unreadable, None);
    assert_eq!(view.refusals.held.len(), 1);
    assert_eq!(view.refusals.held[0].name, KIT);
    assert_eq!(view.refusals.held[0].serving_version, "v4.2.0");
    assert!(view.refusals.held[0].detail.contains(REASON));
    assert_eq!(badge_texts(&view), ["refused"]);
    assert!(view.bundles[0].badges[0].detail.contains(REASON));
}

#[test]
fn an_unreadable_refusal_record_is_unknown_not_empty() {
    let dir = tempfile::tempdir().expect("cache directory");
    served_cache(dir.path());
    std::fs::write(dir.path().join("refusals.json"), "[{").expect("corrupt record");

    let view = view(dir.path());
    assert!(view.refusals.unreadable.is_some());
    assert_eq!(badge_texts(&view), ["refusal unknown"]);
}

#[test]
fn a_source_with_no_refusal_is_not_reported() {
    let dir = tempfile::tempdir().expect("cache directory");
    served_cache(dir.path());

    let view = view(dir.path());
    assert!(view.refusals.held.is_empty());
    assert_eq!(view.refusals.unreadable, None);
    assert_eq!(badge_texts(&view), ["in step"]);
}

#[test]
fn an_import_with_a_refused_source_is_not_recorded_as_unchanged() {
    let refused = [refusal()];
    let row = NewActivity::sources_refreshed(
        &UserId::new("admin"),
        SourcesRefresh {
            changed: false,
            reconciled: false,
            fallback: None,
            refused: &refused,
        },
    );
    assert!(row.description.contains(KIT));
    assert!(row.description.contains(REASON));
    assert!(!row.description.contains("unchanged"));
    assert_eq!(row.metadata["refused"].as_array().map(Vec::len), Some(1));
}
