//! Cowork session resolution: the session that names this organization wins
//! over an ambiguous set, and an ambiguous set hands the GUI its candidates.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use systemprompt_bridge::integration::agent_health::{AgentAction, AgentReason, AgentState};
use systemprompt_bridge::integration::cowork_plugins::{
    ResolveTargetError, TargetSource, dir_names_org, match_org, pick_target,
};

const TENANT: &str = "e43527c1-cbb5-4944-b351-4e20e9a4bbc4";

struct Sessions {
    root: tempfile::TempDir,
}

impl Sessions {
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().expect("tempdir"),
        }
    }

    fn add(&self, org: &str, usable: bool) -> (SystemTime, PathBuf) {
        let path = self.root.path().join("03fe0fc5").join(org);
        fs::create_dir_all(&path).expect("mkdir org");
        if usable {
            fs::create_dir_all(path.join("cowork_plugins")).expect("mkdir cowork_plugins");
        }
        (SystemTime::UNIX_EPOCH + Duration::from_secs(1), path)
    }
}

#[test]
fn an_abbreviated_or_full_org_dir_names_the_tenant() {
    assert!(dir_names_org("e43527c1", TENANT));
    assert!(dir_names_org(
        "E43527C1-CBB5-4944-B351-4E20E9A4BBC4",
        TENANT
    ));
    assert!(dir_names_org("e43527c1cbb5", TENANT));
    assert!(!dir_names_org("00000000", TENANT));
    assert!(!dir_names_org("e435", TENANT), "too short to name an org");
    assert!(!dir_names_org("e43527c2", TENANT));
}

#[test]
fn the_tenant_session_wins_where_two_usable_sessions_were_ambiguous() {
    let s = Sessions::new();
    let other = s.add("5d2a9f17", true);
    let tenant = s.add("e43527c1", true);
    let candidates = vec![other, tenant.clone()];

    assert!(matches!(
        pick_target(&candidates),
        Err(ResolveTargetError::Ambiguous { .. })
    ));
    assert_eq!(match_org(&candidates, TENANT), Some(tenant.1));
}

#[test]
fn a_tenant_session_without_a_plugins_dir_is_not_chosen() {
    let s = Sessions::new();
    let candidates = vec![s.add("e43527c1", false), s.add("a1b2c3d4", true)];
    assert_eq!(match_org(&candidates, TENANT), None);
    assert_eq!(
        pick_target(&candidates).expect("resolves"),
        Some((candidates[1].1.clone(), TargetSource::OnlySession)),
        "the only usable session is still chosen"
    );
}

#[test]
fn an_ambiguous_set_carries_every_usable_candidate_for_the_picker() {
    let s = Sessions::new();
    let a = s.add("5d2a9f17", true);
    let b = s.add("f8e4d915", true);
    let unusable = s.add("a1b2c3d4", false);
    let candidates = vec![a.clone(), b.clone(), unusable];

    assert_eq!(match_org(&candidates, TENANT), None);
    match pick_target(&candidates) {
        Err(ResolveTargetError::Ambiguous { candidates }) => {
            assert_eq!(candidates, vec![a.1, b.1]);
        },
        other => panic!("expected Ambiguous, got {other:?}"),
    }
}

#[test]
fn the_ambiguous_verdict_asks_the_person_to_choose() {
    use systemprompt_bridge::integration::ProxyHealth;
    use systemprompt_bridge::integration::agent_health::{
        AgentSurface, HostHealthInputs, HostModelViewRef, PendingStep, verdict,
    };

    let proxy = ProxyHealth::default();
    let base = verdict(&HostHealthInputs {
        snapshot: None,
        apply_failures: &[],
        proxy: &proxy,
        models: HostModelViewRef {
            checked: false,
            available: false,
            unconfigured_providers: &[],
            missing_personal_keys: &[],
            compatible_models: &[],
        },
        has_download_url: false,
        surface: AgentSurface::LocalProfile,
        manifest_synced: true,
        gateway_routed: true,
        can_open: true,
        pending: PendingStep::None,
    });
    let v = base.with_cowork_session_ambiguous();
    assert_eq!(v.state, AgentState::Attention);
    assert!(matches!(v.reason, AgentReason::CoworkSessionAmbiguous));
    assert_eq!(v.action, Some(AgentAction::PickCoworkSession));
}
