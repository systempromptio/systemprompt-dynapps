//! Row builders for the admin repository suite.
//!
//! The throwaway database is **not empty**: `install_extension_schemas` runs
//! the web extension's seeds, so tests assert on their own UUID-suffixed rows
//! and on deltas, never on absolute counts or empty lists.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};
use systemprompt_web_admin_core::repositories::scope::SubjectScope;
use systemprompt_web_admin_core::util::time_range::{TimeRange, TimeRangePreset};
use systemprompt_web_shared::{GroupId, ProjectId};

// Starts after any seeded `ai_requests` row, so a windowed query sees only
// what the test inserted.
pub(crate) fn narrow_window() -> TimeRange {
    let now = Utc::now();
    TimeRange {
        from: now - Duration::seconds(30),
        to: now + Duration::hours(1),
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    }
}

pub(crate) fn wide_window() -> TimeRange {
    let now = Utc::now();
    TimeRange {
        from: now - Duration::hours(1),
        to: now + Duration::hours(1),
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    }
}

pub(crate) fn unique_group(prefix: &str) -> GroupId {
    GroupId::new(unique(prefix))
}

pub(crate) fn unique_project(prefix: &str) -> ProjectId {
    ProjectId::new(unique(prefix))
}

pub(crate) fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4().simple())
}

// `ContextId::new` panics on anything that is not a UUID v4.
pub(crate) fn new_context_id() -> String {
    ContextId::generate().to_string()
}

pub(crate) fn unclaimed_email(local: &str) -> String {
    format!("{local}@{}.example", uuid::Uuid::new_v4().simple())
}

// `users.name` is unique and `users.email` must be lowercase and trimmed, so
// the email doubles as the name.
// The administrator console writes are attributed to.
pub(crate) fn actor() -> UserId {
    UserId::new("console-test-admin")
}

pub(crate) async fn insert_user(pool: &PgPool, id: &str, email: &str) -> UserId {
    insert_user_full(pool, id, email, Some(email), &["user".to_owned()], "active").await
}

#[expect(
    clippy::too_many_arguments,
    reason = "one parameter per column of the row being seeded; a params struct would churn 20 call sites to say the same thing"
)]
pub(crate) async fn insert_user_full(
    pool: &PgPool,
    id: &str,
    email: &str,
    display_name: Option<&str>,
    roles: &[String],
    status: &str,
) -> UserId {
    static SERVICES: std::sync::Once = std::sync::Once::new();
    SERVICES.call_once(|| {
        if systemprompt::loader::ServicesBootstrap::is_initialized() {
            return;
        }
        let directory = tempfile::tempdir().expect("services fixture");
        let path = directory.path().join("config.yaml");
        std::fs::write(&path, "{}").expect("write services fixture");
        systemprompt::loader::ServicesBootstrap::init_from_path(&path)
            .expect("install services fixture");
    });

    sqlx::query(
        "INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
         VALUES ($1, $2, $3, $4, $5, true, $6)",
    )
    .bind(id)
    .bind(email)
    .bind(email)
    .bind(display_name)
    .bind(status)
    .bind(roles)
    .execute(pool)
    .await
    .expect("insert user");
    UserId::new(id.to_owned())
}

// `None` removes the user from every group and project.
pub(crate) async fn set_project<S: AsRef<str> + Sync + ?Sized>(
    pool: &PgPool,
    user_id: &UserId,
    project: Option<&S>,
) {
    sqlx::query("DELETE FROM project_members WHERE user_id = $1")
        .bind(user_id.as_str())
        .execute(pool)
        .await
        .expect("clear project members");
    sqlx::query("DELETE FROM group_members WHERE user_id = $1")
        .bind(user_id.as_str())
        .execute(pool)
        .await
        .expect("clear group members");
    let Some(project) = project else {
        return;
    };
    let project = project.as_ref();
    insert_project(pool, project, project).await;
    insert_project_member(pool, project, user_id, "manual").await;
    insert_group(pool, project, project).await;
    insert_group_member(pool, project, user_id, "manual").await;
}

pub(crate) async fn project_scope(pool: &PgPool, project: impl AsRef<str>) -> SubjectScope {
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT user_id FROM project_members WHERE project_id = $1")
            .bind(project.as_ref())
            .fetch_all(pool)
            .await
            .expect("project scope");
    SubjectScope::Users(ids)
}

pub(crate) async fn group_scope(pool: &PgPool, group: impl AsRef<str>) -> SubjectScope {
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT user_id FROM user_groups WHERE group_id = $1")
            .bind(group.as_ref())
            .fetch_all(pool)
            .await
            .expect("group scope");
    SubjectScope::Users(ids)
}

// `session_id` is a foreign key to `user_sessions`: pass `None` unless that
// row exists.
pub(crate) async fn insert_context(
    pool: &PgPool,
    context_id: &str,
    user_id: &UserId,
    session_id: Option<&str>,
    name: &str,
) {
    sqlx::query(
        "INSERT INTO user_contexts (context_id, user_id, session_id, name) VALUES ($1, $2, $3, $4)",
    )
    .bind(context_id)
    .bind(user_id.as_str())
    .bind(session_id)
    .bind(name)
    .execute(pool)
    .await
    .expect("insert user context");
}

pub(crate) struct SummarySpec<'a> {
    pub session_id: &'a str,
    pub user_id: &'a UserId,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub tool_uses: i64,
    pub prompts: i64,
    pub errors: i64,
    pub model: Option<&'a str>,
    pub status: Option<&'a str>,
    pub ai_title: Option<&'a str>,
}

impl<'a> SummarySpec<'a> {
    pub(crate) fn open(session_id: &'a str, user_id: &'a UserId) -> Self {
        Self {
            session_id,
            user_id,
            started_at: Some(Utc::now() - Duration::hours(1)),
            ended_at: None,
            tool_uses: 0,
            prompts: 0,
            errors: 0,
            model: None,
            status: None,
            ai_title: None,
        }
    }
}

pub(crate) async fn insert_summary(pool: &PgPool, spec: &SummarySpec<'_>) {
    sqlx::query(
        "INSERT INTO plugin_session_summaries
             (id, session_id, user_id, started_at, ended_at, tool_uses, prompts, errors,
              model, status, ai_title, total_events)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $6 + $7)",
    )
    .bind(unique("summary"))
    .bind(spec.session_id)
    .bind(spec.user_id.as_str())
    .bind(spec.started_at)
    .bind(spec.ended_at)
    .bind(spec.tool_uses)
    .bind(spec.prompts)
    .bind(spec.errors)
    .bind(spec.model)
    .bind(spec.status)
    .bind(spec.ai_title)
    .execute(pool)
    .await
    .expect("insert session summary");
}

// `ai_requests.session_id` is a foreign key to `user_sessions`.
pub(crate) async fn insert_session(pool: &PgPool, session_id: &str, user_id: &UserId) {
    sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id)
        .bind(user_id.as_str())
        .execute(pool)
        .await
        .expect("insert user session");
}

pub(crate) const LEGACY_CONTEXT_ID: &str = "00000000-0000-0000-0000-4c4547414359";

pub(crate) struct RequestSpec<'a> {
    pub id: String,
    pub user_id: &'a UserId,
    pub session_id: Option<&'a str>,
    pub client_session_id: Option<&'a str>,
    pub trace_id: Option<&'a str>,
    pub context_id: Option<&'a str>,
    pub provider: &'a str,
    pub model: &'a str,
    pub status: &'a str,
    pub input_tokens: i32,
    pub output_tokens: i32,
    // Not always `input + output`: it counts cache reads, and a provider with
    // no total leaves it NULL. Both shapes must be seedable.
    pub tokens_used: Option<i32>,
    pub reasoning_tokens: Option<i32>,
    pub cache_read_tokens: Option<i32>,
    pub cache_creation_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: i32,
    pub created_at: DateTime<Utc>,
}

impl<'a> RequestSpec<'a> {
    pub(crate) fn completed(id: &str, user_id: &'a UserId) -> Self {
        Self {
            id: id.to_owned(),
            user_id,
            session_id: None,
            client_session_id: None,
            trace_id: None,
            // ai_requests.context_id is NOT NULL; core's sentinel for no context.
            context_id: Some(LEGACY_CONTEXT_ID),
            provider: "anthropic",
            model: "claude-test-model",
            status: "completed",
            input_tokens: 100,
            output_tokens: 20,
            tokens_used: Some(120),
            reasoning_tokens: None,
            cache_read_tokens: None,
            cache_creation_tokens: None,
            cost_microdollars: 5_000,
            latency_ms: 250,
            created_at: Utc::now(),
        }
    }
}

// The conversation view treats a lone request with no offered tools as a
// side call. Core stores the tool list in `ai_tool_catalogs` by digest, so
// the catalogue row comes first.
pub(crate) async fn insert_offered_tools(pool: &PgPool, request_id: &str) {
    let tools = serde_json::json!([{"name": "bash"}]);
    let sha256 = format!("{:064x}", 0xba5e_ba11u64);
    sqlx::query(
        "INSERT INTO ai_tool_catalogs (sha256, tools) VALUES ($1, $2) \
         ON CONFLICT (sha256) DO NOTHING",
    )
    .bind(&sha256)
    .bind(&tools)
    .execute(pool)
    .await
    .expect("insert tool catalogue");

    sqlx::query(
        "INSERT INTO ai_request_payloads (ai_request_id, offered_tools_sha256) VALUES ($1, $2)",
    )
    .bind(request_id)
    .bind(&sha256)
    .execute(pool)
    .await
    .expect("insert ai request payload");
}

pub(crate) async fn insert_request(pool: &PgPool, spec: &RequestSpec<'_>) {
    sqlx::query(
        "INSERT INTO ai_requests (
             id, request_id, user_id, session_id, trace_id, context_id,
             provider, model, input_tokens, output_tokens, tokens_used,
             reasoning_tokens, cache_read_tokens, cache_creation_tokens,
             cost_microdollars, latency_ms, status, actor_kind, actor_id,
             created_at, updated_at, client_session_id)
         VALUES ($1, $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                 $14, $15, $16, 'user', $2, $17, $17, $18)",
    )
    .bind(&spec.id)
    .bind(spec.user_id.as_str())
    .bind(spec.session_id)
    .bind(spec.trace_id)
    .bind(spec.context_id)
    .bind(spec.provider)
    .bind(spec.model)
    .bind(spec.input_tokens)
    .bind(spec.output_tokens)
    .bind(spec.tokens_used)
    .bind(spec.reasoning_tokens)
    .bind(spec.cache_read_tokens)
    .bind(spec.cache_creation_tokens)
    .bind(spec.cost_microdollars)
    .bind(spec.latency_ms)
    .bind(spec.status)
    .bind(spec.created_at)
    .bind(spec.client_session_id)
    .execute(pool)
    .await
    .expect("insert ai_request");
}

pub(crate) struct DecisionSpec<'a> {
    pub id: String,
    pub user_id: &'a UserId,
    pub session_id: &'a str,
    pub tool_name: &'a str,
    pub agent_id: Option<&'a str>,
    pub agent_scope: Option<&'a str>,
    pub decision: &'a str,
    pub policy: &'a str,
    pub reason: &'a str,
    pub trace_id: Option<&'a str>,
    pub entity_type: Option<&'a str>,
    pub created_at: DateTime<Utc>,
}

impl<'a> DecisionSpec<'a> {
    pub(crate) fn allow(id: &str, user_id: &'a UserId, session_id: &'a str) -> Self {
        Self {
            id: id.to_owned(),
            user_id,
            session_id,
            tool_name: "Bash",
            agent_id: None,
            agent_scope: None,
            decision: "allow",
            policy: "scope_check",
            reason: "within scope",
            trace_id: None,
            entity_type: None,
            created_at: Utc::now(),
        }
    }
}

pub(crate) async fn insert_decision(pool: &PgPool, spec: &DecisionSpec<'_>) {
    sqlx::query(
        "INSERT INTO governance_decisions (
             id, user_id, session_id, context_id, tool_name, agent_id, agent_scope,
             decision, policy, reason, actor_kind, actor_id, created_at,
             trace_id, evaluated_rules)
         VALUES ($1, $2, $3, $11, $4, $5, $6, $7, $8, $9, 'user', $2, $10,
                 $12,
                 CASE WHEN $13::TEXT IS NULL THEN '[]'::JSONB
                      ELSE jsonb_build_object('entity_type', $13::TEXT) END)",
    )
    .bind(&spec.id)
    .bind(spec.user_id.as_str())
    .bind(spec.session_id)
    .bind(spec.tool_name)
    .bind(spec.agent_id)
    .bind(spec.agent_scope)
    .bind(spec.decision)
    .bind(spec.policy)
    .bind(spec.reason)
    .bind(spec.created_at)
    .bind(LEGACY_CONTEXT_ID)
    .bind(spec.trace_id)
    .bind(spec.entity_type)
    .execute(pool)
    .await
    .expect("insert governance decision");
}

pub(crate) struct EventSpec<'a> {
    pub id: String,
    pub user_id: &'a UserId,
    pub session_id: &'a str,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub created_at: DateTime<Utc>,
}

impl<'a> EventSpec<'a> {
    pub(crate) fn tool_use(id: &str, user_id: &'a UserId, session_id: &'a str) -> Self {
        Self {
            id: id.to_owned(),
            user_id,
            session_id,
            event_type: "claude_code_PostToolUse",
            tool_name: Some("Bash"),
            created_at: Utc::now(),
        }
    }
}

pub(crate) async fn insert_event(pool: &PgPool, spec: &EventSpec<'_>) {
    sqlx::query(
        "INSERT INTO plugin_usage_events
             (id, user_id, session_id, event_type, tool_name, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&spec.id)
    .bind(spec.user_id.as_str())
    .bind(spec.session_id)
    .bind(spec.event_type)
    .bind(spec.tool_name)
    .bind(spec.created_at)
    .execute(pool)
    .await
    .expect("insert plugin usage event");
}

pub(crate) async fn insert_activity(
    pool: &PgPool,
    id: &str,
    user_id: &UserId,
    category: &str,
    action: &str,
) {
    sqlx::query(
        "INSERT INTO user_activity (id, user_id, category, action, entity_name, description)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(user_id.as_str())
    .bind(category)
    .bind(action)
    .bind("fixture-entity")
    .bind("fixture activity")
    .execute(pool)
    .await
    .expect("insert user activity");
}

#[expect(
    clippy::too_many_arguments,
    reason = "one parameter per column of the row being seeded; a params struct would churn 42 call sites to say the same thing"
)]
pub(crate) async fn insert_acl_rule(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
    rule_type: &str,
    rule_value: &str,
    access: &str,
) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ($1, $2, false, 'code:test-fixture')
         ON CONFLICT (entity_type, entity_id) DO NOTHING",
    )
    .bind(entity_type)
    .bind(entity_id)
    .execute(pool)
    .await
    .expect("insert access control entity");

    sqlx::query(
        "INSERT INTO access_control_rules
             (id, entity_type, entity_id, rule_type, rule_value, access, source)
         VALUES ($1, $2, $3, $4, $5, $6, 'code:test-fixture')",
    )
    .bind(unique("rule"))
    .bind(entity_type)
    .bind(entity_id)
    .bind(rule_type)
    .bind(rule_value)
    .bind(access)
    .execute(pool)
    .await
    .expect("insert access control rule");
}

pub(crate) async fn insert_federated_identity(
    pool: &PgPool,
    issuer: &str,
    external_sub: &str,
    user_id: &UserId,
) {
    sqlx::query(
        "INSERT INTO federated_identities (issuer, external_sub, user_id) VALUES ($1, $2, $3)",
    )
    .bind(issuer)
    .bind(external_sub)
    .bind(user_id.as_str())
    .execute(pool)
    .await
    .expect("insert federated identity");
}

pub(crate) async fn insert_message(
    pool: &PgPool,
    request_id: &str,
    seq: i32,
    role: &str,
    body: &str,
) {
    sqlx::query(
        "INSERT INTO ai_request_messages (id, request_id, role, content, sequence_number)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(unique("msg"))
    .bind(request_id)
    .bind(role)
    .bind(body)
    .bind(seq)
    .execute(pool)
    .await
    .expect("insert ai request message");
}

pub(crate) async fn insert_group(pool: &PgPool, id: impl AsRef<str>, name: &str) {
    sqlx::query(
        "INSERT INTO groups (id, name, source) VALUES ($1, $2, 'dashboard')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(id.as_ref())
    .bind(name)
    .execute(pool)
    .await
    .expect("insert group");
}

// 'adfs' and 'manual' rows are separate by primary key, which is what lets a
// manual membership survive the directory's replace.
pub(crate) async fn insert_group_member(
    pool: &PgPool,
    group_id: impl AsRef<str>,
    user_id: &UserId,
    source: &str,
) {
    sqlx::query(
        "INSERT INTO group_members (group_id, user_id, source) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(group_id.as_ref())
    .bind(user_id.as_str())
    .bind(source)
    .execute(pool)
    .await
    .expect("insert group member");
}

pub(crate) async fn insert_project(pool: &PgPool, id: impl AsRef<str>, name: &str) {
    sqlx::query(
        "INSERT INTO projects (id, name, source) VALUES ($1, $2, 'dashboard')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(id.as_ref())
    .bind(name)
    .execute(pool)
    .await
    .expect("insert project");
}

pub(crate) async fn insert_project_member(
    pool: &PgPool,
    project_id: impl AsRef<str>,
    user_id: &UserId,
    source: &str,
) {
    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, source) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(project_id.as_ref())
    .bind(user_id.as_str())
    .bind(source)
    .execute(pool)
    .await
    .expect("insert project member");
}
