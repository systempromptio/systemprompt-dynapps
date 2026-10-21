//! The `gateway_routes` plane: `services/ai/gateway.yaml`'s `routes:`
//! sequence projected into `gateway_routes`.
//!
//! The one plane whose declaration is also its runtime input. Core boots
//! the dispatcher from the file and never reads the table, so every
//! user-initiated apply ends by regenerating the file from the rows — after
//! *Overwrite* the two are equal by construction; after *Insert only* the file
//! gains the rows the console added, which the next restart dispatches. The
//! boot job seeds an empty table from the file and otherwise only reports; a
//! fresh image whose file moved on shows here as drift until an administrator
//! chooses a direction, and a console route from the previous image is *In
//! database, not in code* until it is applied or exported.

use async_trait::async_trait;
use sqlx::PgPool;

use super::gateway_routes_drift::rows;
use crate::repositories::gateway_routes::declared::{
    DeclaredRoutes, GATEWAY_FILE, bundle_owner, declared_routes_now, gateway_file_path,
};
use crate::repositories::gateway_routes::drift::compute_route_drift;
use crate::repositories::gateway_routes::render::{regenerate_gateway_file, render_routes_export};
use crate::repositories::gateway_routes::rows::{
    RouteWrite, SOURCE_CODE, count_gateway_routes, delete_gateway_route, list_gateway_routes,
    upsert_gateway_route,
};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::sync::access_control::applied_from;
use systemprompt_web_admin_core::repositories::sync::plane::{
    ActionCounts, Actor, ApplyOutcome, DriftKpi, Export, PlaneDrift, SyncMode, SyncPlane,
};
use systemprompt_web_admin_core::repositories::sync::plane_ids;
use systemprompt_web_admin_core::repositories::sync::registry::PlaneRegistration;
use systemprompt_web_admin_core::repositories::sync::state::{
    Applied, record_applied, record_declared,
};

pub const PLANE_ID: &str = plane_ids::GATEWAY_ROUTES;

pub const RUNTIME_NOTE: &str = "Core boots its dispatcher from services/ai/gateway.yaml and \
    never reads this table. Every console write and every apply regenerates the file from the \
    rows, so a change here is dispatched at the next restart.";

#[derive(Debug, Clone, Copy, Default)]
pub struct GatewayRoutesPlane;

inventory::submit! {
    PlaneRegistration { order: 3, build: || Box::new(GatewayRoutesPlane) }
}

async fn write_declared(
    pool: &PgPool,
    declared: &DeclaredRoutes,
    mode: SyncMode,
) -> AdminResult<ApplyOutcome> {
    let existing = list_gateway_routes(pool).await?;
    let mut outcome = ApplyOutcome::default();
    for (i, d) in declared.routes.iter().enumerate() {
        let present = existing.iter().any(|r| r.route.id == d.route.id);
        if present && mode == SyncMode::InsertOnly {
            continue;
        }
        let position = match mode {
            SyncMode::Overwrite => i,
            SyncMode::InsertOnly => existing.len() + i,
        };
        upsert_gateway_route(
            pool,
            &RouteWrite {
                route: &d.route,
                position: i32::try_from(position).unwrap_or(i32::MAX),
                explicit_id: d.explicit_id,
                source: SOURCE_CODE,
            },
        )
        .await?;
        if present {
            outcome.updated += 1;
        } else {
            outcome.inserted += 1;
        }
    }
    if mode == SyncMode::Overwrite {
        for r in &existing {
            if declared.find(&r.route.id).is_none() {
                outcome.deleted +=
                    usize::try_from(delete_gateway_route(pool, &r.route.id).await?).unwrap_or(0);
            }
        }
    }
    Ok(outcome)
}

#[async_trait]
impl SyncPlane for GatewayRoutesPlane {
    fn id(&self) -> &'static str {
        PLANE_ID
    }

    fn label(&self) -> &'static str {
        "Gateway routes"
    }

    fn source_file(&self) -> &'static str {
        GATEWAY_FILE
    }

    fn projection(&self) -> &'static str {
        "gateway_routes"
    }

    fn owner_url(&self) -> &'static str {
        "/admin/gateway"
    }

    fn runtime_note(&self) -> Option<&'static str> {
        Some(RUNTIME_NOTE)
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_gateway_routes(pool).await?).unwrap_or(0);
        let declared = match declared_routes_now() {
            Ok(d) => d,
            Err(e) => {
                return Ok(PlaneDrift {
                    in_db,
                    unreadable: Some(e.to_string()),
                    ..PlaneDrift::default()
                });
            },
        };
        let declared_hash = declared.declared_hash();
        record_declared(pool, PLANE_ID, &declared_hash).await?;
        let drift = compute_route_drift(&declared, &list_gateway_routes(pool).await?);
        let console_orphans = drift.only_in_db.iter().filter(|o| o.dashboard).count();
        Ok(PlaneDrift {
            declared_hash,
            declared_count: declared.routes.len(),
            in_db,
            kpis: vec![
                DriftKpi {
                    label: "Added in code",
                    value: drift.missing_in_db.len(),
                    note: "Insert only or Overwrite adds these",
                    tone: "ok",
                },
                DriftKpi {
                    label: "Removed from code",
                    value: drift.only_in_db.len(),
                    note: "Overwrite deletes these",
                    tone: "warn",
                },
                DriftKpi {
                    label: "Changed in code",
                    value: drift.changed.len() + usize::from(drift.reordered.is_some()),
                    note: "provider, model, fallback or order disagree",
                    tone: "warn",
                },
            ],
            actions: ActionCounts {
                insert: drift.missing_in_db.len(),
                insert_entities: 0,
                update: drift.changed.len() + usize::from(drift.reordered.is_some()),
                delete: drift.only_in_db.len(),
                delete_console: console_orphans,
                kept: 0,
            },
            is_clean: drift.is_clean(),
            rows: rows(&drift),
            unreadable: None,
        })
    }

    async fn apply(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_routes_now()?;
        let outcome = write_declared(pool, &declared, mode).await?;
        let user_initiated = matches!(actor, Actor::User(_));
        if user_initiated && bundle_owner().is_none() {
            regenerate_gateway_file(pool, &gateway_file_path()?).await?;
        }
        record_applied(
            pool,
            &Applied {
                plane: PLANE_ID,
                declared_hash: &declared.declared_hash(),
                mode: actor.recorded_mode(mode),
                actor: actor.as_str(),
                from: applied_from(),
            },
        )
        .await?;
        Ok(outcome)
    }

    async fn export(&self, pool: &PgPool) -> AdminResult<Option<Export>> {
        let rows = list_gateway_routes(pool).await?;
        let file = std::fs::read_to_string(gateway_file_path()?).map_err(|e| {
            systemprompt_web_admin_core::error::AdminError::invalid(
                "gateway.yaml could not be read",
                e,
            )
        })?;
        let body = render_routes_export(&file, &rows).map_err(|e| {
            systemprompt_web_admin_core::error::AdminError::invalid(
                "gateway.yaml could not be rendered",
                e,
            )
        })?;
        Ok(Some(Export {
            filename: "gateway.yaml",
            content_type: "application/yaml; charset=utf-8",
            row_count: rows.len(),
            body,
        }))
    }
}
