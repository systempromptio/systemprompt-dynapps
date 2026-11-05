//! Document classes: the declared catalog, resolution through folders, the
//! stricter-only rule, the agent-created default, and the policy's cap and
//! capability matrix.

use std::sync::LazyLock;

use systemprompt::files::documents::agents::{AgentActor, AgentCap, AgentCapability, AgentChannel};
use systemprompt_web_admin_platform::document_class::catalog::ClassCatalog;
use systemprompt_web_admin_platform::document_class::movement::{MoveOutcome, on_copy, on_move};
use systemprompt_web_admin_platform::document_class::policy::{allows_in, cap_in};
use systemprompt_web_admin_platform::document_class::resolve::ClassChangeError;
use systemprompt_web_admin_platform::document_class::{
    ChangeKind, ChangeRequest, ClassChain, ClassSource, check_change, resolve,
};

static DECLARED: LazyLock<ClassCatalog> = LazyLock::new(|| {
    ClassCatalog::declared().unwrap_or_else(|e| panic!("services/documents/classes.yaml: {e}"))
});

fn chain(own: Option<&str>, ancestors: &[Option<&str>], by_agent: bool) -> ClassChain {
    ClassChain {
        own: own.map(str::to_owned),
        ancestors: ancestors
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("folder-{i}"), c.map(str::to_owned)))
            .collect(),
        created_by_agent: by_agent.then(|| "test-agent".to_owned()),
    }
}

fn agent(attended: bool) -> AgentActor {
    named("test-agent", attended)
}

fn named(name: &str, attended: bool) -> AgentActor {
    AgentActor {
        name: systemprompt::identifiers::AgentName::try_new(name)
            .expect("valid fixture agent name"),
        attended,
        channel: AgentChannel::Direct,
    }
}

fn on_channel(attended: bool, channel: AgentChannel) -> AgentActor {
    AgentActor {
        channel,
        ..named("sp_bridge", attended)
    }
}

fn change<'r>(requested: &'r str, is_admin: bool, reason: Option<&'r str>) -> ChangeRequest<'r> {
    ChangeRequest {
        requested,
        is_admin,
        reason,
    }
}

#[test]
fn catalog_declares_four_classes_in_strictness_order() {
    let ids: Vec<&str> = DECLARED.classes.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        ["agent-workspace", "internal", "confidential", "restricted"]
    );
}

#[test]
fn cap_matrix_matches_the_declared_classes() {
    let expected = [
        ("agent-workspace", AgentCap::Edit, AgentCap::Edit),
        ("internal", AgentCap::Edit, AgentCap::Suggest),
        ("confidential", AgentCap::Suggest, AgentCap::Suggest),
        ("restricted", AgentCap::Read, AgentCap::None),
    ];
    for (class, attended, unattended) in expected {
        let doc = chain(Some(class), &[], false);
        assert_eq!(
            cap_in(&DECLARED, &doc, &agent(true)),
            attended,
            "{class} attended"
        );
        assert_eq!(
            cap_in(&DECLARED, &doc, &agent(false)),
            unattended,
            "{class} unattended"
        );
    }
}

#[test]
fn nearest_classed_folder_wins() {
    let doc = chain(
        None,
        &[None, Some("confidential"), Some("restricted")],
        false,
    );
    let resolved = resolve(&DECLARED, &doc).unwrap_or_else(|| panic!("unresolved"));
    assert_eq!(resolved.class.id, "confidential");
    assert_eq!(resolved.source, ClassSource::Folder("folder-1".to_owned()));
}

#[test]
fn own_class_beats_folder() {
    let doc = chain(Some("restricted"), &[Some("internal")], false);
    let resolved = resolve(&DECLARED, &doc).unwrap_or_else(|| panic!("unresolved"));
    assert_eq!(resolved.class.id, "restricted");
    assert_eq!(resolved.source, ClassSource::Own);
}

#[test]
fn unclassed_documents_take_the_creator_default() {
    let human =
        resolve(&DECLARED, &chain(None, &[None], false)).unwrap_or_else(|| panic!("unresolved"));
    let agent_doc =
        resolve(&DECLARED, &chain(None, &[None], true)).unwrap_or_else(|| panic!("unresolved"));
    assert_eq!(human.class.id, "internal");
    assert_eq!(agent_doc.class.id, "agent-workspace");
    assert_eq!(agent_doc.source, ClassSource::Default);
}

#[test]
fn folder_class_overrides_the_agent_default() {
    let doc = chain(None, &[Some("confidential")], true);
    assert_eq!(cap_in(&DECLARED, &doc, &agent(false)), AgentCap::Suggest);
}

#[test]
fn owner_may_tighten() {
    let doc = chain(None, &[Some("internal")], false);
    assert_eq!(
        check_change(&DECLARED, &doc, &change("restricted", false, None)),
        Ok(ChangeKind::Tighten)
    );
}

#[test]
fn owner_may_not_loosen_below_the_folder() {
    let doc = chain(None, &[Some("confidential")], false);
    assert!(matches!(
        check_change(&DECLARED, &doc, &change("internal", false, None)),
        Err(ClassChangeError::Looser { .. })
    ));
}

#[test]
fn owner_may_not_loosen_their_own_earlier_choice() {
    let doc = chain(Some("restricted"), &[Some("internal")], false);
    assert!(check_change(&DECLARED, &doc, &change("confidential", false, None)).is_err());
}

#[test]
fn admin_loosens_only_with_a_reason() {
    let doc = chain(None, &[Some("restricted")], false);
    assert_eq!(
        check_change(&DECLARED, &doc, &change("internal", true, Some("  "))),
        Err(ClassChangeError::ReasonRequired)
    );
    assert_eq!(
        check_change(
            &DECLARED,
            &doc,
            &change("internal", true, Some("cleared by legal"))
        ),
        Ok(ChangeKind::AdminLoosen)
    );
}

#[test]
fn unknown_class_is_refused() {
    let doc = chain(None, &[], false);
    assert!(matches!(
        check_change(&DECLARED, &doc, &change("secret", true, Some("x"))),
        Err(ClassChangeError::Unknown(_))
    ));
}

#[test]
fn hard_rules_stay_denied_outside_the_agents_own_drafts() {
    let draft = chain(None, &[], true);
    let internal = chain(None, &[], false);
    let human_in_workspace = chain(None, &[Some("agent-workspace")], false);
    let me = agent(true);
    let allows = |c: &ClassChain, cap, who: &AgentActor| allows_in(&DECLARED, c, cap, who);
    assert!(allows(&draft, AgentCapability::Trash, &me));
    assert!(!allows(&draft, AgentCapability::RestoreRevision, &me));
    assert!(!allows(&draft, AgentCapability::Share, &me));
    assert!(!allows(&draft, AgentCapability::ReplaceContent, &me));
    assert!(!allows(&draft, AgentCapability::CopyOut, &me));
    assert!(!allows(&draft, AgentCapability::ReviewProposal, &me));
    assert!(!allows(
        &draft,
        AgentCapability::Trash,
        &named("other-agent", true)
    ));
    assert!(!allows(&internal, AgentCapability::Trash, &me));
    assert!(!allows(&human_in_workspace, AgentCapability::Trash, &me));
}

#[test]
fn an_undeclared_class_on_the_chain_blocks_agents() {
    for doc in [
        chain(Some("retired-class"), &[Some("agent-workspace")], true),
        chain(None, &[Some("retired-class"), Some("internal")], false),
    ] {
        let resolved = resolve(&DECLARED, &doc).unwrap_or_else(|| panic!("unresolved"));
        assert_eq!(resolved.class.id, "restricted");
        assert!(matches!(resolved.source, ClassSource::Unknown(_)));
        assert_eq!(cap_in(&DECLARED, &doc, &agent(true)), AgentCap::None);
        assert!(!allows_in(
            &DECLARED,
            &doc,
            AgentCapability::Trash,
            &agent(true)
        ));
    }
}

#[test]
fn a_move_into_a_looser_folder_pins_the_current_class() {
    let doc = chain(None, &[Some("restricted")], false);
    let to = vec![("dest".to_owned(), Some("agent-workspace".to_owned()))];
    assert_eq!(
        on_move(&DECLARED, &doc, to, None),
        MoveOutcome::Pin("restricted".to_owned())
    );
}

#[test]
fn a_move_into_a_stricter_folder_or_of_an_own_class_keeps() {
    let inherited = chain(None, &[Some("internal")], false);
    let own = chain(Some("restricted"), &[], false);
    let stricter = vec![("dest".to_owned(), Some("confidential".to_owned()))];
    let looser = vec![("dest".to_owned(), Some("agent-workspace".to_owned()))];
    assert_eq!(
        on_move(&DECLARED, &inherited, stricter, None),
        MoveOutcome::Keep
    );
    assert_eq!(on_move(&DECLARED, &own, looser, None), MoveOutcome::Keep);
}

#[test]
fn a_move_to_the_unclassed_root_pins_an_inherited_class() {
    let doc = chain(None, &[Some("confidential")], false);
    assert_eq!(
        on_move(&DECLARED, &doc, Vec::new(), None),
        MoveOutcome::Pin("confidential".to_owned())
    );
}

#[test]
fn an_admin_with_a_reason_may_move_into_a_looser_folder() {
    let doc = chain(None, &[Some("restricted")], false);
    let to = vec![("dest".to_owned(), Some("internal".to_owned()))];
    assert_eq!(
        on_move(&DECLARED, &doc, to.clone(), Some("declassified")),
        MoveOutcome::AdminLoosen
    );
    assert_eq!(
        on_move(&DECLARED, &doc, to, Some(" ")),
        MoveOutcome::Pin("restricted".to_owned())
    );
}

#[test]
fn an_agents_copy_inherits_the_source_class_not_its_workspace_default() {
    let classed = chain(None, &[Some("confidential")], false);
    let unclassed_human = chain(None, &[], false);
    let agent_copy = chain(None, &[], true);
    assert_eq!(
        on_copy(&DECLARED, &classed, &agent_copy),
        Some("confidential".to_owned())
    );
    assert_eq!(
        on_copy(&DECLARED, &unclassed_human, &agent_copy),
        Some("internal".to_owned())
    );
}

#[test]
fn a_copy_into_a_stricter_folder_stays_under_it() {
    let source = chain(Some("internal"), &[], false);
    let landed = chain(None, &[Some("restricted")], true);
    assert_eq!(on_copy(&DECLARED, &source, &landed), None);
}

#[test]
fn only_sync_and_checkpoints_replace_content_and_only_where_it_may_edit() {
    let sync = on_channel(true, AgentChannel::DriveSync);
    let headless_sync = on_channel(false, AgentChannel::DriveSync);
    let live = on_channel(true, AgentChannel::LiveCheckpoint);
    let direct_bridge = on_channel(true, AgentChannel::Direct);
    let internal = chain(Some("internal"), &[], false);
    let confidential = chain(Some("confidential"), &[], false);
    let replace = |c: &ClassChain, who: &AgentActor| {
        allows_in(&DECLARED, c, AgentCapability::ReplaceContent, who)
    };
    assert!(replace(&internal, &sync));
    assert!(!replace(&internal, &headless_sync));
    assert!(!replace(&confidential, &sync));
    assert!(replace(&internal, &live));
    assert!(!replace(&confidential, &live));
    assert!(!replace(&internal, &direct_bridge));
}

#[test]
fn a_synced_folder_trashes_only_where_the_device_may_edit() {
    let sync = on_channel(true, AgentChannel::DriveSync);
    let headless_sync = on_channel(false, AgentChannel::DriveSync);
    let live = on_channel(true, AgentChannel::LiveCheckpoint);
    let internal = chain(Some("internal"), &[], false);
    let confidential = chain(Some("confidential"), &[], false);
    let trash =
        |c: &ClassChain, who: &AgentActor| allows_in(&DECLARED, c, AgentCapability::Trash, who);
    assert!(trash(&internal, &sync));
    assert!(!trash(&internal, &headless_sync));
    assert!(!trash(&confidential, &sync));
    assert!(!trash(&internal, &live));
}

#[test]
fn only_an_attended_agent_rewrites_broadly_and_only_where_it_may_edit() {
    let broad = |class: &str, who: &AgentActor| {
        let doc = chain(Some(class), &[], false);
        allows_in(&DECLARED, &doc, AgentCapability::BroadRewrite, who)
    };
    assert!(broad("internal", &agent(true)));
    assert!(broad("agent-workspace", &agent(true)));
    assert!(!broad("agent-workspace", &agent(false)));
    assert!(!broad("internal", &agent(false)));
    assert!(!broad("confidential", &agent(true)));
    assert!(!broad("restricted", &agent(true)));
}
