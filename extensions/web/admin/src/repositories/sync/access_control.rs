//! The `access_control` plane: `services/access-control/rules.yaml` projected
//! into `access_control_rules` and `access_control_entities`.
//!
//! A thin adapter over the engine in [`crate::repositories::access_control`]
//! — declared set, drift, apply, export — that adds the two things the sync
//! page needs from every plane: a declared hash, and a `sync_state` row
//! written on every apply. The database-only rows a kit's own `access:`
//! block wrote (source `bundle:<name>`) surface here as *bundle* orphans.

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::UserId;
use systemprompt::loader::services_root::ServicesRootBootstrap;

use super::access_control_rows::{awaiting_rows, rows};
use super::plane::{ActionCounts, ApplyOutcome, DriftKpi, Export, PlaneDrift, SyncMode, SyncPlane};
use super::state::{Applied, AppliedFrom, record_applied, record_declared};
use crate::error::{AdminError, AdminResult};
use crate::repositories::access_control::declared::DeclaredSet;
use crate::repositories::access_control::declared_load::{
    load_declared_set, marketplace_ids_from_services,
};
use crate::repositories::access_control::drift::{DriftReport, compute_drift};
use crate::repositories::access_control::export::render_export;
use crate::repositories::access_control::rules::{
    count_band_rules, list_band_rules, list_entity_defaults,
};
use crate::repositories::access_control::sync::apply_sync;
use crate::repositories::config::gateway::registered_routes_from_services;
use crate::repositories::config::rules_yaml_loader::RULES_FILE;

pub const PLANE_ID: &str = "access_control";

#[derive(Debug, Clone, Copy, Default)]
pub struct AccessControlPlane;

// Why: one place assembles the declared set for every request-time caller so
// the page, the drift API and the apply all read the same file the same way
// — from the composed root, so a kit's marketplace validates once its
// bundle is active.
pub async fn declared_now(pool: &PgPool) -> AdminResult<DeclaredSet> {
    let profile = ProfileBootstrap::get()?;
    let services_path = ServicesRootBootstrap::active_root_or(&profile.paths.services);
    let registered = registered_routes_from_services()?;
    load_declared_set(
        pool,
        &services_path,
        &marketplace_ids_from_services(),
        &registered,
    )
    .await
    // Why: 400-boundary classification — an unparsable declaration is the
    // operator's input, not a server fault. lint-ok: error-adapt
    .map_err(|e| AdminError::BadRequest(format!("rules.yaml could not be loaded: {e}")))
}

pub async fn drift_now(pool: &PgPool, declared: &DeclaredSet) -> AdminResult<DriftReport> {
    let rules = list_band_rules(pool).await?;
    let entities = list_entity_defaults(pool).await?;
    Ok(compute_drift(declared, &rules, &entities))
}

#[must_use]
pub fn applied_from() -> AppliedFrom {
    // Why: discard-ok: the hashes decorate the sync_state row; an unreadable
    // profile or cache (a test, a CLI) records an apply without them rather than
    // refusing the apply.
    let sources = super::sources::build_sources().ok();
    AppliedFrom {
        base_tree_hash: sources.as_ref().and_then(|s| s.base.tree_hash.clone()),
        composed_hash: sources.and_then(|s| s.composed_hash),
    }
}

#[async_trait]
impl SyncPlane for AccessControlPlane {
    fn id(&self) -> &'static str {
        PLANE_ID
    }

    fn label(&self) -> &'static str {
        "Access control"
    }

    fn source_file(&self) -> &'static str {
        RULES_FILE
    }

    fn projection(&self) -> &'static str {
        "access_control_rules · access_control_entities"
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_band_rules(pool).await?).unwrap_or(0);
        // Why: an unreadable file is the card's headline, not a 500 — the
        // operator came to see the state of the declaration, and "it does
        // not parse" is that state.
        let declared = match declared_now(pool).await {
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
        let report = drift_now(pool, &declared).await?;
        let counts = report.counts();
        let mut rows = rows(&report);
        rows.extend(awaiting_rows(&declared));
        let is_clean = report.is_clean();
        Ok(PlaneDrift {
            declared_hash,
            declared_count: declared.rule_count(),
            in_db,
            kpis: vec![
                DriftKpi {
                    label: "In code, not in database",
                    value: counts.missing_in_db,
                    note: "Insert only adds these",
                    tone: "info",
                },
                DriftKpi {
                    label: "In database, not in code",
                    value: counts.only_in_db,
                    note: "Overwrite deletes those on declared entities",
                    tone: "warn",
                },
                DriftKpi {
                    label: "Differ",
                    value: counts.changed,
                    note: "access or reason disagrees",
                    tone: "warn",
                },
                DriftKpi {
                    label: "Defaults differ",
                    value: counts.default_changed,
                    note: "open vs closed",
                    tone: "warn",
                },
                DriftKpi {
                    label: "Awaiting a bundle",
                    value: declared.awaiting.len(),
                    note: "declared with owner:, bundle not active",
                    tone: "muted",
                },
            ],
            actions: ActionCounts {
                insert: counts.missing_in_db,
                insert_entities: counts.entities_missing,
                update: counts.changed + counts.default_changed,
                delete: counts.only_in_db_governed,
                delete_console: counts.only_in_db_dashboard,
            },
            rows,
            is_clean,
            unreadable: None,
        })
    }

    async fn apply(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: &UserId,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_now(pool).await?;
        let outcome = apply_sync(pool, &declared, mode).await?;
        record_applied(
            pool,
            &Applied {
                plane: PLANE_ID,
                declared_hash: &declared.declared_hash(),
                mode: Some(mode),
                actor: actor.as_str(),
                from: applied_from(),
            },
        )
        .await?;
        Ok(ApplyOutcome {
            inserted: outcome.inserted,
            updated: outcome.updated,
            deleted: outcome.deleted,
            entities_inserted: outcome.entities_inserted,
            entities_updated: outcome.entities_updated,
        })
    }

    // Why: renders the database as the file for copy-out only; the instance
    // never writes into `services/`. Committing the result is the operator's
    // act.
    async fn export(&self, pool: &PgPool) -> AdminResult<Option<Export>> {
        let rules = list_band_rules(pool).await?;
        let entities = list_entity_defaults(pool).await?;
        // Why: the owners come from the file, not the database — an
        // unreadable file exports without them rather than not at all.
        let owners = declared_now(pool)
            .await
            .map(|d| d.owners)
            .unwrap_or_default();
        Ok(Some(Export {
            filename: "rules.yaml",
            content_type: "application/yaml; charset=utf-8",
            row_count: rules.len(),
            body: render_export(&rules, &entities, &owners),
        }))
    }
}
