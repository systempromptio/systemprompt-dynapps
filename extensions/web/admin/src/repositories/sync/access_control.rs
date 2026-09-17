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
use systemprompt::loader::services_root::ServicesRootBootstrap;

use super::access_control_rows::{awaiting_rows, rows};
use super::plane::{
    ActionCounts, Actor, ApplyOutcome, DeclarationSource, DriftKpi, EntityScope, Export,
    PlaneDrift, SyncMode, SyncPlane,
};
use super::state::{Applied, AppliedFrom, record_applied, record_declared};
use crate::error::{AdminError, AdminResult};
use crate::repositories::access_control::declared::DeclaredSet;
use crate::repositories::access_control::declared_load::{
    build_declared_from_doc, load_declared_set, marketplace_ids_from_services,
};
use crate::repositories::access_control::drift::{DriftReport, compute_drift};
use crate::repositories::access_control::export::render_export;
use crate::repositories::access_control::rules::{
    count_band_rules, list_band_rules, list_entity_defaults,
};
use crate::repositories::access_control::sync::{SyncOutcome, apply_sync, apply_sync_scoped};
use crate::repositories::config::gateway::registered_routes_from_services;
use crate::repositories::config::rules_yaml_loader::{RULES_FILE, parse_rules_doc};

pub const PLANE_ID: &str = "access_control";

#[derive(Debug, Clone, Copy, Default)]
pub struct AccessControlPlane;

// Why: one place assembles the declared set for every request-time caller so
// the page, the drift API and the apply all read the same file the same way
// — from the composed root, so a kit's marketplace validates once its
// bundle is active.
pub async fn declared_now(pool: &PgPool) -> AdminResult<DeclaredSet> {
    declared_from(pool, DeclarationSource::Disk).await
}

// Why: uploaded text is projected by the same code as the file — same route
// catalog, same marketplace ids — so a preview's drift is the drift the
// file would show once committed.
pub async fn declared_from(pool: &PgPool, from: DeclarationSource<'_>) -> AdminResult<DeclaredSet> {
    let registered = registered_routes_from_services()?;
    // Why: the catalog failing is not the file failing; the two get different
    // headlines so an operator fixes the tree, not the declaration.
    let marketplace_ids = marketplace_ids_from_services()
        .map_err(|source| AdminError::invalid("services tree could not be composed", source))?;
    let loaded = match from {
        DeclarationSource::Disk => {
            let profile = ProfileBootstrap::get()?;
            let services_path = ServicesRootBootstrap::active_root_or(&profile.paths.services);
            load_declared_set(pool, &services_path, &marketplace_ids, &registered).await
        },
        DeclarationSource::Text(text) => match parse_rules_doc(text) {
            Ok(doc) => build_declared_from_doc(pool, &doc, &marketplace_ids, &registered).await,
            Err(e) => Err(e),
        },
    };
    loaded.map_err(|source| AdminError::invalid("rules.yaml could not be loaded", source))
}

pub async fn drift_now(pool: &PgPool, declared: &DeclaredSet) -> AdminResult<DriftReport> {
    let rules = list_band_rules(pool).await?;
    let entities = list_entity_defaults(pool).await?;
    Ok(compute_drift(declared, &rules, &entities))
}

#[must_use]
pub fn applied_from() -> AppliedFrom {
    // Why: the hashes decorate the sync_state row; an unreadable profile or
    // cache (a test, a CLI) records an apply without them.
    // Why: discard-ok: an apply is never refused for want of a decoration
    let sources = super::sources::build_sources().ok();
    AppliedFrom {
        base_tree_hash: sources.as_ref().and_then(|s| s.base.tree_hash.clone()),
        composed_hash: sources.and_then(|s| s.composed_hash),
    }
}

impl AccessControlPlane {
    async fn record(
        &self,
        pool: &PgPool,
        declared: &DeclaredSet,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<()> {
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
        Ok(())
    }
}

impl From<SyncOutcome> for ApplyOutcome {
    fn from(outcome: SyncOutcome) -> Self {
        Self {
            inserted: outcome.inserted,
            updated: outcome.updated,
            deleted: outcome.deleted,
            entities_inserted: outcome.entities_inserted,
            entities_updated: outcome.entities_updated,
        }
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

    fn owner_url(&self) -> &'static str {
        "/admin/access-control"
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        self.drift_from(pool, DeclarationSource::Disk).await
    }

    async fn drift_from(
        &self,
        pool: &PgPool,
        from: DeclarationSource<'_>,
    ) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_band_rules(pool).await?).unwrap_or(0);
        // Why: an unreadable file is the card's headline, not a 500 — the
        // operator came to see the state of the declaration, and "it does
        // not parse" is that state.
        let declared = match declared_from(pool, from).await {
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
        if from.is_disk() {
            record_declared(pool, PLANE_ID, &declared_hash).await?;
        }
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
        actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome> {
        self.apply_from(pool, mode, actor, DeclarationSource::Disk)
            .await
    }

    // Why: a scoped apply is a participant syncing their own marketplaces.
    // It records to `sync_state` like any apply — the actor and mode are the
    // provenance — but the hash it records is the whole file's, since the
    // projection it leaves behind is still measured against the whole file.
    async fn apply_scoped(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
        scope: EntityScope<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_now(pool).await?;
        let outcome = apply_sync_scoped(pool, &declared, mode, scope).await?;
        self.record(pool, &declared, mode, actor).await?;
        Ok(outcome.into())
    }

    async fn apply_from(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
        from: DeclarationSource<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_from(pool, from).await?;
        let outcome = apply_sync(pool, &declared, mode).await?;
        self.record(pool, &declared, mode, actor).await?;
        Ok(outcome.into())
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
