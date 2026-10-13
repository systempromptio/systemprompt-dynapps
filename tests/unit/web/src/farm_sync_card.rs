//! The farm sync card's comparison: a farm's synced commit against the
//! source commit of the bundle that ships its marketplace, and which bundle
//! that is.

use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_admin_farm::views::sync_card::{
    BundleFacts, FarmSyncFacts, SyncState, bundle_for, commits_match, short_sha, sync_state,
};

const SHA: &str = "4f2a9c1e7b3d5a6f8e0c2b4d6a8f0e1c3b5d7a9f";

fn farm(sha: Option<&'static str>, error: Option<&'static str>) -> FarmSyncFacts<'static> {
    FarmSyncFacts {
        synced_commit_sha: sha,
        sync_error: error,
        source_repo: "https://github.com/systempromptio/systemprompt-agent-farm",
        source_ref: "main",
        ..FarmSyncFacts::default()
    }
}

fn bundle(name: &str, marketplaces: &[&str], commit: Option<&str>) -> BundleFacts {
    BundleFacts {
        name: name.to_owned(),
        owner_key: format!("bundle:{name}"),
        marketplaces: marketplaces.iter().map(|m| (*m).to_owned()).collect(),
        source_commit: commit.map(str::to_owned),
        ..BundleFacts::default()
    }
}

#[test]
fn a_farm_never_synced_says_so() {
    assert_eq!(sync_state(&farm(None, None), None), SyncState::NeverSynced);
    assert_eq!(
        sync_state(&farm(Some("  "), None), None),
        SyncState::NeverSynced
    );
}

#[test]
fn a_recorded_sync_error_wins_over_the_commit() {
    let b = bundle("dynapps-agent-farm", &["dynapps-agent-farm"], Some(SHA));
    let state = sync_state(&farm(Some(SHA), Some("clone failed")), Some(&b));
    assert_eq!(state, SyncState::Failed);
    assert_eq!(state.tone(), "err");
}

#[test]
fn the_same_commit_is_in_step_even_abbreviated() {
    let b = bundle(
        "dynapps-agent-farm",
        &["dynapps-agent-farm"],
        Some(&SHA[..7]),
    );
    assert_eq!(
        sync_state(&farm(Some(SHA), None), Some(&b)),
        SyncState::InStep
    );
    let upper = SHA.to_ascii_uppercase();
    let b = bundle("dynapps-agent-farm", &[], Some(&upper));
    assert_eq!(
        sync_state(&farm(Some(SHA), None), Some(&b)),
        SyncState::InStep
    );
}

#[test]
fn a_different_commit_is_behind_with_a_warning_tone() {
    let b = bundle(
        "dynapps-agent-farm",
        &["dynapps-agent-farm"],
        Some("0000000aaaa"),
    );
    let state = sync_state(&farm(Some(SHA), None), Some(&b));
    assert_eq!(state, SyncState::Behind);
    assert_eq!(state.tone(), "warn");
    assert_eq!(state.key(), "behind");
}

#[test]
fn without_a_bundle_commit_there_is_nothing_to_compare() {
    let b = bundle("dynapps-agent-farm", &["dynapps-agent-farm"], None);
    assert_eq!(
        sync_state(&farm(Some(SHA), None), Some(&b)),
        SyncState::Synced
    );
    assert_eq!(sync_state(&farm(Some(SHA), None), None), SyncState::Synced);
}

#[test]
fn commits_match_needs_an_abbreviated_sha_length() {
    assert!(commits_match(SHA, &SHA[..7]));
    assert!(!commits_match(SHA, &SHA[..6]));
    assert!(!commits_match(
        SHA,
        "4f2a9c1e7b3d5a6f8e0c2b4d6a8f0e1c3b5d7a90"
    ));
    assert!(!commits_match("", ""));
}

#[test]
fn short_sha_keeps_seven_characters() {
    assert_eq!(short_sha(SHA), "4f2a9c1");
    assert_eq!(short_sha("abc"), "abc");
}

#[test]
fn bundle_for_prefers_ownership_then_name() {
    let bundles = vec![
        bundle("other", &["x"], None),
        bundle("kit", &["dynapps-agent-farm"], None),
        bundle("dynapps-agent-farm", &[], None),
    ];
    assert_eq!(
        bundle_for(&bundles, &MarketplaceId::new("dynapps-agent-farm")).map(|b| b.name.as_str()),
        Some("kit")
    );
    let named = vec![bundle("dynapps-agent-farm", &[], None)];
    assert_eq!(
        bundle_for(&named, &MarketplaceId::new("dynapps-agent-farm")).map(|b| b.name.as_str()),
        Some("dynapps-agent-farm")
    );
    assert!(bundle_for(&bundles, &MarketplaceId::new("missing")).is_none());
}
