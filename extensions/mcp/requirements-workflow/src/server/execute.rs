//! Tool dispatch: every operation re-checks project access, revision and the
//! frozen evidence hash before it touches the workflow.
use super::tools::{Handler, ToolInput};
use crate::evidence::snapshot;
use crate::model::{Gate, Phase, Workflow, WorkflowId, digest};
use crate::store::StoredWorkflow;
use crate::{jobs, publication, store, validation};
use anyhow::{Result, ensure};
// JSON: every tool reply is the workflow document plus operation-specific
// fields, serialised straight into the text artifact the client reads.
use serde_json::{Value, json};
use systemprompt::identifiers::UserId;
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
            return self.start(input, user).await;
        }
        let id = WorkflowId(required(input.workflow_id.take(), "workflow_id")?.parse()?);
        let mut stored = store::get(pool, user, id).await?;
        if self.name == "get_requirements_workflow" {
            let jobs = jobs::list(pool, &stored).await?;
            let mut value = serde_json::to_value(stored)?;
            value["validation_jobs"] = serde_json::to_value(jobs)?;
            return Ok(value);
        }
        ensure!(
            input.expected_revision == Some(stored.revision),
            "Reload workflow; expected_revision does not match"
        );
        let current = snapshot(&self.server.knowledge, user).await?;
        let current_hash = digest(&current)?;
        if self.name == "refresh_requirements_evidence" {
            if current_hash != stored.workflow.evidence_hash {
                stored.workflow.evidence = current;
                stored.workflow.evidence_hash = current_hash;
                stored.workflow.artifacts.clear();
                stored.workflow.grooming_review = None;
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

    async fn start(&self, input: ToolInput, user: &UserId) -> Result<Value> {
        let pool = &self.server.pool;
        ensure!(
            input.delivery_scope.as_deref() == Some("functional"),
            "Select functional scope explicitly. Split integration services into separate changes before discovery."
        );
        let project = input.project.as_deref().unwrap_or("storefront");
        store::require_project(pool, user, project).await?;
        let evidence = snapshot(&self.server.knowledge, user).await?;
        let workflow = Workflow::new(required(input.name, "name")?, evidence)?;
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
                    return jobs::start((**pool).clone(), user.clone(), stored, phase).await;
                },
                "request_requirements_review" => {
                    let gate = match input.gate.as_deref() {
                        Some("grooming") => Gate::Grooming,
                        Some("publication") => Gate::Publication,
                        _ => anyhow::bail!("gate must be grooming or publication"),
                    };
                    stored.workflow.request_review(gate)?;
                },
                "generate_fsd_scaffold" => {
                    return Ok(serde_json::to_value(
                        validation::scaffold(&stored.workflow).await?,
                    )?);
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
