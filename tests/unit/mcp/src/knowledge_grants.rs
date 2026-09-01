//! Translation of source-system permissions into systemprompt subjects — the
//! step that makes REQ-052 enforceable. Unrestricted content gets the
//! source's configured grants (everyone signed in by default); restricted
//! content maps through linked accounts, e-mail, and the configured group
//! map; anything that cannot be mapped is dropped, so a restricted page whose
//! restrictions all fail to map is reachable by nobody.

use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::config::{
    AtlassianSettings, ChunkStrategy, GrantConfig, SourceConfig, SourceKind,
};
use systemprompt_mcp_knowledge_bank::ingest::grants::{
    GrantResolver, IdentityMap, RawRestrictions,
};

fn source(grants: Vec<GrantConfig>) -> SourceConfig {
    SourceConfig {
        kind: SourceKind::Confluence,
        strategy: ChunkStrategy::Script,
        description: String::new(),
        archived: false,
        path: None,
        include: Vec::new(),
        exclude: Vec::new(),
        spaces: vec!["EAICOMM".to_owned()],
        projects: Vec::new(),
        jql: None,
        grants,
    }
}

fn settings() -> AtlassianSettings {
    AtlassianSettings {
        include_comments: true,
        group_map: [("confluence-nava".to_owned(), "Systemprompt-Nava".to_owned())]
            .into_iter()
            .collect(),
    }
}

fn identities() -> IdentityMap {
    IdentityMap::default()
        .with_account("5f1a", UserId::new("u-linked"))
        .with_email("Alice@Astound.com", UserId::new("u-alice"))
}

fn pairs(subjects: &[systemprompt_mcp_knowledge_bank::Subject]) -> Vec<(String, String)> {
    subjects
        .iter()
        .map(|s| (s.kind.clone(), s.value.clone()))
        .collect()
}

#[test]
fn unrestricted_content_is_granted_to_everyone_signed_in_by_default() {
    let resolver = GrantResolver::new(&source(Vec::new()), &settings(), identities());
    assert_eq!(
        pairs(&resolver.resolve(&RawRestrictions::default())),
        vec![("role".to_owned(), "user".to_owned())]
    );
}

#[test]
fn configured_grants_replace_the_default() {
    let resolver = GrantResolver::new(
        &source(vec![GrantConfig {
            kind: "adfs_group".to_owned(),
            value: "Systemprompt-Nava".to_owned(),
        }]),
        &settings(),
        identities(),
    );
    assert_eq!(
        pairs(&resolver.unrestricted()),
        vec![("adfs_group".to_owned(), "Systemprompt-Nava".to_owned())]
    );
}

#[test]
fn restrictions_map_through_linked_accounts_email_and_the_group_map() {
    let resolver = GrantResolver::new(&source(Vec::new()), &settings(), identities());
    let resolved = resolver.resolve(&RawRestrictions {
        account_ids: vec!["5f1a".to_owned()],
        emails: vec!["alice@astound.com".to_owned()],
        group_names: vec!["confluence-nava".to_owned()],
    });
    let got = pairs(&resolved);
    assert!(got.contains(&("user".to_owned(), "u-linked".to_owned())));
    assert!(got.contains(&("user".to_owned(), "u-alice".to_owned())));
    assert!(got.contains(&("adfs_group".to_owned(), "Systemprompt-Nava".to_owned())));
    assert!(
        !got.contains(&("role".to_owned(), "user".to_owned())),
        "a restricted page must not fall back to everyone"
    );
}

#[test]
fn unmappable_restrictions_fail_closed() {
    let resolver = GrantResolver::new(&source(Vec::new()), &settings(), identities());
    let resolved = resolver.resolve(&RawRestrictions {
        account_ids: vec!["unknown-account".to_owned()],
        emails: vec!["nobody@example.invalid".to_owned()],
        group_names: vec!["some-other-group".to_owned()],
    });
    assert!(resolved.is_empty(), "nothing mapped, so nobody may read it");
}

#[test]
fn duplicate_subjects_collapse() {
    let resolver = GrantResolver::new(&source(Vec::new()), &settings(), identities());
    let resolved = resolver.resolve(&RawRestrictions {
        account_ids: vec!["5f1a".to_owned(), "5f1a".to_owned()],
        emails: Vec::new(),
        group_names: Vec::new(),
    });
    assert_eq!(resolved.len(), 1);
}
