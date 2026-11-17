//! The two SSR pages. Both are console-only (the same `is_console` check the
//! other admin domains make in their handlers) and read-only. Anything farm-api
//! or systemprompt does not deliver is recorded on the page's [`Faults`], so
//! the shell shows it in its "could not be loaded" notice and the page still
//! answers 200.

use std::collections::HashMap;

use axum::Extension;
use axum::extract::Query;
use axum::response::Response;
use serde::Deserialize;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::client::{ALL_TENANTS, FarmApi, FarmApiError, is_all_tenants};
use crate::models::{TaskList, TenantRef, WhoAmI};
use crate::sp::SharedSp;
use crate::usage;
use crate::views::{self, TasksInput};
pub use crate::handlers_task::{TaskRead, read_sp, read_task, result_or_fault, task_page};

/// What a fault from systemprompt's tables is filed under.
pub const SP_FAULT: &str = "Systemprompt usage";
/// The value the Cost page used for "every tenant" before `*`.
const LEGACY_ALL: &str = "all";

#[derive(Debug, Default, Deserialize)]
pub struct TenantQuery {
    #[serde(default)]
    pub tenant: Option<String>,
    #[serde(default)]
    pub cursor: Option<String>,
}

impl TenantQuery {
    /// The tenant asked for, if the request names one. Empty, `*` and the
    /// legacy `all` of the old Cost page all mean "every tenant".
    pub fn tenant(&self) -> Option<&str> {
        self.tenant
            .as_deref()
            .map(str::trim)
            .filter(|t| !is_all_tenants(t) && *t != LEGACY_ALL)
    }

    /// The filter a list page sends to farm-api: the tenant asked for, else
    /// [`ALL_TENANTS`].
    pub fn filter(&self) -> String {
        self.tenant()
            .map_or_else(|| ALL_TENANTS.to_owned(), ToOwned::to_owned)
    }

    fn cursor(&self) -> Option<&str> {
        self.cursor
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
    }
}

pub fn require_console(shell: &Page) -> Result<(), AdminError> {
    if shell.user.is_console {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Admin access required.".to_owned()))
    }
}

/// Who the console's credential is, and so which tenants it may read. `None`
/// when farm-api could not say; the reason is on `faults`.
pub async fn identify(api: &FarmApi, faults: &mut Faults) -> Option<WhoAmI> {
    match api.whoami().await {
        Ok(who) => Some(who),
        Err(e) => {
            faults.record("Farm", &e);
            None
        },
    }
}

/// The one tenant a detail page reads: the one asked for, else the
/// credential's home tenant. Detail reads never take "all".
pub fn pick_tenant(query: &TenantQuery, who: &WhoAmI) -> String {
    query
        .tenant()
        .map_or_else(|| who.tenant.clone(), ToOwned::to_owned)
}

/// The tenants to offer. A credential without the platform scope gets no
/// `tenants` list from farm-api: offer its home tenant alone.
pub fn tenant_list(who: &WhoAmI) -> Vec<TenantRef> {
    if who.tenants.is_empty() && !who.tenant.is_empty() {
        return vec![TenantRef {
            tenant: who.tenant.clone(),
            name: None,
        }];
    }
    who.tenants.clone()
}

/// What every list page needs before it reads its own data: the faults so
/// far, the tenant filter and the tenants offered. `tenant` is the slug asked
/// for or [`ALL_TENANTS`], which is the default. `ready` is false when
/// farm-api could not say who the credential is (the reason is on `faults`).
pub struct Scope {
    pub faults: Faults,
    pub tenant: String,
    /// The credential's own tenant: what a detail page reads when the request
    /// names none.
    pub home: String,
    pub tenants: Vec<TenantRef>,
    pub ready: bool,
}

impl Scope {
    /// The tenant a detail read needs: the filter when it names one, else the
    /// home tenant.
    pub fn concrete(&self) -> &str {
        if is_all_tenants(&self.tenant) {
            &self.home
        } else {
            &self.tenant
        }
    }
}

pub async fn scope(api: &FarmApi, query: &TenantQuery) -> Scope {
    let mut faults = Faults::new();
    let who = identify(api, &mut faults).await;
    let ready = who.is_some();
    let (home, tenants) = who.as_ref().map_or_else(
        || (String::new(), Vec::new()),
        |w| (w.tenant.clone(), tenant_list(w)),
    );
    Scope {
        faults,
        tenant: query.filter(),
        home,
        tenants,
        ready,
    }
}

/// systemprompt's cost per gateway session for the tasks of one list page,
/// in one query. `None` when systemprompt could not be read.
async fn page_costs(
    sp: &SharedSp,
    list: Option<&TaskList>,
    faults: &mut Faults,
) -> Option<HashMap<String, i64>> {
    let ids: Vec<String> = list?
        .items
        .iter()
        .filter_map(|t| {
            usage::gateway_session_id(
                t.usage
                    .as_ref()
                    .and_then(|u| u.gateway_session_id.as_deref()),
            )
        })
        .collect();
    if ids.is_empty() {
        return Some(HashMap::new());
    }
    match sp.session_totals(&ids).await {
        Ok(totals) => Some(
            totals
                .into_iter()
                .map(|t| (t.session_id, t.cost_microdollars))
                .collect(),
        ),
        Err(e) => {
            faults.record(SP_FAULT, &e);
            None
        },
    }
}

/// The value of a farm-api read, or `None` with the failure filed under
/// `what` on the page's faults.
pub fn or_fault<T>(
    result: Result<T, FarmApiError>,
    what: &'static str,
    faults: &mut Faults,
) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            faults.record(what, &e);
            None
        },
    }
}

pub async fn tasks_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Extension(sp): Extension<SharedSp>,
    Query(query): Query<TenantQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let mut s = scope(&api, &query).await;
    let list = if s.ready {
        match api.list_tasks(&s.tenant, query.cursor()).await {
            Ok(list) => Some(list),
            Err(e) => {
                s.faults.record("Tasks", &e);
                None
            },
        }
    } else {
        None
    };
    let costs = page_costs(&sp, list.as_ref(), &mut s.faults).await;
    let ctx = views::tasks_page(&TasksInput {
        tenant: &s.tenant,
        tenants: &s.tenants,
        cursor: query.cursor(),
        list: list.as_ref(),
        costs: costs.as_ref(),
    });
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-tasks",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}
