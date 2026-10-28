//! Exhaustiveness: the routes the router actually mounts, recovered from the
//! route modules themselves.
//!
//! Axum exposes no way to enumerate a built `Router`, so the method/path pairs
//! are read back from the source that declares them — pulled in with
//! `include_str!`, so the suite and the router can never disagree about what
//! exists. Deriving the table rather than hand-writing it is what makes
//! coverage exhaustive by construction: a new `.route(...)` is picked up and
//! exercised on the next run, and shows up as a baseline addition.

use systemprompt_web_admin_platform::routes::drive_user::USER_BASE_URL;

use crate::app::{ADMIN_API_PREFIX, FARM_PUBLIC_PREFIX, SSR_PREFIX};

const ADMIN_API_SRC: &str = include_str!("../../../../extensions/web/src/router/admin/api.rs");
const ANALYTICS_API_SRC: &str =
    include_str!("../../../../extensions/web/admin/analytics/src/routes/api.rs");
const ANALYTICS_OVERVIEW_SRC: &str =
    include_str!("../../../../extensions/web/admin/analytics/src/routes/overview.rs");
const ANALYTICS_ACTIVITY_SRC: &str =
    include_str!("../../../../extensions/web/admin/analytics/src/routes/activity_ssr.rs");
const ANALYTICS_ANALYSIS_SRC: &str =
    include_str!("../../../../extensions/web/admin/analytics/src/routes/analysis_ssr.rs");
const ANALYTICS_OBSERVABILITY_SRC: &str =
    include_str!("../../../../extensions/web/admin/analytics/src/routes/observability_ssr.rs");
const IDENTITY_API_SRC: &str =
    include_str!("../../../../extensions/web/admin/identity/src/routes/api.rs");
const IDENTITY_GROUPS_SRC: &str =
    include_str!("../../../../extensions/web/admin/identity/src/routes/admin_groups.rs");
const IDENTITY_PEOPLE_SRC: &str =
    include_str!("../../../../extensions/web/admin/identity/src/routes/people_ssr.rs");
const IDENTITY_ACCOUNT_SRC: &str =
    include_str!("../../../../extensions/web/admin/identity/src/routes/account_ssr.rs");
const IDENTITY_PUBLIC_SRC: &str =
    include_str!("../../../../extensions/web/admin/identity/src/routes/public.rs");
const MANAGED_SRC: &str =
    include_str!("../../../../extensions/web/admin/catalog/src/routes/managed_resources.rs");
const CATALOG_API_SRC: &str =
    include_str!("../../../../extensions/web/admin/catalog/src/routes/api.rs");
const CATALOG_SSR_SRC: &str =
    include_str!("../../../../extensions/web/admin/catalog/src/routes/ssr.rs");
const GOVERNANCE_API_SRC: &str =
    include_str!("../../../../extensions/web/admin/governance/src/routes/api.rs");
const GOVERNANCE_SSR_SRC: &str =
    include_str!("../../../../extensions/web/admin/governance/src/routes/ssr_governance.rs");
const PLATFORM_API_SRC: &str =
    include_str!("../../../../extensions/web/admin/platform/src/routes/api.rs");
const PLATFORM_SSR_SRC: &str =
    include_str!("../../../../extensions/web/admin/platform/src/routes/platform_ssr.rs");
const SSR_SRC: &str = include_str!("../../../../extensions/web/src/router/admin/ssr.rs");
// The agent farm crate mounts four route tables: pages and their fragments in
// the SSR router, the console JSON API beside the other admin reads and
// writes, and the public run API at its own prefix.
const FARM_SSR_SRC: &str =
    include_str!("../../../../extensions/web/admin/farm/src/routes/farm_ssr.rs");
const FARM_FRAGMENTS_SRC: &str =
    include_str!("../../../../extensions/web/admin/farm/src/routes/farm_fragments.rs");
const FARM_PUBLIC_SRC: &str =
    include_str!("../../../../extensions/web/admin/farm/src/routes/public.rs");
const FARM_API_SRC: &str = include_str!("../../../../extensions/web/admin/farm/src/routes/api.rs");
// Merged into the SSR router from its own module, so it is read on its own:
// without it every export route — the PAT surface included — drops out of the
// table and out of the contract.
const EXPORT_SRC: &str =
    include_str!("../../../../extensions/web/admin/analytics/src/routes/export_ssr.rs");
// The legacy paths, which answer 308 rather than HTML. They are a route table
// like any other and are exercised like one: a redirect that stops redirecting
// is exactly as much a contract change as a page that stops rendering.
const SSR_REDIRECT_SRC: &str =
    include_str!("../../../../extensions/web/src/router/admin/redirects.rs");

// The user-facing Drive pages, nested at `/drive` outside the admin plane.
const DRIVE_USER_SRC: &str =
    include_str!("../../../../extensions/web/admin/platform/src/routes/drive_user.rs");

const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

// A single method/path pair the router serves.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) struct MountedRoute {
    pub method: String,
    // The path as mounted, path parameters still in `{brace}` form.
    pub template: String,
}

impl MountedRoute {
    // A concrete URL for this route.
    //
    // Every path parameter is filled with a well-formed id that exists in no
    // table. That is the interesting case rather than a limitation: a route
    // asked for an id it cannot find owes the caller a 404, and one that
    // answers 500 instead is the exact defect this suite is here to catch.
    pub(crate) fn request_path(&self) -> String {
        self.template
            .split('/')
            .map(|seg| {
                if seg.starts_with('{') && seg.ends_with('}') {
                    UNKNOWN_ID
                } else {
                    seg
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    pub(crate) fn key(&self) -> String {
        format!("{} {}", self.method.to_uppercase(), self.template)
    }
}

// Well-formed, and deliberately absent from every table.
const UNKNOWN_ID: &str = "00000000-0000-4000-8000-000000000000";

pub(crate) fn mounted_routes() -> Vec<MountedRoute> {
    let mut routes = Vec::new();
    for (src, prefix) in [
        (ADMIN_API_SRC, ADMIN_API_PREFIX),
        (ANALYTICS_API_SRC, ADMIN_API_PREFIX),
        (IDENTITY_API_SRC, ADMIN_API_PREFIX),
        (IDENTITY_GROUPS_SRC, ADMIN_API_PREFIX),
        (MANAGED_SRC, ADMIN_API_PREFIX),
        (CATALOG_API_SRC, ADMIN_API_PREFIX),
        (CATALOG_SSR_SRC, SSR_PREFIX),
        (GOVERNANCE_API_SRC, ADMIN_API_PREFIX),
        (GOVERNANCE_SSR_SRC, SSR_PREFIX),
        (PLATFORM_API_SRC, ADMIN_API_PREFIX),
        (PLATFORM_SSR_SRC, SSR_PREFIX),
        (IDENTITY_PEOPLE_SRC, SSR_PREFIX),
        (IDENTITY_ACCOUNT_SRC, SSR_PREFIX),
        (IDENTITY_PUBLIC_SRC, SSR_PREFIX),
        (SSR_SRC, SSR_PREFIX),
        (ANALYTICS_OVERVIEW_SRC, SSR_PREFIX),
        (ANALYTICS_ACTIVITY_SRC, SSR_PREFIX),
        (ANALYTICS_ANALYSIS_SRC, SSR_PREFIX),
        (ANALYTICS_OBSERVABILITY_SRC, SSR_PREFIX),
        (EXPORT_SRC, SSR_PREFIX),
        (SSR_REDIRECT_SRC, SSR_PREFIX),
        (FARM_SSR_SRC, SSR_PREFIX),
        (FARM_FRAGMENTS_SRC, SSR_PREFIX),
        (FARM_API_SRC, ADMIN_API_PREFIX),
        (FARM_PUBLIC_SRC, FARM_PUBLIC_PREFIX),
        (DRIVE_USER_SRC, USER_BASE_URL),
    ] {
        parse(src, prefix, &mut routes);
    }
    routes.sort();
    routes.dedup();
    routes
}

// Split the source on `.route(` and read each call's path literal plus every
// method constructor applied to it.
fn parse(src: &str, prefix: &str, out: &mut Vec<MountedRoute>) {
    let calls: Vec<usize> = src.match_indices(".route(").map(|(i, _)| i).collect();
    for (n, &start) in calls.iter().enumerate() {
        let end = calls.get(n + 1).copied().unwrap_or(src.len());
        let segment = &src[start..end];

        let Some(template) = path_literal(segment) else {
            continue;
        };
        let mounted = if template == "/" {
            prefix.to_owned()
        } else {
            format!("{prefix}{template}")
        };

        for method in methods_in(segment) {
            out.push(MountedRoute {
                method,
                template: mounted.clone(),
            });
        }
    }
}

fn path_literal(segment: &str) -> Option<String> {
    let open = segment.find('"')?;
    let after = &segment[open + 1..];
    let close = after.find('"')?;
    let literal = &after[..close];
    assert!(
        literal.starts_with('/'),
        "route literal {literal:?} is not a path — the route-source parser needs updating"
    );
    Some(literal.to_owned())
}

// Method constructors applied within one `.route(...)` call, in declaration
// order. Matches only a bare `name(`, so handler identifiers that merely
// contain a method name (`get_gateway_handler`) are not mistaken for one.
fn methods_in(segment: &str) -> Vec<String> {
    let bytes = segment.as_bytes();
    let mut found = Vec::new();
    for method in METHODS {
        let needle = format!("{method}(");
        for (idx, _) in segment.match_indices(&needle) {
            let preceded_by_ident = idx > 0 && {
                let prev = bytes[idx - 1];
                prev.is_ascii_alphanumeric() || prev == b'_'
            };
            if !preceded_by_ident && !found.contains(&method.to_owned()) {
                found.push(method.to_owned());
            }
        }
    }
    assert!(
        !found.is_empty(),
        "no HTTP method found in route call: {segment:?}"
    );
    found
}
