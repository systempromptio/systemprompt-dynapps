//! Query-parameter coverage for the server-rendered admin pages.
//!
//! [`crate::status_contract`] drives every route once, with no query string,
//! and asserts the properties that hold for all of them. That leaves the
//! branching *inside* a page untested: a list page is a tab selector, a filter
//! set, a sort order, and a pager, and each of those is a separate query the
//! handler builds and a separate block the template renders. A page can answer
//! 200 on its default view for years while its `?tab=providers` arm has been
//! broken since a refactor.
//!
//! Each case therefore asserts two things: the status, and a marker string that
//! only the intended branch emits. The marker is what makes the case a test
//! rather than a smoke check — `200` proves the handler did not panic, the
//! marker proves it rendered the thing the query asked for.
//!
//! Markers are drawn from the templates in `storage/files/admin/templates/`
//! and their partials, so a template edit that drops a branch fails here
//! rather than shipping a page that silently renders nothing.
//!
//! The database is the throwaway one, seeded only with the two principals, so
//! every list is empty. That is deliberate: the empty state is a rendered
//! branch like any other, and asserting its message proves the query ran and
//! returned nothing rather than erroring into an `unwrap_or_default()`.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

// A path (query string included) and the substring its response must contain.
struct Variant {
    path: &'static str,
    marker: &'static str,
}

const fn v(path: &'static str, marker: &'static str) -> Variant {
    Variant { path, marker }
}

// Markers used by more than one case, named so a template edit has one place
// to be reflected.
const REQUESTS_TITLE: &str = "Inference Requests";
const REQUESTS_OVERVIEW: &str = "Latency distribution";
const REQUESTS_EMPTY: &str = "No inference requests match.";
// The seeded fixture puts requests inside the default window, so an unfiltered
// log page renders rows. Match the row's data attribute rather than a class:
// the class is design-system property and moves with the stylesheet, while
// `data-request-row` is what the page's own markup and its JS agree on.
const REQUESTS_LOG_ROWS: &str = "data-request-row";
const TRACES_TITLE: &str = "Trace Explorer";
const TRACES_EMPTY: &str = "No traces in this window";
const CONTEXTS_TITLE: &str = "Conversation KPIs";
const CONTEXTS_EMPTY: &str = "No conversation matches the selected scope and filters.";
const CONTEXTS_USERS_EMPTY: &str =
    "Nobody under the selected group, project and time range has a conversation.";

// The requests page: five tabs, a time window, four filter chips, a sort, and
// a pager — the densest branching in the admin plane.
const REQUESTS: [Variant; 28] = [
    // The log is the default view: a reader opening the page wants rows.
    v("/admin/requests", REQUESTS_LOG_ROWS),
    v("/admin/requests?tab=log", REQUESTS_LOG_ROWS),
    v("/admin/requests?tab=overview", REQUESTS_OVERVIEW),
    v("/admin/requests?tab=overview", "pre-flight denies"),
    v(
        "/admin/requests?tab=models",
        "attributed to the model that produced them.",
    ),
    v(
        "/admin/requests?tab=providers",
        "rolled up to the upstream provider.",
    ),
    v("/admin/requests?tab=status", "Outcome mix for the window."),
    // An unrecognised tab falls back to the log rather than rendering a page
    // with every section switched off.
    v("/admin/requests?tab=nonsense", REQUESTS_LOG_ROWS),
    v("/admin/requests?tab=log&page=2", REQUESTS_EMPTY),
    v("/admin/requests?tab=log&page=100000", REQUESTS_EMPTY),
    // A negative page is clamped to the first, not turned into a negative
    // OFFSET the database would reject.
    v("/admin/requests?tab=log&page=-5", REQUESTS_LOG_ROWS),
    v(
        "/admin/requests?tab=log&model=claude-opus-5",
        r#"filter-ribbon__chip-value">claude-opus-5</span>"#,
    ),
    v(
        "/admin/requests?tab=log&provider=anthropic",
        r#"filter-ribbon__chip-value">anthropic</span>"#,
    ),
    v(
        "/admin/requests?tab=log&status=error",
        r#"filter-ribbon__chip-value">error</span>"#,
    ),
    v(
        "/admin/requests?tab=log&q=needle-xyz",
        r#"filter-ribbon__chip-value">needle-xyz</span>"#,
    ),
    // The page no longer draws its own preset buttons — the shell's scope bar
    // owns the window for every AI-activity page, and two of them on one page
    // was the duplication that pushed the table below the fold. The header
    // states the window it resolved instead, which is what the assertion needs.
    v("/admin/requests?preset=15m", r#"data-window="15m""#),
    v("/admin/requests?preset=7d", r#"data-window="7d""#),
    v("/admin/requests?preset=30d", r#"data-window="30d""#),
    v("/admin/requests?preset=nonsense", REQUESTS_TITLE),
    v(
        "/admin/requests?from=2026-01-01T00:00:00Z&to=2026-02-01T00:00:00Z",
        REQUESTS_TITLE,
    ),
    v(
        "/admin/requests?tab=log&sort=cost&dir=asc",
        REQUESTS_LOG_ROWS,
    ),
    v(
        "/admin/requests?tab=log&sort=nonsense&dir=nonsense",
        REQUESTS_LOG_ROWS,
    ),
    // The facet the log added: a tool the request issued. There is deliberately
    // no skill facet — a skill invocation has no join to a gateway request on
    // this instance, so the filter could only ever return a plausible wrong
    // number or a misleading empty table.
    v("/admin/requests?tab=log&tool=no-such-tool", REQUESTS_EMPTY),
    // `unrouted` is the sentinel the analytics drill-downs send for the rows
    // the gateway rejected before resolving a route. It has to mean "no model",
    // not "a model called unrouted", or that drill lands on an empty table.
    v("/admin/requests?model=unrouted", REQUESTS_EMPTY),
    v("/admin/requests?provider=unrouted", REQUESTS_EMPTY),
    // The short parameter spellings the analytics drill-downs emit fold into
    // the canonical ones rather than being ignored.
    v("/admin/requests?range=7d", r#"data-window="7d""#),
    v(
        "/admin/requests?scope=project:no-such-project",
        REQUESTS_EMPTY,
    ),
    v("/admin/requests?scope=nonsense", REQUESTS_LOG_ROWS),
];

// The trace explorer: the same window and pager, plus the two stat tiles that
// double as filters.
const TRACES: [Variant; 11] = [
    v("/admin/traces", TRACES_EMPTY),
    // Why: the explorer has no preset control of its own; the window rides the
    // filter form as a preserved parameter, which is where the URL's value
    // has to reappear.
    v("/admin/traces?preset=7d", r#"name="preset" value="7d""#),
    v("/admin/traces?page=3", TRACES_EMPTY),
    v("/admin/traces?page=99999", TRACES_EMPTY),
    v("/admin/traces?page=-2", TRACES_EMPTY),
    // `deny_only` / `error_only` mark their tile as the applied filter; the
    // tile is the only thing on the page that says so.
    v("/admin/traces?deny_only=true", "is-active"),
    v("/admin/traces?error_only=true", "is-active"),
    v("/admin/traces?sort=cost&dir=asc", TRACES_EMPTY),
    v("/admin/traces?policy=blocklist&decision=deny", TRACES_EMPTY),
    v(
        "/admin/traces?agent_scope=global&agent_id=no-such-agent",
        TRACES_EMPTY,
    ),
    // An unparseable custom window falls back to the default one and still
    // renders the page rather than refusing it.
    v("/admin/traces?from=not-a-date&to=not-a-date", TRACES_TITLE),
];

// The contexts page: a two-way view switch, a search box, a since-window pill
// group, and a row limit.
const CONTEXTS: [Variant; 9] = [
    v("/admin/contexts", CONTEXTS_USERS_EMPTY),
    v("/admin/contexts?view=all", CONTEXTS_EMPTY),
    v("/admin/contexts?view=users", CONTEXTS_USERS_EMPTY),
    v("/admin/contexts?q=needle-xyz", r#"value="needle-xyz""#),
    // Why: the window has no control of its own on this page; it is carried
    // as a preserved parameter of the scope form, which is where a value the
    // URL named has to reappear.
    v("/admin/contexts?since=7d", r#"name="since" value="7d""#),
    v("/admin/contexts?since=30d", r#"name="since" value="30d""#),
    v("/admin/contexts?since=nonsense", CONTEXTS_TITLE),
    v("/admin/contexts?limit=1", CONTEXTS_USERS_EMPTY),
    v("/admin/contexts?limit=100000", CONTEXTS_USERS_EMPTY),
];

// The roster, the per-user page, the customer report, and the catalog.
//
// The roster is the one list with rows: the suite seeds two principals, so it
// renders the table rather than the empty state, and the seeded address is the
// marker that proves a row reached the template.
// The landing page: three windows, seven tiles, a chart and two boards. The
// markers are the tiles and boards the redesign added, plus the shared
// time-range pill that marks the window as active.
const OVERVIEW: [Variant; 5] = [
    v("/admin", "p50 latency"),
    v("/admin", "Most used models"),
    v("/admin", "Request volume"),
    // Why: Handlebars escapes `=` inside attribute values, so the pill's href is
    // not a literal to match on; the active state is asserted via aria-current.
    v("/admin?preset=7d", r#"aria-current="page">7d</a>"#),
    v("/admin?preset=30d", "vs previous 30d"),
];

const PAGES: [Variant; 6] = [
    v("/admin/users", "contract-admin@contract.test"),
    // Why: a page past the end is an empty page, not a redirect to the last
    // one; the unknown parameter is ignored rather than rejected.
    v("/admin/users?unknown_param=1&page=99", "No users match"),
    // Why: the roster's filter chips are `WHERE` arms, not saved searches, so
    // each one is a branch of the same statement. An id that matches nothing is
    // a miss and answers 404 — pinned in `crate::handler_errors`, not here.
    v(
        "/admin/users?filter=unassigned&sort=name&dir=asc",
        "Unassigned",
    ),
    // Why: the two report *pages* are gone — the analytics dashboard's cost
    // tab replaced them — so their `?month=` branches have no page left to
    // select. The month parameter survives on the CSV exports, whose statuses
    // are pinned in the baseline.
    v(
        "/admin/plugins",
        "A plugin bundles skills, MCP servers, agents and hooks",
    ),
    v("/admin/skills", "The instruction sets people invoke"),
    v("/admin/mcp", "what it is serving right now"),
];

#[tokio::test(flavor = "multi_thread")]
async fn admin_pages_render_the_branch_their_query_selects() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping admin handler-variant suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;

    // The requests-log rows the `tab=log` variants assert on. One row keeps
    // page 1 populated and page 2 empty, which is exactly what the variant
    // table expects.
    let request_user = seed::insert_user(
        &db.pool,
        &seed::unique("variant-user"),
        "variant-user@contract.test",
    )
    .await;
    seed::insert_request(
        &db.pool,
        &seed::RequestSpec {
            id: seed::unique("variant-request"),
            user_id: &request_user,
            session_id: None,
            trace_id: None,
            context_id: None,
            status: "completed",
        },
    )
    .await;

    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();
    for variant in OVERVIEW
        .iter()
        .chain(REQUESTS.iter())
        .chain(TRACES.iter())
        .chain(CONTEXTS.iter())
        .chain(PAGES.iter())
    {
        let (status, body) = app.call(Call::get(variant.path, Principal::Admin)).await;

        if status != StatusCode::OK {
            failures.push(format!(
                "  {} -> {} (expected 200){}",
                variant.path,
                status.as_u16(),
                snippet(&body)
            ));
            continue;
        }
        if !body.contains(variant.marker) {
            failures.push(format!(
                "  {} -> 200 but the body never contained {:?} — the query selected a \
                 different branch than the one it names{}",
                variant.path,
                variant.marker,
                snippet(&body)
            ));
        }
    }

    db.cleanup().await;

    assert!(
        failures.is_empty(),
        "{} page variant(s) rendered the wrong thing:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// Enough of the body to tell which page came back, without pasting a whole
// rendered document into the failure.
fn snippet(body: &str) -> String {
    let head: String = body.chars().take(300).collect();
    format!("\n      body: {head}")
}
