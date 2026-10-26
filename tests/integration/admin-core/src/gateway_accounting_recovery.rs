//! A failed completion write must retain encrypted evidence and replay exactly
//! once.
use crate::fixtures::{insert_session, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;
use std::sync::Arc;
use systemprompt::ai::models::{
    ClientAttestation, ClientEvidence, ClientKind, InboundWireProtocol, RequestOrigin,
};
use systemprompt::database::{Database, DbPool};
use systemprompt::gateway::captures::CapturedToolUse;
use systemprompt::gateway::protocol::canonical::CanonicalRequest;
use systemprompt::gateway::protocol::inbound::InboundAdapter;
use systemprompt::gateway::protocol::inbound::anthropic_messages::AnthropicMessagesInbound;
use systemprompt::gateway::{GatewayAudit, GatewayRepositories, GatewayRequestContext};
use systemprompt::governance::policy::types::AccessScope;
use systemprompt::identifiers::{AiRequestId, AiToolCallId, ContextId, SessionId, TraceId, UserId};
use systemprompt::wire::canonical::{CanonicalResponse, CanonicalUsage};

fn audit(repos: &GatewayRepositories, user: &UserId, request: &CanonicalRequest) -> GatewayAudit {
    let conversation = request
        .derived_gateway_conversation_id()
        .expect("conversation");
    let ctx = GatewayRequestContext {
        ai_request_id: AiRequestId::generate(),
        user_id: user.clone(),
        session_id: Some(SessionId::generate()),
        context_id: ContextId::derived_from_gateway_conversation(user, &conversation),
        gateway_conversation_id: Some(conversation),
        client_session_id: None,
        trace_id: Some(TraceId::generate()),
        access_scope: AccessScope::Unknown,
        client_id: None,
        provider: "anthropic".into(),
        hop: None,
        model: "test-model".into(),
        requested_model: None,
        max_tokens: Some(16),
        is_streaming: false,
        context_bound: false,
        origin: RequestOrigin::gateway(
            ClientKind::Other,
            InboundWireProtocol::AnthropicMessages,
            ClientAttestation::None,
        ),
        evidence: ClientEvidence::none(),
        access_log: None,
        attribution: systemprompt::models::attribution::RequestAttribution::none(),
        api_key_windows: Vec::new(),
    };
    let audit = GatewayAudit::new(repos, ctx);
    audit
        .pin_pricing(systemprompt::manifest::services::ModelPricing::default())
        .expect("explicit fixture pricing");
    audit
}

#[tokio::test]
async fn terminal_receipt_survives_database_failure_and_replays_without_duplicate_tools() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let fixture = dynapps_test_common::bootstrap_master_key().await;
    let user = insert_user(
        &db.pool,
        &unique("journal-owner"),
        &unclaimed_email("journal"),
    )
    .await;
    let pool: DbPool = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let materializer = Arc::new(systemprompt::agent::services::ContextProviderService::new(
        systemprompt::agent::repository::ContextRepository::new(&pool),
    ));
    let journal = systemprompt::gateway::audit::journal::GatewayJournal::open(
        fixture.path(),
        fixture
            .boot()
            .secrets()
            .encryption_master_key()
            .expect("encryption master key"),
    )
    .expect("gateway journal opens");
    let repos = GatewayRepositories::new(
        &pool,
        journal,
        materializer,
        systemprompt::gateway::GatewayHandles {
            background: systemprompt::traits::BackgroundTasks::new(),
            cache_epochs: systemprompt::traits::CacheEpochs::new(),
            logging: systemprompt::logging::LoggingRuntime::new(
                systemprompt::logging::LogSink::default(),
                Arc::new(systemprompt::manifest::services::SystemAdmin::new(
                    user.clone(),
                    "journal-owner".to_owned(),
                )),
            ),
            services: dynapps_test_common::empty_services_view(),
            subject_providers: systemprompt::governance::authz::SubjectProviderSet::from_providers(
                Vec::new(),
            ),
            at_rest: systemprompt::security::at_rest::AtRestCipher::from_secrets(
                &fixture.boot().secrets().get(),
            )
            .expect("at-rest cipher"),
            unapplied_plane: systemprompt::manifest::profile::UnappliedPlaneMode::Warn,
        },
    );
    let raw = Vec::from(
        br#"{"model":"test-model","max_tokens":16,"messages":[{"role":"user","content":"test"}]}"#
            .as_slice(),
    )
    .into();
    let request = AnthropicMessagesInbound
        .parse_request(&raw)
        .expect("parse request");
    let first = audit(&repos, &user, &request);
    insert_session(
        &db.pool,
        first.ctx.session_id.as_ref().expect("session").as_str(),
        &user,
    )
    .await;
    first.open(&request, &raw).await.expect("admission");
    first
        .commit_admission()
        .await
        .expect("admission committed before dispatch");
    sqlx::raw_sql("CREATE FUNCTION fail_completion() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.status='completed' THEN RAISE EXCEPTION 'injected storage outage'; END IF; RETURN NEW; END $$; CREATE TRIGGER fail_completion BEFORE UPDATE ON ai_requests FOR EACH ROW EXECUTE FUNCTION fail_completion();")
        .execute(&*db.pool).await.expect("install fault");
    let usage = CanonicalUsage {
        input_tokens: 11,
        output_tokens: 7,
        ..Default::default()
    };
    let response = CanonicalResponse {
        id: "response".into(),
        model: "test-model".into(),
        content: vec![],
        stop_reason: None,
        usage,
        grounding: None,
        code_execution: None,
        raw_finish_reason: None,
        received_surface: systemprompt::wire::inspect::ForwardedSurface::default(),
    };
    let body = Vec::from(br#"{"text":"private-recovery-evidence"}"#.as_slice()).into();
    let tools = vec![CapturedToolUse {
        ai_tool_call_id: AiToolCallId::new(unique("tool")),
        tool_name: "Read".into(),
        tool_input: "private-recovery-evidence".into(),
    }];
    assert!(
        first
            .complete(usage, tools, &response, &body)
            .await
            .is_err()
    );
    assert!(
        first
            .fail("late failure must not overwrite completion")
            .await
            .is_err()
    );
    let receipt = std::fs::read_dir(fixture.path().join("gateway-journal"))
        .expect("journal")
        .map(|entry| entry.expect("entry").path())
        .find(|path| path.extension().is_some_and(|ext| ext == "receipt"))
        .expect("durable receipt");
    let bytes = std::fs::read(&receipt).expect("encrypted receipt");
    assert!(
        !bytes
            .windows(b"private-recovery-evidence".len())
            .any(|part| part == b"private-recovery-evidence")
    );
    sqlx::query("DROP TRIGGER fail_completion ON ai_requests")
        .execute(&*db.pool)
        .await
        .expect("restore storage");
    let recovered = systemprompt::gateway::audit::journal::recover(&repos.settlement())
        .await
        .expect("owned recovery settles the durable receipt");
    assert_eq!(recovered, 1);
    let second = audit(&repos, &user, &request);
    insert_session(
        &db.pool,
        second.ctx.session_id.as_ref().expect("session").as_str(),
        &user,
    )
    .await;
    second
        .open(&request, &raw)
        .await
        .expect("admission after receipt recovery");
    second
        .commit_admission()
        .await
        .expect("second admission committed");
    let row:(String,Option<i32>,i64)=sqlx::query_as("SELECT status,tokens_used,(SELECT count(*) FROM ai_request_tool_calls WHERE request_id=r.id) FROM ai_requests r WHERE id=$1")
        .bind(first.ctx.ai_request_id.as_str()).fetch_one(&*db.pool).await.expect("replayed completion");
    assert_eq!(row, ("completed".into(), Some(18), 1));
    assert!(!receipt.exists());
    second
        .fail("fixture complete")
        .await
        .expect("close fixture admission");
    db.cleanup().await;
}
