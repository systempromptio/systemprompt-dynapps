//! A fake farm-api serving responses recorded from farm-dev (sanitised: no
//! presigned URLs, no prompts, no real names), a fake systemprompt usage
//! source, and the admin router wired the way the console wires it, rendering
//! the real templates under `storage/files/admin`.

use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Request, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Router};
use chrono::{DateTime, Utc};
use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin_agent_farm::client::FarmApi;
use systemprompt_web_admin_agent_farm::routes::routes_with;
use systemprompt_web_admin_agent_farm::sp::{SharedSp, SpCall, SpError, SpModelTotal, SpUsage};
use systemprompt_web_admin_agent_farm::types::SessionTotalRow;
use systemprompt_web_admin_core::repositories::scope::Participation;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};
use systemprompt_web_shared::RankTier;
use tower::ServiceExt;

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) const TOKEN: &str = "test-token-not-a-secret";
pub(crate) const TASK_OK: &str = "01a17751-467a-7792-b87c-8b67e9d8fa36";
pub(crate) const TASK_FAILED: &str = "01a167f2-0f8f-7e40-b9cb-9f1db758922a";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    // Answers like farm-dev does.
    Normal,
    // Every call is a 500 with a non-JSON body.
    Broken,
    // whoami works; every task read is a 403 (credential without the scope).
    NoPlatformScope,
    // A dated task list never ends: every page has a `next_cursor`.
    Endless,
    // farm-api before 16a: it knows no `tenant=*` and answers 422 to it.
    NoWildcard,
}

#[derive(Clone)]
struct FakeState {
    mode: Mode,
    seen: Arc<Mutex<Vec<String>>>,
}

pub(crate) struct FakeFarm {
    pub(crate) base: String,
    seen: Arc<Mutex<Vec<String>>>,
}

impl FakeFarm {
    // Every request the fake received, as `path?query`.
    pub(crate) fn seen(&self) -> Vec<String> {
        self.seen.lock().map(|v| v.clone()).unwrap_or_default()
    }
}

pub(crate) fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/farm")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| format!("missing fixture {name}: {e}"))
}

fn json(status: StatusCode, name: &str) -> Response {
    (
        status,
        [("content-type", "application/json")],
        fixture(name),
    )
        .into_response()
}

fn record(state: &FakeState, uri: &Uri, headers: &HeaderMap) -> Option<Response> {
    if let Ok(mut seen) = state.seen.lock() {
        seen.push(
            uri.path_and_query()
                .map_or_else(|| uri.path().to_owned(), ToString::to_string),
        );
    }
    let expected = format!("Bearer {TOKEN}");
    let authorised = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == expected);
    if !authorised {
        return Some(json(StatusCode::UNAUTHORIZED, "noauth.json"));
    }
    if state.mode == Mode::Broken {
        return Some((StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response());
    }
    None
}

async fn whoami(State(s): State<FakeState>, uri: Uri, headers: HeaderMap) -> Response {
    record(&s, &uri, &headers).unwrap_or_else(|| json(StatusCode::OK, "whoami_platform.json"))
}

fn task_gate(s: &FakeState, uri: &Uri, headers: &HeaderMap) -> Option<Response> {
    record(s, uri, headers).or_else(|| {
        if s.mode == Mode::NoWildcard && tenant_of(uri).as_deref() == Some("*") {
            return Some(json(
                StatusCode::UNPROCESSABLE_ENTITY,
                "wildcard_unsupported.json",
            ));
        }
        (s.mode == Mode::NoPlatformScope)
            .then(|| json(StatusCode::FORBIDDEN, "insufficient_scope.json"))
    })
}

fn query_has(uri: &Uri, needle: &str) -> bool {
    uri.query()
        .is_some_and(|q| q.split('&').any(|p| p == needle))
}

// The `tenant` the request names; `None` when it names none.
fn tenant_of(uri: &Uri) -> Option<String> {
    uri.query()?
        .split('&')
        .find_map(|p| p.strip_prefix("tenant="))
        .map(ToOwned::to_owned)
}

// Phase 16a's contract for list reads: `tenant=*` is every tenant the
// credential can see (all of them for the console's platform scope), a slug
// is that tenant. The items of the `parts` fixtures are filtered that way;
// `next_cursor` is the first part's, kept while it still has items.
fn list_for_tenant(uri: &Uri, parts: &[&str]) -> Response {
    let wanted = tenant_of(uri);
    let mut items = Vec::new();
    let mut next = serde_json::Value::Null;
    for (index, name) in parts.iter().enumerate() {
        let body: serde_json::Value = serde_json::from_str(&fixture(name)).unwrap_or_default();
        let matching: Vec<serde_json::Value> = body["items"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| {
                wanted
                    .as_deref()
                    .is_none_or(|w| w == "*" || item["tenant"].as_str() == Some(w))
            })
            .collect();
        if index == 0 && !matching.is_empty() {
            next = body["next_cursor"].clone();
        }
        items.extend(matching);
    }
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        serde_json::json!({ "items": items, "next_cursor": next }).to_string(),
    )
        .into_response()
}

// A detail read must name one tenant: farm-api does not take `*` there.
fn detail_gate(s: &FakeState, uri: &Uri, headers: &HeaderMap) -> Option<Response> {
    task_gate(s, uri, headers).or_else(|| {
        (tenant_of(uri).as_deref() == Some("*"))
            .then(|| json(StatusCode::BAD_REQUEST, "wildcard_detail.json"))
    })
}

async fn tasks(State(s): State<FakeState>, uri: Uri, headers: HeaderMap) -> Response {
    task_gate(&s, &uri, &headers).unwrap_or_else(|| {
        let dated = uri.query().is_some_and(|q| q.contains("since="));
        if !dated {
            return list_for_tenant(&uri, &["tasks_page.json", "tasks_lab_b.json"]);
        }
        // The cost page asks for a date range: serve the cost fixtures.
        if s.mode == Mode::Endless {
            return json(StatusCode::OK, "cost_endless.json");
        }
        if query_has(&uri, "tenant=lab-b") {
            return json(StatusCode::OK, "cost_lab_b.json");
        }
        let everyone = query_has(&uri, "tenant=*");
        if query_has(&uri, "cursor=c3") {
            return json(StatusCode::OK, "cost_lab_b.json");
        }
        if query_has(&uri, "cursor=c2") {
            if everyone {
                // One stream across tenants: the second tenant follows the
                // first behind the same kind of cursor.
                return chained(&fixture("cost_page2.json"), "c3");
            }
            json(StatusCode::OK, "cost_page2.json")
        } else {
            json(StatusCode::OK, "cost_page1.json")
        }
    })
}

// `body` with its `next_cursor` replaced.
fn chained(body: &str, cursor: &str) -> Response {
    let mut value: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    value["next_cursor"] = serde_json::Value::String(cursor.to_owned());
    (
        StatusCode::OK,
        [("content-type", "application/json")],
        value.to_string(),
    )
        .into_response()
}

async fn agents(State(s): State<FakeState>, uri: Uri, headers: HeaderMap) -> Response {
    task_gate(&s, &uri, &headers)
        .unwrap_or_else(|| list_for_tenant(&uri, &["agents.json", "agents_lab_b.json"]))
}

async fn agent(
    State(s): State<FakeState>,
    Path(slug): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    detail_gate(&s, &uri, &headers).unwrap_or_else(|| match slug.as_str() {
        "lab-helper" if tenant_of(&uri).as_deref() == Some("lab-b") => {
            json(StatusCode::OK, "agent_lab_helper.json")
        },
        "code-reviewer" => json(StatusCode::OK, "agent_code_reviewer.json"),
        "release-notes" => json(StatusCode::OK, "agent_release_notes.json"),
        "my-helper" => json(StatusCode::OK, "agent_my_helper.json"),
        _ => json(StatusCode::NOT_FOUND, "notfound.json"),
    })
}

async fn schedules(State(s): State<FakeState>, uri: Uri, headers: HeaderMap) -> Response {
    task_gate(&s, &uri, &headers)
        .unwrap_or_else(|| list_for_tenant(&uri, &["schedules.json", "schedules_lab_b.json"]))
}

async fn credentials(State(s): State<FakeState>, uri: Uri, headers: HeaderMap) -> Response {
    task_gate(&s, &uri, &headers).unwrap_or_else(|| {
        if query_has(&uri, "scope=shared") {
            list_for_tenant(&uri, &["credentials_shared.json"])
        } else {
            list_for_tenant(&uri, &["credentials.json", "credentials_lab_b.json"])
        }
    })
}

async fn task(
    State(s): State<FakeState>,
    Path(id): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    detail_gate(&s, &uri, &headers).unwrap_or_else(|| match id.as_str() {
        TASK_OK => json(StatusCode::OK, "task.json"),
        TASK_FAILED => json(StatusCode::OK, "task_failed.json"),
        _ => json(StatusCode::NOT_FOUND, "notfound.json"),
    })
}

async fn result(
    State(s): State<FakeState>,
    Path(id): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    detail_gate(&s, &uri, &headers).unwrap_or_else(|| match id.as_str() {
        TASK_OK => json(StatusCode::OK, "result.json"),
        TASK_FAILED => json(StatusCode::CONFLICT, "result_none.json"),
        _ => json(StatusCode::NOT_FOUND, "notfound.json"),
    })
}

async fn logs(
    State(s): State<FakeState>,
    Path(id): Path<String>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    detail_gate(&s, &uri, &headers).unwrap_or_else(|| match id.as_str() {
        TASK_OK => json(StatusCode::OK, "logs.json"),
        TASK_FAILED => json(StatusCode::OK, "logs_failed.json"),
        _ => json(StatusCode::NOT_FOUND, "notfound.json"),
    })
}

pub(crate) async fn fake_farm(mode: Mode) -> Result<FakeFarm, Box<dyn Error>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let state = FakeState {
        mode,
        seen: Arc::clone(&seen),
    };
    let app = Router::new()
        .route("/farm/api/v1/whoami", get(whoami))
        .route("/farm/api/v1/tasks", get(tasks))
        .route("/farm/api/v1/agents", get(agents))
        .route("/farm/api/v1/agents/{slug}", get(agent))
        .route("/farm/api/v1/schedules", get(schedules))
        .route("/farm/api/v1/credentials", get(credentials))
        .route("/farm/api/v1/tasks/{id}", get(task))
        .route("/farm/api/v1/tasks/{id}/result", get(result))
        .route("/farm/api/v1/tasks/{id}/logs", get(logs))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move { axum::serve(listener, app).await });
    Ok(FakeFarm {
        base: format!("http://{addr}/farm/api/v1"),
        seen,
    })
}

// A base URL nothing listens on.
pub(crate) async fn dead_url() -> Result<String, Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    drop(listener);
    Ok(format!("http://{addr}/farm/api/v1"))
}

fn user(is_console: bool) -> Result<UserContext, Box<dyn Error>> {
    Ok(UserContext {
        user_id: UserId::generate(),
        username: "admin".to_owned(),
        email: Email::try_new("admin@example.com")?,
        roles: if is_console {
            vec!["admin".to_owned()]
        } else {
            vec!["user".to_owned()]
        },
        group_ids: Vec::new(),
        project_ids: Vec::new(),
        participation: Participation::default(),
        is_admin: is_console,
        is_console,
        is_platform_admin: is_console,
        is_developer: false,
        email_verified: true,
        session_id: None,
    })
}

fn marketplace() -> MarketplaceContext {
    MarketplaceContext {
        user_id: UserId::generate(),
        site_url: "https://governance.invalid".to_owned(),
        total_plugins: 0,
        total_skills: 0,
        agents_count: 0,
        mcp_count: 0,
        rank_level: 0,
        rank_name: String::new(),
        rank_tier: RankTier::default(),
        total_xp: 0,
        xp_progress_pct: 0.0,
        has_completed_onboarding: true,
        current_streak: 0,
        longest_streak: 0,
        next_rank_name: String::new(),
        xp_to_next_rank: 0,
    }
}

fn engine() -> Result<AdminTemplateEngine, Box<dyn Error>> {
    let admin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../storage/files/admin");
    Ok(AdminTemplateEngine::new(&admin_dir)?)
}

pub(crate) struct Page {
    pub(crate) status: StatusCode,
    pub(crate) html: String,
}

impl Page {
    // Whether the page contains `needle`, after undoing the HTML escaping
    // Handlebars applies (`=` → `&#x3D;`, `&` → `&amp;`, …).
    pub(crate) fn has(&self, needle: &str) -> bool {
        let text = self
            .html
            .replace("&#x3D;", "=")
            .replace("&#x27;", "'")
            .replace("&#x60;", "`")
            .replace("&quot;", "\"")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        text.contains(needle)
    }

    // The shell's "could not be loaded" notice (components/unavailable).
    pub(crate) fn has_fault_notice(&self) -> bool {
        self.has("Some of this page could not be loaded")
    }
}

// GET `uri` (relative to `/admin`) through the farm routes as a
// signed-in viewer, with systemprompt answering like [`SpMode::Normal`].
pub(crate) async fn get_page(
    api: FarmApi,
    uri: &str,
    is_console: bool,
) -> Result<Page, Box<dyn Error>> {
    get_page_sp(api, Arc::new(FakeSp::new(SpMode::Normal)), uri, is_console).await
}

// [`get_page`] with a given systemprompt usage source.
pub(crate) async fn get_page_sp(
    api: FarmApi,
    sp: Arc<FakeSp>,
    uri: &str,
    is_console: bool,
) -> Result<Page, Box<dyn Error>> {
    let sp: SharedSp = sp;
    let app: Router = routes_with(api, sp)
        .layer(Extension(engine()?))
        .layer(Extension(user(is_console)?))
        .layer(Extension(marketplace()));
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024).await?;
    Ok(Page {
        status,
        html: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

// ------------------------------------------------------------ systemprompt

// The gateway session of [`TASK_OK`] (`task.json`, recorded from farm-dev).
pub(crate) const GW_TASK_OK: &str = "sess_3eb3e826-fad4-460b-b35f-e7ce2b3ad9eb";
// The gateway session of the first task in `tasks_page.json`.
pub(crate) const GW_TASK_2: &str = "sess_9e954023-f3a5-491f-b050-203823c9390c";
// [`TASK_OK`]'s Claude Code session (`session_ref` without `sess_`).
pub(crate) const CLIENT_TASK_OK: &str = "547de7ed-4450-4a6a-8dff-d8cf287f9746";
// A real context id from the recorded proof (its pairing with
// [`CLIENT_TASK_OK`] is assumed here, not proven).
pub(crate) const CONTEXT_TASK_OK: &str = "35763d68-cffb-4d39-ab4c-5c75c88a49eb";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpMode {
    // Rows for the recorded tasks and the cost fixtures.
    Normal,
    // No rows at all (e.g. past systemprompt's 90-day retention).
    Empty,
    // Every read fails.
    Broken,
}

// A systemprompt usage source with fixed rows. Calls follow the one real
// call of each recorded task (output tokens and cost as measured
// on farm-dev); the second call of [`TASK_OK`] and the cost fixtures' rows
// are synthesised.
pub(crate) struct FakeSp {
    mode: SpMode,
    seen: Arc<Mutex<Vec<String>>>,
}

impl FakeSp {
    pub(crate) fn new(mode: SpMode) -> Self {
        Self {
            mode,
            seen: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Every read, as `method arg`.
    pub(crate) fn seen(&self) -> Vec<String> {
        self.seen.lock().map(|v| v.clone()).unwrap_or_default()
    }

    fn note(&self, entry: String) -> Result<(), SpError> {
        if let Ok(mut seen) = self.seen.lock() {
            seen.push(entry);
        }
        if self.mode == SpMode::Broken {
            return Err(SpError("connection refused".to_owned()));
        }
        Ok(())
    }

    fn rows(&self) -> Vec<(&'static str, SpCall)> {
        if self.mode == SpMode::Empty || self.mode == SpMode::Broken {
            return Vec::new();
        }
        let mut rows = vec![
            (
                GW_TASK_OK,
                call(
                    "r1",
                    "2026-10-26T08:26:26Z",
                    "claude-haiku-5-5",
                    "completed",
                    [2, 4315, 12_000, 3_000, 2_374, 8_200],
                ),
            ),
            (
                GW_TASK_OK,
                call(
                    "r2",
                    "2026-10-26T08:26:40Z",
                    "claude-sonnet-5",
                    "failed",
                    [7, 0, 0, 0, 0, 450],
                ),
            ),
            (
                GW_TASK_2,
                call(
                    "r3",
                    "2026-10-27T07:10:27Z",
                    "claude-haiku-5-5",
                    "completed",
                    [2, 822, 0, 0, 4_557, 3_100],
                ),
            ),
        ];
        // The cost fixtures: one call per linked task.
        for (session, micro) in [
            ("sess_00000000-0000-4000-8000-000000000001", 500_000),
            ("sess_00000000-0000-4000-8000-000000000002", 250_000),
            ("sess_00000000-0000-4000-8000-000000000004", 1_250_000),
            ("sess_00000000-0000-4000-8000-000000000005", 2_500),
            ("sess_00000000-0000-4000-8000-000000000006", 4_000_000),
        ] {
            rows.push((
                session,
                call(
                    "c",
                    "2026-10-26T08:00:05Z",
                    "claude-haiku-5-5",
                    "completed",
                    [10, 20, 100, 0, micro, 1_000],
                ),
            ));
        }
        rows
    }
}

// `numbers`: input, output, cache-read and cache-creation tokens, cost in
// microdollars, latency in ms.
fn call(id: &str, at: &str, model: &str, status: &str, numbers: [i64; 6]) -> SpCall {
    let [
        input,
        output,
        cache_read,
        cache_creation,
        cost_microdollars,
        latency,
    ] = numbers;
    SpCall {
        id: id.to_owned(),
        created_at: DateTime::parse_from_rfc3339(at)
            .map(|t| t.with_timezone(&Utc))
            .unwrap_or_default(),
        model: Some(model.to_owned()),
        status: status.to_owned(),
        request_kind: "turn".to_owned(),
        input_tokens: Some(input as i32),
        output_tokens: Some(output as i32),
        cache_read_tokens: Some(cache_read as i32),
        cache_creation_tokens: Some(cache_creation as i32),
        cost_microdollars,
        latency_ms: Some(latency as i32),
    }
}

fn sum(calls: &[&SpCall], f: impl Fn(&SpCall) -> Option<i32>) -> i64 {
    calls.iter().map(|c| i64::from(f(c).unwrap_or(0))).sum()
}

#[async_trait::async_trait]
impl SpUsage for FakeSp {
    async fn calls(&self, gateway_session_id: &str) -> Result<Vec<SpCall>, SpError> {
        self.note(format!("calls {gateway_session_id}"))?;
        Ok(self
            .rows()
            .into_iter()
            .filter(|(s, _)| *s == gateway_session_id)
            .map(|(_, c)| c)
            .collect())
    }

    async fn totals_by_model(
        &self,
        gateway_session_id: &str,
    ) -> Result<Vec<SpModelTotal>, SpError> {
        self.note(format!("totals_by_model {gateway_session_id}"))?;
        let rows = self.rows();
        let calls: Vec<&SpCall> = rows
            .iter()
            .filter(|(s, _)| *s == gateway_session_id)
            .map(|(_, c)| c)
            .collect();
        let mut models: Vec<Option<String>> = calls.iter().map(|c| c.model.clone()).collect();
        models.dedup();
        let mut out: Vec<SpModelTotal> = models
            .into_iter()
            .map(|m| {
                let of: Vec<&SpCall> = calls.iter().copied().filter(|c| c.model == m).collect();
                SpModelTotal {
                    calls: of.len() as i64,
                    failed: of.iter().filter(|c| c.status == "failed").count() as i64,
                    input_tokens: sum(&of, |c| c.input_tokens),
                    output_tokens: sum(&of, |c| c.output_tokens),
                    cache_read_tokens: sum(&of, |c| c.cache_read_tokens),
                    cache_creation_tokens: sum(&of, |c| c.cache_creation_tokens),
                    cost_microdollars: of.iter().map(|c| c.cost_microdollars).sum(),
                    model: m,
                }
            })
            .collect();
        out.sort_by_key(|t| std::cmp::Reverse(t.cost_microdollars));
        Ok(out)
    }

    async fn contexts(&self, client_session_id: &str) -> Result<Vec<String>, SpError> {
        self.note(format!("contexts {client_session_id}"))?;
        if self.mode == SpMode::Normal && client_session_id == CLIENT_TASK_OK {
            return Ok(vec![CONTEXT_TASK_OK.to_owned()]);
        }
        Ok(Vec::new())
    }

    async fn session_totals(
        &self,
        gateway_session_ids: &[String],
    ) -> Result<Vec<SessionTotalRow>, SpError> {
        self.note(format!("session_totals {}", gateway_session_ids.join(",")))?;
        let rows = self.rows();
        Ok(gateway_session_ids
            .iter()
            .filter_map(|id| {
                let of: Vec<&SpCall> = rows
                    .iter()
                    .filter(|(s, _)| *s == id.as_str())
                    .map(|(_, c)| c)
                    .collect();
                (!of.is_empty()).then(|| SessionTotalRow {
                    gateway_session: id.clone(),
                    calls: of.len() as i64,
                    input_tokens: sum(&of, |c| c.input_tokens),
                    output_tokens: sum(&of, |c| c.output_tokens),
                    cache_read_tokens: sum(&of, |c| c.cache_read_tokens),
                    cache_creation_tokens: sum(&of, |c| c.cache_creation_tokens),
                    cost_microdollars: of.iter().map(|c| c.cost_microdollars).sum(),
                })
            })
            .collect())
    }
}
