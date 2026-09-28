//! Raw-SQL row builders.
//!
//! Every test owns its own database, but that database is not empty: the web
//! extension's schema seeds rows of its own. Fixtures therefore mint their own
//! ids, and assertions either name those ids or use a historical time window
//! no seeded row can reach.

use chrono::{DateTime, TimeZone, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::scope::SubjectScope;

// A window far enough in the past that no seeded row falls in it.
#[must_use]
pub(crate) fn ancient_window() -> (DateTime<Utc>, DateTime<Utc>) {
    (at(2001, 3, 1, 0), at(2001, 4, 1, 0))
}

#[must_use]
pub(crate) fn at(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, 0, 0)
        .single()
        .expect("fixed timestamp is unambiguous")
}

#[must_use]
pub(crate) fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4().simple())
}

#[must_use]
pub(crate) fn user_id(raw: &str) -> UserId {
    UserId::new(raw.to_owned())
}

pub(crate) async fn insert_user(pool: &PgPool, id: &str) {
    insert_user_with_roles(pool, id, &["user".to_owned()]).await;
}

pub(crate) async fn insert_user_with_roles(pool: &PgPool, id: &str, roles: &[String]) {
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
         VALUES ($1, $2, $2, $3, 'active', TRUE, $4)",
    )
    .bind(id)
    .bind(format!("{id}@example.test"))
    .bind(format!("User {id}"))
    .bind(roles.to_vec())
    .execute(pool)
    .await
    .expect("insert user");
}

// Put `user` in `project` (creating the project), or, with `None`, take them
// out of every project. The customer report scopes by resolved user id now,
// so membership is what the fixture has to establish.
pub(crate) async fn set_project(pool: &PgPool, user: &str, project: Option<&str>) {
    sqlx::query("DELETE FROM project_members WHERE user_id = $1")
        .bind(user)
        .execute(pool)
        .await
        .expect("clear project members");
    let Some(project) = project else {
        return;
    };
    sqlx::query(
        "INSERT INTO projects (id, name, source) VALUES ($1, $1, 'dashboard')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(project)
    .execute(pool)
    .await
    .expect("insert project");
    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, source) VALUES ($1, $2, 'manual')
         ON CONFLICT DO NOTHING",
    )
    .bind(project)
    .bind(user)
    .execute(pool)
    .await
    .expect("insert project member");
}

// The users in one project, as a bound scope.
pub(crate) async fn project_scope(pool: &PgPool, project: &str) -> SubjectScope {
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT user_id FROM project_members WHERE project_id = $1")
            .bind(project)
            .fetch_all(pool)
            .await
            .expect("project scope");
    SubjectScope::Users(ids)
}

// One `ai_requests` row. Defaults describe a completed, costed request; each
// test overrides only the columns its assertion reads.
pub(crate) struct RequestSeed<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub provider: Option<&'a str>,
    pub model: Option<&'a str>,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub cache_read_tokens: i32,
    // Reasoning is billed inside `output_tokens`; `tokens_used` is the
    // provider's billable total and is settable so a seed can give it a value
    // no re-summation of the components would produce.
    pub reasoning_tokens: i32,
    pub tokens_used: Option<i32>,
    pub cost_microdollars: i64,
    pub status: &'a str,
    pub created_at: DateTime<Utc>,
}

impl<'a> RequestSeed<'a> {
    #[must_use]
    pub(crate) fn new(id: &'a str, user_id: &'a str, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            user_id,
            provider: Some("anthropic"),
            model: Some("claude-sonnet-4-5-20250929"),
            input_tokens: 100,
            output_tokens: 20,
            cache_read_tokens: 0,
            reasoning_tokens: 0,
            tokens_used: None,
            cost_microdollars: 1_000,
            status: "completed",
            created_at,
        }
    }
}

pub(crate) async fn insert_request(pool: &PgPool, seed: &RequestSeed<'_>) {
    sqlx::query(
        "INSERT INTO ai_requests
            (id, request_id, user_id, context_id, provider, model, input_tokens, output_tokens,
             tokens_used, cache_read_tokens, reasoning_tokens, cost_microdollars, status,
             actor_kind, actor_id, created_at, updated_at)
         VALUES ($1, $1, $2, md5($2)::uuid, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'user', $2,
                 $12, $12)",
    )
    .bind(seed.id)
    .bind(seed.user_id)
    .bind(seed.provider)
    .bind(seed.model)
    .bind(seed.input_tokens)
    .bind(seed.output_tokens)
    .bind(
        seed.tokens_used
            .unwrap_or(seed.input_tokens + seed.output_tokens + seed.cache_read_tokens),
    )
    .bind(seed.cache_read_tokens)
    .bind(seed.reasoning_tokens)
    .bind(seed.cost_microdollars)
    .bind(seed.status)
    .bind(seed.created_at)
    .execute(pool)
    .await
    .expect("insert ai request");
}

// Registers the catalog row `access_control_rules` foreign-keys against.
pub(crate) async fn insert_acl_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
    default: bool,
) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ($1, $2, $3, 'test')
         ON CONFLICT (entity_type, entity_id) DO UPDATE SET default_included = EXCLUDED.default_included",
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(default)
    .execute(pool)
    .await
    .expect("insert access-control entity");
}

#[expect(
    clippy::too_many_arguments,
    reason = "one parameter per column of the row being seeded; a params struct would churn 18 call sites to say the same thing"
)]
pub(crate) async fn insert_env_var(
    pool: &PgPool,
    user: &str,
    plugin: &str,
    name: &str,
    value: &str,
    secret: bool,
) -> String {
    let id = unique("env");
    sqlx::query(
        "INSERT INTO plugin_env_vars (id, user_id, plugin_id, var_name, var_value, is_secret)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&id)
    .bind(user)
    .bind(plugin)
    .bind(name)
    .bind(value)
    .bind(secret)
    .execute(pool)
    .await
    .expect("insert plugin env var");
    id
}

pub(crate) async fn count_rows(pool: &PgPool, sql: &'static str, arg: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .bind(arg)
        .fetch_one(pool)
        .await
        .expect("count rows")
}

// Writes `services/<rel>` under `dir`, creating parents.
pub(crate) fn write_services_file(dir: &std::path::Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create services subdirectory");
    }
    std::fs::write(path, contents).expect("write services file");
}
