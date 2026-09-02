//! Per-call dispatch for the knowledge-bank MCP server, driven directly.
//!
//! `call_tool` takes an rmcp `RequestContext<RoleServer>` whose `Peer` only
//! exists once a transport is serving, so `dispatch_tool` is the seam below
//! it: it takes the already authenticated core `RequestContext`, which is what
//! [`Principal::from_request_context`] reads. That makes every retrieval arm
//! reachable here — including the admin gate on `knowledge_index_stats`, the
//! error mapping that decides whether a failure is the caller's fault, and the
//! rendering of each result into the artifact the client actually receives.
//!
//! The retriever is a fake: the backend's own SQL is pinned by the ACL and
//! live-retrieval suites, and what is asserted here is the layer above it.
//! The ingestor and pruner are real but never reached — the arms driven here
//! return before either is touched. Also pinned here: every call leaves a
//! `mcp_tool_executions` row with identity, input, outcome and timing
//! (FR-ACC-3, BR-2, NFR-AUDIT-1); a backend failure never leaks driver text
//! (FR-ACC-5); `sync_knowledge` needs Atlassian headers only for Atlassian
//! categories; and `add_knowledge_document` validates before it touches
//! anything (FR-ING-6, FR-ACC-1).

use std::sync::Arc;

use async_trait::async_trait;
use rmcp::ServerHandler;
use rmcp::model::CallToolRequestParams;
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::{
    Actor, AgentName, ContextId, McpServerId, SessionId, TraceId, UserId,
};
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{ClientProfile, McpArtifactRepository, McpToolExecutor};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::security::authz::DenyAllHook;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::downstream::{
    HEADER_API_TOKEN, HEADER_BASE_URL, HEADER_EMAIL,
};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::ingest::Ingestor;
use systemprompt_mcp_knowledge_bank::maintenance::Pruner;
use systemprompt_mcp_knowledge_bank::model::build_provider;
use systemprompt_mcp_knowledge_bank::principal::Principal;
use systemprompt_mcp_knowledge_bank::retriever::{
    IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, SearchFilter,
};
use systemprompt_mcp_knowledge_bank::server::tool::{Dispatch, dispatch_tool};
use systemprompt_mcp_knowledge_bank::server::{KnowledgeBackend, KnowledgeBankServer};
use systemprompt_mcp_knowledge_bank::tools::{
    ADD_DOCUMENT_TOOL, INDEX_STATS_TOOL, LIST_SOURCES_TOOL, PRUNE_TOOL, SEARCH_TOOL, SYNC_TOOL,
};

// A catalogue with one files category and one Atlassian category, so the
// credential rule of `sync_knowledge` has both sides to show.
const CONFIG: &str = "atlassian: {base_url: https://unreachable.invalid}
files: {root: /nonexistent/knowledge}
sources:
  adrs: {kind: files, strategy: script}
  confluence: {kind: confluence, strategy: script, spaces: [ENG]}
";

use crate::tempdb::TempDb;

// What the fake backend answers with, per method. `Err` carries the error the
// real backend would have raised, so the mapping below it is exercised.
struct FakeRetriever {
    hits: Result<Vec<KnowledgeHit>, KnowledgeBankError>,
    sources: Result<Vec<KnowledgeSource>, KnowledgeBankError>,
    stats: Result<IndexStats, KnowledgeBankError>,
}

impl FakeRetriever {
    fn empty() -> Self {
        Self {
            hits: Ok(Vec::new()),
            sources: Ok(Vec::new()),
            stats: Ok(stats()),
        }
    }
}

fn clone_err(e: &KnowledgeBankError) -> KnowledgeBankError {
    match e {
        KnowledgeBankError::Forbidden(m) => KnowledgeBankError::Forbidden(m.clone()),
        KnowledgeBankError::UnknownSource(m) => KnowledgeBankError::UnknownSource(m.clone()),
        KnowledgeBankError::MissingHeader(m) => KnowledgeBankError::MissingHeader(m),
        KnowledgeBankError::Internal(m) => KnowledgeBankError::Internal(m.clone()),
        other => KnowledgeBankError::Backend(other.to_string()),
    }
}

#[async_trait]
impl KnowledgeRetriever for FakeRetriever {
    async fn search(
        &self,
        _principal: &Principal,
        _query: &str,
        _filter: &SearchFilter,
    ) -> Result<Vec<KnowledgeHit>, KnowledgeBankError> {
        self.hits.as_ref().map(Clone::clone).map_err(clone_err)
    }

    async fn list_sources(
        &self,
        _principal: &Principal,
    ) -> Result<Vec<KnowledgeSource>, KnowledgeBankError> {
        self.sources.as_ref().map(Clone::clone).map_err(clone_err)
    }

    async fn index_stats(&self) -> Result<IndexStats, KnowledgeBankError> {
        self.stats.as_ref().map(Clone::clone).map_err(clone_err)
    }
}

fn hit() -> KnowledgeHit {
    KnowledgeHit {
        doc_id: "doc-1".to_owned(),
        source_type: "confluence".to_owned(),
        source: "ENG".to_owned(),
        title: "Deployment runbook".to_owned(),
        snippet: "Deploy with just deploy-next.".to_owned(),
        uri: "https://example.atlassian.net/wiki/ENG/runbook".to_owned(),
        lines: Some("12-40".to_owned()),
        date: Some("2026-08-30".to_owned()),
        date_source: Some("frontmatter".to_owned()),
        chunk_index: 0,
        chunk_total: 3,
        score: 0.87,
    }
}

fn source() -> KnowledgeSource {
    KnowledgeSource {
        id: "confluence".to_owned(),
        name: "Confluence".to_owned(),
        description: "Engineering spaces".to_owned(),
        archived: false,
        available: true,
        doc_count: Some(42),
        last_synced: Some("2026-08-31T09:00:00Z".to_owned()),
    }
}

fn stats() -> IndexStats {
    IndexStats {
        documents: 120,
        chunks: 4_800,
        grants: 260,
        syncing_users: 3,
        embedding_cache_rows: 4_800,
        database_bytes: 52_428_800,
        last_built: Some("2026-08-31T09:00:00Z".to_owned()),
        version: Some("2026-08-31".to_owned()),
    }
}

fn executor(pool: &Arc<PgPool>) -> McpToolExecutor {
    let db_pool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let usage = Arc::new(ToolUsageRepository::new(&db_pool).expect("tool usage repository"));
    let artifacts = Arc::new(McpArtifactRepository::new(&db_pool).expect("artifact repository"));
    McpToolExecutor::new(usage, artifacts, "knowledge-bank")
}

fn request_context(roles: &[&str]) -> RequestContext {
    let mut ctx = RequestContext::new(
        SessionId::new("kb-session"),
        TraceId::new("kb-trace"),
        ContextId::new_unchecked("00000000-0000-4000-8000-00000000c0de"),
        AgentName::new("kb-agent"),
    )
    .with_actor(Actor::user(UserId::new("kb-caller")));
    let mut user = AuthenticatedUser::new(
        uuid::Uuid::nil(),
        "kb-caller".to_owned(),
        "kb-caller@example.com".to_owned(),
        Vec::new(),
    );
    user.roles = roles.iter().map(|r| (*r).to_owned()).collect();
    ctx.user = Some(user);
    ctx
}

fn call(tool: &'static str, arguments: serde_json::Value) -> CallToolRequestParams {
    let object = arguments
        .as_object()
        .expect("tool arguments are a JSON object")
        .clone();
    CallToolRequestParams::new(tool).with_arguments(object)
}

// Everything `dispatch_tool` borrows, owned for the length of a test.
struct Harness {
    executor: McpToolExecutor,
    ingestor: Arc<Ingestor>,
    pruner: Pruner,
    client: ClientProfile,
}

impl Harness {
    fn new(pool: &Arc<PgPool>) -> Self {
        let config = Arc::new(KnowledgeBankConfig::parse(CONFIG).expect("harness config"));
        let model = build_provider(&config.model, "test-key".to_owned())
            .expect("the default model config builds a provider");
        Self {
            executor: executor(pool),
            ingestor: Arc::new(Ingestor::new(Arc::clone(pool), model, Arc::clone(&config))),
            pruner: Pruner::new(Arc::clone(pool), Arc::clone(pool)),
            client: ClientProfile {
                protocol_version: Some(rmcp::model::ProtocolVersion::V_2025_06_18),
                ..ClientProfile::default()
            },
        }
    }

    async fn dispatch(
        &self,
        retriever: &dyn KnowledgeRetriever,
        ctx: &RequestContext,
        request: &CallToolRequestParams,
        tool: &str,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        self.dispatch_with(retriever, ctx, request, tool, None)
            .await
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors `Dispatch`, which borrows every collaborator a call needs"
    )]
    async fn dispatch_with(
        &self,
        retriever: &dyn KnowledgeRetriever,
        ctx: &RequestContext,
        request: &CallToolRequestParams,
        tool: &str,
        headers: Option<&axum::http::HeaderMap>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        dispatch_tool(
            &Dispatch {
                executor: &self.executor,
                request,
                request_context: ctx,
                client: &self.client,
                headers,
                retriever,
                ingestor: &self.ingestor,
                pruner: &self.pruner,
            },
            tool,
        )
        .await
    }
}

// The text the client is handed, joined across content blocks.
fn rendered(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .flat_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
async fn a_search_hit_is_rendered_with_the_fields_the_agent_ranks_by() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let retriever = FakeRetriever {
        hits: Ok(vec![hit()]),
        ..FakeRetriever::empty()
    };
    let request = call(SEARCH_TOOL, serde_json::json!({ "query": "runbook" }));

    let result = harness
        .dispatch(&retriever, &request_context(&[]), &request, SEARCH_TOOL)
        .await
        .expect("a search with results succeeds");
    let body = rendered(&result);

    // Why: the client ranks on these without parsing the snippet, so each has
    // to survive rendering — the score, the category, the date and its
    // provenance, and the `uri#lines` pointer back to the original.
    assert!(body.contains("[0.87]"), "the score is rendered: {body}");
    assert!(body.contains("Deployment runbook"));
    assert!(body.contains("confluence / ENG"));
    assert!(body.contains("chunk 1/3"), "chunk index is 1-based: {body}");
    assert!(body.contains("date: 2026-08-30 (frontmatter)"));
    assert!(
        body.contains("wiki/ENG/runbook#L12-40"),
        "the line pointer is appended to the uri: {body}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_empty_search_tells_the_caller_to_sync_rather_than_returning_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(SEARCH_TOOL, serde_json::json!({ "query": "nothing here" }));

    let result = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&[]),
            &request,
            SEARCH_TOOL,
        )
        .await
        .expect("an empty search is a success, not an error");
    let body = rendered(&result);

    assert!(body.contains("No results for `nothing here`"), "{body}");
    assert!(
        body.contains("sync_knowledge"),
        "an empty index is the likely cause, so the answer names the fix: {body}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn an_unknown_source_type_is_the_callers_mistake_not_a_server_fault() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let retriever = FakeRetriever {
        hits: Err(KnowledgeBankError::UnknownSource("jira-archive".to_owned())),
        ..FakeRetriever::empty()
    };
    let request = call(
        SEARCH_TOOL,
        serde_json::json!({ "query": "x", "source_types": ["jira-archive"] }),
    );

    let error = harness
        .dispatch(&retriever, &request_context(&[]), &request, SEARCH_TOOL)
        .await
        .expect_err("an unknown source is refused");

    assert_eq!(
        error.code,
        rmcp::model::ErrorCode::INVALID_PARAMS,
        "a bad argument is reported as one, so the agent can correct it"
    );
    assert!(error.message.contains("jira-archive"), "{}", error.message);

    db.cleanup().await;
}

#[tokio::test]
async fn a_backend_failure_is_reported_as_internal_rather_than_blamed_on_the_caller() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let retriever = FakeRetriever {
        hits: Err(KnowledgeBankError::Backend("pgvector is gone".to_owned())),
        ..FakeRetriever::empty()
    };
    let request = call(SEARCH_TOOL, serde_json::json!({ "query": "x" }));

    let error = harness
        .dispatch(&retriever, &request_context(&[]), &request, SEARCH_TOOL)
        .await
        .expect_err("a backend failure surfaces");

    assert_eq!(error.code, rmcp::model::ErrorCode::INTERNAL_ERROR);

    db.cleanup().await;
}

#[tokio::test]
async fn listing_sources_reports_what_the_caller_can_read_and_when_they_synced() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let retriever = FakeRetriever {
        sources: Ok(vec![source()]),
        ..FakeRetriever::empty()
    };
    let request = call(LIST_SOURCES_TOOL, serde_json::json!({}));

    let result = harness
        .dispatch(
            &retriever,
            &request_context(&[]),
            &request,
            LIST_SOURCES_TOOL,
        )
        .await
        .expect("listing sources succeeds");
    let body = rendered(&result);

    // Why: both counts are per-caller under REQ-052, and the wording says so —
    // a shared total here would misreport the index as the caller's view of it.
    assert!(body.contains("42 docs readable by you"), "{body}");
    assert!(body.contains("you synced 2026-08-31T09:00:00Z"), "{body}");
    assert!(body.contains("available"), "{body}");

    db.cleanup().await;
}

#[tokio::test]
async fn an_instance_with_no_sources_says_so_plainly() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(LIST_SOURCES_TOOL, serde_json::json!({}));

    let result = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&[]),
            &request,
            LIST_SOURCES_TOOL,
        )
        .await
        .expect("listing an empty instance succeeds");

    assert!(
        rendered(&result).contains("No knowledge sources are configured"),
        "{}",
        rendered(&result)
    );

    db.cleanup().await;
}

#[tokio::test]
async fn index_stats_are_refused_to_a_caller_without_the_admin_role() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(INDEX_STATS_TOOL, serde_json::json!({}));

    let error = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&["user"]),
            &request,
            INDEX_STATS_TOOL,
        )
        .await
        .expect_err("the shared index's totals are administrative");

    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    assert!(
        error.message.contains("administrators"),
        "the refusal says why: {}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn index_stats_reach_an_administrator_in_human_units() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(INDEX_STATS_TOOL, serde_json::json!({}));

    let result = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&["admin"]),
            &request,
            INDEX_STATS_TOOL,
        )
        .await
        .expect("an administrator may read the index totals");
    let body = rendered(&result);

    assert!(body.contains("documents: 120"), "{body}");
    assert!(body.contains("grants: 260"), "{body}");
    // 52_428_800 bytes is exactly 50 MiB — the report is in MiB, not bytes.
    assert!(body.contains("database_mib: 50.0"), "{body}");

    db.cleanup().await;
}

#[tokio::test]
async fn an_unknown_tool_name_lists_the_tools_that_do_exist() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(SEARCH_TOOL, serde_json::json!({ "query": "x" }));

    let error = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&[]),
            &request,
            "search_everything",
        )
        .await
        .expect_err("an unknown tool name is refused");

    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    for tool in [SEARCH_TOOL, LIST_SOURCES_TOOL, INDEX_STATS_TOOL] {
        assert!(
            error.message.contains(tool),
            "the refusal names {tool}: {}",
            error.message
        );
    }

    db.cleanup().await;
}

// `call_tool`, `initialize` and `list_tools` all take an rmcp
// `RequestContext<RoleServer>` whose `Peer` only exists once a transport is
// serving, so the server itself is asserted through what it can answer without
// one: construction against a live pool, and the identity it advertises.

fn server(pool: &Arc<PgPool>, service_id: &str) -> KnowledgeBankServer {
    let db_pool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let config = Arc::new(KnowledgeBankConfig::default());
    let model = build_provider(&config.model, "test-key".to_owned()).expect("model provider");
    KnowledgeBankServer::new(
        db_pool,
        McpServerId::new(service_id),
        Arc::new(DenyAllHook::null()),
        KnowledgeBackend {
            retriever: Arc::new(FakeRetriever::empty()),
            ingestor: Arc::new(Ingestor::new(Arc::clone(pool), model, Arc::clone(&config))),
            pruner: Arc::new(Pruner::new(Arc::clone(pool), Arc::clone(pool))),
        },
    )
    .expect("construct the knowledge-bank server against a live pool")
}

#[tokio::test]
async fn the_server_builds_its_repositories_from_the_pool_and_advertises_tools() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let info = server(&db.pool, "knowledge-bank").get_info();

    assert!(info.capabilities.tools.is_some(), "it serves tools");
    assert_eq!(info.server_info.name, "Knowledge Bank (knowledge-bank)");
    assert_eq!(
        info.server_info.title.as_deref(),
        Some("Astound Knowledge Bank")
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_instructions_tell_an_agent_how_to_judge_a_hit() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let info = server(&db.pool, "kb-staging").get_info();
    let instructions = info.instructions.expect("the server instructs its callers");

    // Why: the agent policy ranks on recency and provenance, and cites the
    // source. An instruction set that dropped these would leave the model
    // trusting a stale page with no way to check it.
    for expected in ["source_types", "date_source", "uri#lines", "sync_knowledge"] {
        assert!(
            instructions.contains(expected),
            "the instructions mention {expected}: {instructions}"
        );
    }
    assert_eq!(
        info.server_info.name, "Knowledge Bank (kb-staging)",
        "the service id names the instance, not the product"
    );

    db.cleanup().await;
}

// The caller's own Atlassian credentials arrive as headers on the connection;
// nothing else authenticates to Atlassian, so their absence has to be refused
// before any indexing starts.
fn atlassian_headers() -> axum::http::HeaderMap {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        HEADER_EMAIL,
        "kb-caller@example.com".parse().expect("header"),
    );
    headers.insert(HEADER_API_TOKEN, "token-value".parse().expect("header"));
    headers.insert(
        HEADER_BASE_URL,
        "https://unreachable.invalid".parse().expect("header"),
    );
    headers
}

#[tokio::test]
async fn a_sync_without_atlassian_credentials_names_the_header_that_is_missing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(
        SYNC_TOOL,
        serde_json::json!({ "source_types": ["confluence"] }),
    );

    let error = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&[]),
            &request,
            SYNC_TOOL,
        )
        .await
        .expect_err("a sync with no credentials cannot start");

    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    assert!(
        error.message.contains("x-atlassian-email"),
        "the refusal names the header to send: {}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_sync_without_atlassian_headers_syncs_the_files_categories_only() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(SYNC_TOOL, serde_json::json!({}));

    let result = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&["user"]),
            &request,
            SYNC_TOOL,
        )
        .await
        .expect("files need no credential, so an unscoped sync starts");
    let body = rendered(&result);
    assert!(
        body.contains("adrs"),
        "the run is narrowed to the files categories: {body}"
    );
    assert!(
        body.contains("no Atlassian credentials"),
        "and the caller is told why confluence was left out: {body}"
    );

    let files_only = call(SYNC_TOOL, serde_json::json!({ "source_types": ["adrs"] }));
    harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&["user"]),
            &files_only,
            SYNC_TOOL,
        )
        .await
        .expect("an explicit files-only sync needs no headers either");

    db.cleanup().await;
}

#[tokio::test]
async fn a_backend_failure_never_leaks_driver_text_to_the_caller() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let retriever = FakeRetriever {
        hits: Err(KnowledgeBankError::Internal(
            "postgres://knowledge:s3cret@db.internal:5432/knowledge: relation \"knowledge_chunks\" does not exist".to_owned(),
        )),
        ..FakeRetriever::empty()
    };
    let request = call(SEARCH_TOOL, serde_json::json!({ "query": "anything" }));

    let error = harness
        .dispatch(&retriever, &request_context(&[]), &request, SEARCH_TOOL)
        .await
        .expect_err("the failure is reported");

    assert_eq!(error.code, rmcp::model::ErrorCode::INTERNAL_ERROR);
    for fragment in ["s3cret", "db.internal", "relation", "knowledge_chunks"] {
        assert!(
            !error.message.contains(fragment),
            "'{fragment}' reached the caller: {}",
            error.message
        );
    }
    assert!(error.message.contains("logged"), "{}", error.message);

    db.cleanup().await;
}

#[tokio::test]
async fn every_call_leaves_an_execution_audit_row_with_identity_input_and_timing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(
        SEARCH_TOOL,
        serde_json::json!({ "query": "runbook", "top_k": 3 }),
    );
    let retriever = FakeRetriever {
        hits: Ok(vec![hit()]),
        ..FakeRetriever::empty()
    };
    harness
        .dispatch(
            &retriever,
            &request_context(&["user"]),
            &request,
            SEARCH_TOOL,
        )
        .await
        .expect("search succeeds");

    let failing = FakeRetriever {
        hits: Err(KnowledgeBankError::Backend("upstream down".to_owned())),
        ..FakeRetriever::empty()
    };
    harness
        .dispatch(&failing, &request_context(&["user"]), &request, SEARCH_TOOL)
        .await
        .expect_err("the second call fails");

    let rows = sqlx::query_as::<_, (String, String, String, String, Option<i32>)>(
        "SELECT user_id, tool_name, input, status, execution_time_ms \
         FROM mcp_tool_executions WHERE server_name = 'knowledge-bank' ORDER BY started_at",
    )
    .fetch_all(db.pool.as_ref())
    .await
    .expect("audit rows are queryable after the call");
    assert_eq!(rows.len(), 2, "one row per call, success and failure alike");
    let (user, tool, input, status, elapsed) = &rows[0];
    assert_eq!(user, "kb-caller");
    assert_eq!(tool, SEARCH_TOOL);
    assert!(
        input.contains("runbook") && input.contains("\"top_k\":3"),
        "{input}"
    );
    assert_eq!(status, "success");
    assert!(elapsed.is_some(), "timing is recorded");
    assert_eq!(rows[1].3, "failed");

    db.cleanup().await;
}

#[tokio::test]
async fn adding_a_document_is_validated_before_anything_is_touched() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let ctx = request_context(&["user"]);

    let escaping = call(
        ADD_DOCUMENT_TOOL,
        serde_json::json!({ "source_type": "adrs", "file_name": "../etc/passwd.md", "content": "x" }),
    );
    let error = harness
        .dispatch(&FakeRetriever::empty(), &ctx, &escaping, ADD_DOCUMENT_TOOL)
        .await
        .expect_err("a path is not a file name");
    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    assert!(error.message.contains("file_name"), "{}", error.message);

    let atlassian = call(
        ADD_DOCUMENT_TOOL,
        serde_json::json!({ "source_type": "confluence", "file_name": "page.md", "content": "x" }),
    );
    let error = harness
        .dispatch(&FakeRetriever::empty(), &ctx, &atlassian, ADD_DOCUMENT_TOOL)
        .await
        .expect_err("confluence is synced, never uploaded to");
    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    assert!(
        error.message.contains("files categories"),
        "{}",
        error.message
    );

    let overwrite = call(
        ADD_DOCUMENT_TOOL,
        serde_json::json!({ "source_type": "adrs", "file_name": "010.md", "content": "x", "overwrite": true }),
    );
    let error = harness
        .dispatch(&FakeRetriever::empty(), &ctx, &overwrite, ADD_DOCUMENT_TOOL)
        .await
        .expect_err("overwrite is an administrator's act");
    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    assert!(
        error.message.contains("administrators"),
        "{}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_sync_of_a_source_this_instance_does_not_have_is_refused_before_it_starts() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(
        SYNC_TOOL,
        serde_json::json!({ "source_types": ["sharepoint"] }),
    );

    let error = harness
        .dispatch_with(
            &FakeRetriever::empty(),
            &request_context(&[]),
            &request,
            SYNC_TOOL,
            Some(&atlassian_headers()),
        )
        .await
        .expect_err("an unconfigured source is refused");

    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    assert!(error.message.contains("sharepoint"), "{}", error.message);

    db.cleanup().await;
}

#[tokio::test]
async fn pruning_is_refused_to_a_caller_without_the_admin_role() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let harness = Harness::new(&db.pool);
    let request = call(PRUNE_TOOL, serde_json::json!({ "dry_run": true }));

    let error = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&["user"]),
            &request,
            PRUNE_TOOL,
        )
        .await
        .expect_err("the broom is the administrator's");

    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
    assert!(
        error.message.contains("administrators"),
        "{}",
        error.message
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_dry_run_prune_reports_what_it_would_remove_without_removing_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // The broom reads the knowledge tables, which live in their own schema —
    // the shared index is a separate database in production.
    systemprompt_mcp_knowledge_bank::schema::apply(&db.pool)
        .await
        .expect("apply the knowledge schema");
    let harness = Harness::new(&db.pool);
    let request = call(PRUNE_TOOL, serde_json::json!({ "dry_run": true }));

    let result = harness
        .dispatch(
            &FakeRetriever::empty(),
            &request_context(&["admin"]),
            &request,
            PRUNE_TOOL,
        )
        .await
        .expect("an administrator may run the broom");
    let body = rendered(&result);

    // Why: the verb is the whole difference between a report and a deletion,
    // and an administrator reads it before running the same call for real.
    assert!(body.contains("mode: dry run"), "{body}");
    assert!(body.contains("would remove"), "{body}");
    assert!(!body.contains("\nremoved"), "nothing was deleted: {body}");

    db.cleanup().await;
}
