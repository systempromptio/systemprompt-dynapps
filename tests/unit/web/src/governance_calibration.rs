use systemprompt::ai::SafetyScanner;
use systemprompt::identifiers::{CallId, SessionId, UserId};
use systemprompt::models::wire::canonical::{
    CanonicalContent, CanonicalMessage, CanonicalRequest, Role,
};
use systemprompt::models::wire::inspect::{SurfaceBudget, string_leaves};
use systemprompt_security::authz::types::Decision;
use systemprompt_security::policy::secrets::{
    EntropyConfig, detect_secrets, redact_spans, secret_findings,
};
use systemprompt_security::policy::types::AccessScope;
use systemprompt_security::policy::{
    AgentScope, GovernanceConfig, GovernanceEngine, GovernedInput, GovernedTarget, McpToolInput,
    PolicyContext,
};
use systemprompt_web_admin::gateway_safety::PiiScanner;

fn engine(mode: &str) -> GovernanceEngine {
    GovernanceEngine::from_config(
        &GovernanceConfig::parse(&format!(
            "governance:\n  mode: {mode}\n  policies:\n    - id: secret_scan\n"
        ))
        .unwrap(),
    )
    .unwrap()
}

fn context<'a>(
    input: &'a GovernedInput,
    user: &'a UserId,
    session: &'a SessionId,
    call: &'a CallId,
) -> PolicyContext<'a> {
    PolicyContext {
        target: GovernedTarget::Prompt,
        agent_scope: AgentScope::User {
            user_id: user.clone(),
        },
        access_scope: AccessScope::User,
        user_id: user,
        session_id: session,
        call_id: call,
        input,
    }
}

#[test]
fn aws_names_and_references_are_not_credentials() {
    for text in [
        "AWS_SECRET_ACCESS_KEY",
        "Do not expose aws_secret_access_key",
        "AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}",
        "aws_secret_access_key =",
        "aws_secret_access_key = sensitive-value",
    ] {
        assert!(
            detect_secrets(&GovernedInput::prompt_text(text.to_owned())).is_none(),
            "{text}"
        );
    }
}

#[test]
fn aws_values_are_precisely_redacted_in_assignment_formats() {
    let key = "Ab9/".repeat(10);
    for text in [
        format!("before AWS_SECRET_ACCESS_KEY={key} after"),
        format!("{{\"aws_secret_access_key\":\"{key}\",\"note\":\"keep\"}}"),
        format!("aws_secret_access_key: '{key}'\nkeep: yes"),
    ] {
        let input = GovernedInput::prompt_text(text.clone());
        let hits = secret_findings(&input, &EntropyConfig::default());
        let aws = hits
            .into_iter()
            .filter(|hit| hit.pattern_id.as_str() == "aws-secret-key")
            .collect::<Vec<_>>();
        assert_eq!(aws.len(), 1);
        assert_eq!(&text[aws[0].span.clone()], key);
        let repaired = redact_spans(&text, aws.into_iter().map(|hit| hit.span)).unwrap();
        assert_eq!(repaired, text.replace(&key, "[REDACTED_BY_GOVERNANCE]"));
    }
}

#[test]
fn structured_aws_value_is_detected_and_can_be_excluded() {
    let input = GovernedInput::tool_arguments(McpToolInput::new(
        serde_json::json!({"AWS_SECRET_ACCESS_KEY": "Ab9/".repeat(10)}),
    ));
    assert_eq!(detect_secrets(&input).unwrap().pattern.id, "aws-secret-key");
    let excluded = vec![systemprompt::identifiers::SecretPatternId::new(
        "aws-secret-key",
    )];
    let entropy = EntropyConfig {
        enabled: false,
        ..Default::default()
    };
    assert!(
        systemprompt_security::policy::secrets::detect_secrets_with_exclusions(
            &input, &entropy, &excluded
        )
        .is_none()
    );
}

#[test]
fn entropy_never_blocks_or_invokes_recovery_but_remains_in_evidence() {
    let input =
        GovernedInput::prompt_text("PHL+ERIbxzlQOeiiRybQwgV7GvYmIclsJe1zsFIyuuM".to_owned());
    let user = UserId::new("calibration-user");
    let session = SessionId::generate();
    let call = CallId::generate();
    for mode in ["warn", "enforce"] {
        let result = engine(mode)
            .evaluate_with_prompt_recovery(&context(&input, &user, &session, &call), |_| {
                panic!("entropy must not redact")
            });
        assert!(matches!(result.decision, Decision::Allow { .. }));
        assert!(
            result
                .chain
                .iter()
                .any(|entry| entry.detail.contains("observation: high-entropy-token"))
        );
    }
}

#[test]
fn confirmed_secret_recovery_is_required_on_every_replayed_turn() {
    let token = format!("ghp_{}", "Ab9x".repeat(9));
    let original = format!("keep this {token} and that");
    let input = GovernedInput::prompt_text(original.clone());
    let user = UserId::new("calibration-user");
    let session = SessionId::generate();
    let engine = engine("enforce");
    for _ in 0..2 {
        let call = CallId::generate();
        let ctx = context(&input, &user, &session, &call);
        let repaired = engine.evaluate_with_prompt_recovery(&ctx, |hits| {
            let text = redact_spans(&original, hits.iter().map(|hit| hit.span.clone())).unwrap();
            assert_eq!(text, "keep this [REDACTED_BY_GOVERNANCE] and that");
            Some(GovernedInput::prompt_text(text))
        });
        assert!(repaired.decision.permits());
        let failed = engine.evaluate_with_prompt_recovery(&ctx, |_| None);
        assert!(matches!(failed.decision, Decision::Deny { .. }));
        let ineffective = engine.evaluate_with_prompt_recovery(&ctx, |_| Some(input.clone()));
        assert!(matches!(ineffective.decision, Decision::Deny { .. }));
    }
}

#[test]
fn warn_mode_never_repairs_a_confirmed_secret() {
    let input = GovernedInput::prompt_text(format!("ghp_{}", "Ab9x".repeat(9)));
    let user = UserId::new("calibration-user");
    let session = SessionId::generate();
    let call = CallId::generate();
    let result = engine("warn")
        .evaluate_with_prompt_recovery(&context(&input, &user, &session, &call), |_| {
            panic!("warn must not mutate")
        });
    assert!(matches!(result.decision, Decision::Warn { .. }));
}

#[test]
fn early_entropy_does_not_hide_a_later_confirmed_credential() {
    let input = GovernedInput::prompt_parts(vec![
        (
            "first".to_owned(),
            "PHL+ERIbxzlQOeiiRybQwgV7GvYmIclsJe1zsFIyuuM".to_owned(),
        ),
        ("second".to_owned(), format!("ghp_{}", "Ab9x".repeat(9))),
    ]);
    assert_eq!(
        detect_secrets(&input).unwrap().pattern.id,
        "github-token-classic"
    );
}

#[tokio::test]
async fn historical_pii_is_separate_in_canonical_and_forwarded_requests() {
    let mut request = CanonicalRequest {
        messages: vec![
            CanonicalMessage {
                role: Role::User,
                content: vec![CanonicalContent::Text("Phone +442079460100".to_owned())],
            },
            CanonicalMessage {
                role: Role::User,
                content: vec![CanonicalContent::Text("Summarize".to_owned())],
            },
        ],
        ..Default::default()
    };
    for forwarded in [false, true] {
        if forwarded {
            request.forwarded_surface = string_leaves(br#"{"messages":[{"role":"user","content":"Phone +442079460100"},{"role":"user","content":"Summarize"}]}"#, SurfaceBudget::default());
        }
        assert!(PiiScanner::new().scan_request(&request).await.is_empty());
        let history = PiiScanner::new().scan_request_history(&request).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].phase, "request_history");
    }
}

#[test]
fn installation_excludes_aws_and_keeps_warn_mode() {
    let cfg = GovernanceConfig::parse(include_str!("../../../../services/governance/config.yaml"))
        .unwrap();
    let engine = GovernanceEngine::from_config(&cfg).unwrap();
    assert!(!engine.enforces_prompt_secrets());
    let exclusions = engine.secret_pattern_exclusions();
    assert!(exclusions.iter().any(|id| id.as_str() == "aws-access-key"));
    assert!(exclusions.iter().any(|id| id.as_str() == "aws-secret-key"));
}
