//! Tool dispatch: every operation re-checks project access, revision and the
//! frozen evidence hash before it touches the workflow.
use super::tools::{Handler, ToolInput};
use crate::evidence::snapshot;
use crate::model::{Gate, Phase, Workflow, WorkflowId, digest};
use crate::store::StoredWorkflow;
use crate::{publication, store, validation};
use anyhow::{Result, ensure};
// JSON: every tool reply is the workflow document plus operation-specific
// fields, serialised straight into the text artifact the client reads.
use serde_json::{Value, json};
use systemprompt::models::execution::context::RequestContext as SysContext;

fn parse_phase(name: &str) -> Result<Phase> {
    Phase::ALL
        .into_iter()
        .find(|p| p.name() == name)
        .ok_or_else(|| anyhow::anyhow!("Unknown phase"))
}

fn required<T>(value: Option<T>, what: &str) -> Result<T> {
    value.ok_or_else(|| anyhow::anyhow!("{what} is required"))
}

impl Handler<'_> {
    pub(super) async fn execute(&self, mut input: ToolInput, ctx: &SysContext) -> Result<Value> {
        let pool = &self.server.pool;
        let user = ctx.user_id();
        if self.name == "start_requirements_workflow" {
            return self.start(input, ctx).await;
        }
        let id = WorkflowId(required(input.workflow_id.take(), "workflow_id")?.parse()?);
        let mut stored = store::get(pool, user, id).await?;
        if self.name == "get_requirements_workflow" {
            return Ok(serde_json::to_value(stored)?);
        }
        ensure!(
            input.expected_revision == Some(stored.revision),
            "Reload workflow; expected_revision does not match"
        );
        let mut selected = stored
            .workflow
            .evidence
            .iter()
            .map(|e| crate::model::SourceSelection {
                source_type: e.source_type.clone(),
                source_ref: e.source_ref.clone(),
            })
            .collect::<Vec<_>>();
        if self.name == "refresh_requirements_evidence" {
            if let Some(sources) = input.sources.take() {
                selected = sources;
            }
        }
        let current = snapshot(&self.server.knowledge, ctx, self.bearer, &selected).await?;
        let current_hash = digest(&current)?;
        if self.name == "refresh_requirements_evidence" {
            let scope = input
                .delivery_scope
                .take()
                .unwrap_or_else(|| stored.workflow.delivery_scope.clone());
            ensure!(
                !scope.trim().is_empty() && scope.len() <= 4000,
                "Describe the delivery scope (1–4000 bytes)"
            );
            if current_hash != stored.workflow.evidence_hash
                || scope != stored.workflow.delivery_scope
            {
                stored.workflow.delivery_scope = scope;
                stored.workflow.evidence = current;
                stored.workflow.evidence_hash = current_hash;
                stored.workflow.artifacts.clear();
                stored.workflow.context_review = None;
                stored.workflow.publication_review = None;
            }
            stored.revision = store::save(pool, user, stored.revision, &stored, self.name).await?;
            return Ok(serde_json::to_value(stored)?);
        }
        ensure!(
            current_hash == stored.workflow.evidence_hash,
            "Source evidence changed. Refresh the snapshot; dependent artifacts and approvals must be regenerated."
        );
        self.advance(input, ctx, stored).await
    }

    async fn start(&self, input: ToolInput, ctx: &SysContext) -> Result<Value> {
        let pool = &self.server.pool;
        let user = ctx.user_id();
        ensure!(
            input
                .delivery_scope
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty() && s.len() <= 4000),
            "Describe the delivery scope explicitly (1–4000 bytes)."
        );
        let project = required(input.project.as_deref(), "project")?;
        store::require_project(pool, user, project).await?;
        let evidence = snapshot(
            &self.server.knowledge,
            ctx,
            self.bearer,
            &required(input.sources, "sources")?,
        )
        .await?;
        let workflow = Workflow::new(
            required(input.name, "name")?,
            required(input.delivery_scope, "delivery_scope")?,
            evidence,
        )?;
        Ok(serde_json::to_value(
            store::create(pool, user, project, workflow).await?,
        )?)
    }

    async fn advance(
        &self,
        input: ToolInput,
        ctx: &SysContext,
        mut stored: StoredWorkflow,
    ) -> Result<Value> {
        let pool = &self.server.pool;
        let user = ctx.user_id();
        if let Some(name) = self.name.strip_prefix("submit_") {
            stored
                .workflow
                .submit(parse_phase(name)?, required(input.files, "files")?)?;
        } else {
            match self.name {
                "validate_requirements_phase" => {
                    let phase = parse_phase(required(input.phase.as_deref(), "phase")?)?;
                    let result = validation::validate(&stored.workflow, phase)?;
                    stored
                        .workflow
                        .artifacts
                        .get_mut(&phase)
                        .ok_or_else(|| anyhow::anyhow!("Submit the artifact first"))?
                        .validation = Some(result);
                },
                "request_requirements_review" => {
                    let gate = match input.gate.as_deref() {
                        Some("context") => Gate::Context,
                        Some("publication") => Gate::Publication,
                        _ => anyhow::bail!("gate must be context or publication"),
                    };
                    stored.workflow.request_review(gate)?;
                },
                "set_requirements_destination" => {
                    stored
                        .workflow
                        .set_destination(required(input.destination, "destination")?)?;
                },
                "publish_requirements" => {
                    return publication::publish(pool, ctx, self.bearer, &mut stored).await;
                },
                "verify_requirements_publication" => {
                    return publication::verify(pool, ctx, self.bearer, &stored).await;
                },
                _ => anyhow::bail!("Unknown operation"),
            }
        }
        let id = stored.workflow.id.0;
        stored.revision = store::save(pool, user, stored.revision, &stored, self.name).await?;
        Ok(json!({"workflow":stored,"review_url":format!("/admin/requirements/{id}")}))
    }
}
