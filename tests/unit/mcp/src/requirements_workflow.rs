use std::collections::BTreeMap;
use systemprompt_requirements_workflow::model::*;
use systemprompt_requirements_workflow::signing::{ReceiptSigner, verify};

fn workflow() -> Workflow {
    Workflow::new(
        "linen-polos".into(),
        "Preserve category browsing behavior".into(),
        vec![Evidence {
            source_ref: "EAICOMM-1".into(),
            uri: "https://example.test/EAICOMM-1".into(),
            title: "[DEMO] Baseline".into(),
            source_type: "jira".into(),
            body: "RQ-1: preserve category state".into(),
            hash: "test".into(),
            indexed_hash: "test".into(),
            retrieval_method: "fixture".into(),
            version: "1".into(),
        }],
    )
    .unwrap()
}
fn submit(w: &mut Workflow, p: Phase) {
    let path = format!("{}.md", p.name());
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
        validator_version: VALIDATOR_VERSION.into(),
    });
}
fn context() -> Workflow {
    let mut w = workflow();
    submit(&mut w, Phase::Context);
    validate(&mut w, Phase::Context);
    w
}
fn approve(w: &mut Workflow, gate: Gate) {
    if gate == Gate::Publication {
        w.set_destination(destination()).unwrap();
    }
    w.request_review(gate).unwrap();
    let signer =
        ReceiptSigner::from_seed_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap();
    let hash = w.bundle_hash(gate).unwrap();
    w.review_mut(gate).unwrap().receipt = Some(signer.sign(&hash).unwrap());
}

#[test]
fn cannot_skip_or_use_unvalidated_predecessors() {
    let mut w = workflow();
    assert!(w.require_predecessors(Phase::Specification).is_err());
    submit(&mut w, Phase::Context);
    assert!(w.require_predecessors(Phase::Specification).is_err());
    validate(&mut w, Phase::Context);
    assert!(w.require_predecessors(Phase::Specification).is_err());
}
#[test]
fn pending_context_blocks_specs_even_with_valid_artifacts() {
    let mut w = context();
    w.request_review(Gate::Context).unwrap();
    assert!(w.require_predecessors(Phase::Specification).is_err());
    approve(&mut w, Gate::Context);
    assert!(w.require_predecessors(Phase::Specification).is_ok());
}
#[test]
fn revising_upstream_invalidates_downstream_and_approvals() {
    let mut w = context();
    approve(&mut w, Gate::Context);
    w.submit(
        Phase::Context,
        BTreeMap::from([("context.md".into(), "changed scope".into())]),
    )
    .unwrap();
    assert_eq!(w.artifacts.len(), 1);
    assert!(w.context_review.is_none());
    assert!(w.require_predecessors(Phase::Specification).is_err());
}
#[test]
fn duplicate_submission_preserves_validation_and_receipt() {
    let mut w = context();
    approve(&mut w, Gate::Context);
    let before = w.bundle_hash(Gate::Context).unwrap();
    submit(&mut w, Phase::Context);
    assert_eq!(w.bundle_hash(Gate::Context).unwrap(), before);
    assert!(w.require_approved(Gate::Context).is_ok());
}
#[test]
fn publication_requires_its_own_review() {
    let mut w = context();
    approve(&mut w, Gate::Context);
    for p in [Phase::Specification] {
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
        "../context.md",
        "/context.md",
        "specs/../spec.md",
        "specs/a/../../spec.md",
        "specs/a\\b/spec.md",
    ] {
        assert!(!Phase::Specification.accepts_path(path));
    }
    assert!(!Phase::Context.accepts_path("specification.md"));
    assert!(Phase::Specification.accepts_path("specification.md"));
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
    let mut w = context();
    approve(&mut w, Gate::Context);
    w.evidence_hash = "new-snapshot".into();
    assert!(w.require_approved(Gate::Context).is_err());
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

fn destination() -> Destination {
    Destination {
        cloud_id: "cloud".into(),
        space_id: "space".into(),
        parent_id: "parent".into(),
        title: "Category requirements".into(),
    }
}

#[test]
fn destination_change_invalidates_only_publication_approval() {
    let mut w = context();
    approve(&mut w, Gate::Context);
    submit(&mut w, Phase::Specification);
    validate(&mut w, Phase::Specification);
    approve(&mut w, Gate::Publication);
    let mut changed = destination();
    changed.parent_id = "different-parent".into();
    w.set_destination(changed).unwrap();
    assert!(w.require_approved(Gate::Context).is_ok());
    assert!(w.require_approved(Gate::Publication).is_err());
    assert!(w.require_validated(Phase::Specification).is_ok());
}

#[test]
fn publication_review_requires_destination() {
    let mut w = context();
    approve(&mut w, Gate::Context);
    submit(&mut w, Phase::Specification);
    validate(&mut w, Phase::Specification);
    assert!(w.request_review(Gate::Publication).is_err());
}

#[test]
fn identical_destination_preserves_approval() {
    let mut w = context();
    approve(&mut w, Gate::Context);
    submit(&mut w, Phase::Specification);
    validate(&mut w, Phase::Specification);
    approve(&mut w, Gate::Publication);
    w.set_destination(destination()).unwrap();
    assert!(w.require_approved(Gate::Publication).is_ok());
}

#[test]
fn deterministic_validation_does_not_claim_semantic_quality() {
    let mut w = workflow();
    w.submit(
        Phase::Context,
        BTreeMap::from([(
            "context.md".into(),
            format!(
                "# Context\n{}\n[Source](https://example.test/EAICOMM-1)",
                "Reviewed scope. ".repeat(12)
            ),
        )]),
    )
    .unwrap();
    let validation =
        systemprompt_requirements_workflow::validation::validate(&w, Phase::Context).unwrap();
    assert!(validation.passed);
    let report: serde_json::Value = serde_json::from_str(&validation.report).unwrap();
    assert_eq!(report["semantic_status"], "not_scored");
    w.submit(
        Phase::Context,
        BTreeMap::from([("context.md".into(), "Unsupported claim. ".repeat(12))]),
    )
    .unwrap();
    assert!(
        !systemprompt_requirements_workflow::validation::validate(&w, Phase::Context)
            .unwrap()
            .passed
    );
}
