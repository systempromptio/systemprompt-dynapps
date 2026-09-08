use std::collections::BTreeMap;
use systemprompt_requirements_workflow::model::*;
use systemprompt_requirements_workflow::signing::{ReceiptSigner, verify};

fn workflow() -> Workflow {
    Workflow::new(
        "linen-polos".into(),
        vec![Evidence {
            source_ref: "EAICOMM-1".into(),
            uri: "https://example.test/EAICOMM-1".into(),
            title: "[DEMO] Baseline".into(),
            source_type: "jira".into(),
            body: "RQ-1: preserve category state".into(),
            hash: "test".into(),
            version: "1".into(),
        }],
    )
    .unwrap()
}
fn submit(w: &mut Workflow, p: Phase) {
    let path = match p {
        Phase::Specs => "specs/category/spec.md".into(),
        Phase::Fsd => "fsd/category.md".into(),
        _ => format!("{}.md", p.name()),
    };
    w.submit(p, BTreeMap::from([(path, "# candidate".into())]))
        .unwrap();
}
fn validate(w: &mut Workflow, p: Phase) {
    let dependency_hash = w.dependency_hash(p).unwrap();
    let a = w.artifacts.get_mut(&p).unwrap();
    a.validation = Some(Validation {
        artifact_hash: a.hash.clone(),
        dependency_hash,
        passed: true,
        report: "test validator".into(),
        rubric_commit: RUBRIC_COMMIT.into(),
    });
}
fn grooming() -> Workflow {
    let mut w = workflow();
    for p in [
        Phase::Discovery,
        Phase::Design,
        Phase::Proposal,
        Phase::Grooming,
    ] {
        submit(&mut w, p);
        validate(&mut w, p);
    }
    w
}
fn approve(w: &mut Workflow, gate: Gate) {
    w.request_review(gate).unwrap();
    let signer =
        ReceiptSigner::from_seed_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap();
    let hash = w.bundle_hash(gate).unwrap();
    w.review_mut(gate).unwrap().receipt = Some(signer.sign(&hash).unwrap());
}

#[test]
fn cannot_skip_or_use_unvalidated_predecessors() {
    let mut w = workflow();
    assert!(w.require_predecessors(Phase::Design).is_err());
    submit(&mut w, Phase::Discovery);
    assert!(w.require_predecessors(Phase::Design).is_err());
    validate(&mut w, Phase::Discovery);
    assert!(w.require_predecessors(Phase::Design).is_ok());
}
#[test]
fn pending_grooming_blocks_specs_even_with_valid_artifacts() {
    let mut w = grooming();
    w.request_review(Gate::Grooming).unwrap();
    assert!(w.require_predecessors(Phase::Specs).is_err());
    approve(&mut w, Gate::Grooming);
    assert!(w.require_predecessors(Phase::Specs).is_ok());
}
#[test]
fn revising_upstream_invalidates_downstream_and_approvals() {
    let mut w = grooming();
    approve(&mut w, Gate::Grooming);
    w.submit(
        Phase::Discovery,
        BTreeMap::from([("discovery.md".into(), "changed scope".into())]),
    )
    .unwrap();
    assert_eq!(w.artifacts.len(), 1);
    assert!(w.grooming_review.is_none());
    assert!(w.require_predecessors(Phase::Specs).is_err());
}
#[test]
fn duplicate_submission_preserves_validation_and_receipt() {
    let mut w = grooming();
    approve(&mut w, Gate::Grooming);
    let before = w.bundle_hash(Gate::Grooming).unwrap();
    submit(&mut w, Phase::Discovery);
    assert_eq!(w.bundle_hash(Gate::Grooming).unwrap(), before);
    assert!(w.require_approved(Gate::Grooming).is_ok());
}
#[test]
fn publication_requires_its_own_review() {
    let mut w = grooming();
    approve(&mut w, Gate::Grooming);
    for p in [Phase::Specs, Phase::Fsd] {
        submit(&mut w, p);
        validate(&mut w, p);
    }
    assert!(w.require_approved(Gate::Publication).is_err());
    approve(&mut w, Gate::Publication);
    assert!(w.require_approved(Gate::Publication).is_ok());
}
#[test]
fn forged_approval_fields_are_not_tool_inputs() {
    assert!(
        serde_json::from_value::<systemprompt_requirements_workflow::server::ToolInput>(
            serde_json::json!({"approved":true,"reviewer_id":"human"})
        )
        .is_err()
    );
}
#[test]
fn path_traversal_and_wrong_artifact_ownership_are_rejected() {
    for path in [
        "../discovery.md",
        "/discovery.md",
        "specs/../spec.md",
        "specs/a/../../spec.md",
        "specs/a\\b/spec.md",
    ] {
        assert!(!Phase::Specs.accepts_path(path));
    }
    assert!(!Phase::Discovery.accepts_path("grooming.md"));
    assert!(Phase::Specs.accepts_path("specs/category-state/spec.md"));
}
#[test]
fn signatures_bind_exact_content_and_key() {
    let a =
        ReceiptSigner::from_seed_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap();
    let b =
        ReceiptSigner::from_seed_base64("AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=").unwrap();
    let mut r = a
        .sign(&serde_json::json!({"approved":"content-hash","reviewer":"actual-user"}))
        .unwrap();
    verify(&r, &a.public_key()).unwrap();
    assert!(verify(&r, &b.public_key()).is_err());
    r.payload.push(' ');
    assert!(verify(&r, &a.public_key()).is_err());
}
#[test]
fn evidence_change_makes_existing_receipt_stale() {
    let mut w = grooming();
    approve(&mut w, Gate::Grooming);
    w.evidence_hash = "new-snapshot".into();
    assert!(w.require_approved(Gate::Grooming).is_err());
}
#[test]
fn canonical_hash_ignores_map_insertion_order() {
    assert_eq!(
        digest(&serde_json::json!({"a":1,"b":2})).unwrap(),
        digest(&serde_json::json!({"b":2,"a":1})).unwrap()
    );
}

#[test]
fn confluence_escaping_preserves_reviewed_content_but_link_changes_do_not() {
    use systemprompt_requirements_workflow::publication::canonical_markdown;
    assert_eq!(
        canonical_markdown("[DEMO] Approved source\n"),
        canonical_markdown("\\[DEMO\\] Approved source\n")
    );
    assert_ne!(
        canonical_markdown("[Source](https://example.test/approved)"),
        canonical_markdown("[Source](https://example.test/changed)")
    );
    assert_ne!(
        canonical_markdown("```json\n{\"approved\":true}\n```"),
        canonical_markdown("```json\n{\"approved\":false}\n```")
    );
    assert_ne!(
        canonical_markdown("**required**"),
        canonical_markdown("\\*\\*required\\*\\*")
    );
}
