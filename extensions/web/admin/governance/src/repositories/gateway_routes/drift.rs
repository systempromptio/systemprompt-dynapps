//! Declared routes against the rows: missing, orphaned, changed, and the
//! one difference the other planes do not have — order.
//!
//! Order is policy for this plane (the first route that matches a request
//! wins), so two sets with the same routes in a different sequence are not in
//! step. It is
//! reported once, as a single `reordered` line naming both sequences, rather
//! than as a change on every route that moved.

use serde::Serialize;

use super::declared::{DeclaredRoutes, route_fingerprint};
use super::rows::{RouteRow, SOURCE_DASHBOARD};
use systemprompt_web_admin_core::types::GatewayRouteView;

#[derive(Debug, Clone, Serialize)]
pub struct ChangedRoute {
    pub id: String,
    pub fields: Vec<String>,
    pub in_code: String,
    pub in_db: String,
    pub dashboard: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrphanRoute {
    pub id: String,
    pub summary: String,
    pub dashboard: bool,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct RouteDrift {
    pub missing_in_db: Vec<String>,
    pub only_in_db: Vec<OrphanRoute>,
    pub changed: Vec<ChangedRoute>,
    pub reordered: Option<(Vec<String>, Vec<String>)>,
}

impl RouteDrift {
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.missing_in_db.is_empty()
            && self.only_in_db.is_empty()
            && self.changed.is_empty()
            && self.reordered.is_none()
    }
}

#[must_use]
pub fn summarise_route(route: &GatewayRouteView) -> String {
    let mut s = route.models_line();
    if let Some(name) = route.name.as_deref().filter(|n| !n.is_empty()) {
        s = format!("{name}: {s}");
    }
    if !route.deployments.is_empty() {
        let pinned: Vec<&str> = route
            .deployments
            .iter()
            .map(|d| d.endpoint.as_str())
            .collect();
        s.push_str(&format!(" → {}", pinned.join(", then ")));
    }
    if route.requires.is_some() {
        s.push_str(", requires");
    }
    if route.when.is_some() {
        s.push_str(", when");
    }
    if let Some(strategy) = &route.strategy {
        s.push_str(&format!(", {strategy}"));
    }
    if !route.context_fallbacks.is_empty() {
        s.push_str(&format!(
            ", context fallback {}",
            route.context_fallbacks.join(", ")
        ));
    }
    s
}

fn changed_fields(code: &GatewayRouteView, db: &GatewayRouteView) -> Vec<String> {
    let mut out = Vec::new();
    let mut check = |name: &str, differs: bool| {
        if differs {
            out.push(name.to_owned());
        }
    };
    check("name", code.name != db.name);
    check("description", code.description != db.description);
    check("models", code.models != db.models);
    check("deployments", code.deployments != db.deployments);
    check("alias", code.alias != db.alias);
    check("when", code.when != db.when);
    check("requires", code.requires != db.requires);
    check("strategy", code.strategy != db.strategy);
    check(
        "context_fallbacks",
        code.context_fallbacks != db.context_fallbacks,
    );
    out
}

#[must_use]
pub fn compute_route_drift(declared: &DeclaredRoutes, rows: &[RouteRow]) -> RouteDrift {
    let mut drift = RouteDrift::default();
    for d in &declared.routes {
        match rows.iter().find(|r| r.route.id == d.route.id) {
            None => drift.missing_in_db.push(d.route.id.clone()),
            Some(row) => {
                if route_fingerprint(&d.route) != route_fingerprint(&row.route) {
                    drift.changed.push(ChangedRoute {
                        id: d.route.id.clone(),
                        fields: changed_fields(&d.route, &row.route),
                        in_code: summarise_route(&d.route),
                        in_db: summarise_route(&row.route),
                        dashboard: row.source == SOURCE_DASHBOARD,
                    });
                }
            },
        }
    }
    for row in rows {
        if declared.find(&row.route.id).is_none() {
            drift.only_in_db.push(OrphanRoute {
                id: row.route.id.clone(),
                summary: summarise_route(&row.route),
                dashboard: row.source == SOURCE_DASHBOARD,
            });
        }
    }
    let shared = |ids: Vec<String>| -> Vec<String> {
        ids.into_iter()
            .filter(|id| declared.find(id).is_some() && rows.iter().any(|r| &r.route.id == id))
            .collect()
    };
    let code_order = shared(declared.routes.iter().map(|d| d.route.id.clone()).collect());
    let db_order = shared(rows.iter().map(|r| r.route.id.clone()).collect());
    if code_order != db_order {
        drift.reordered = Some((code_order, db_order));
    }
    drift
}
