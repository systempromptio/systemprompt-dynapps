//! What the caller's access grants: marketplaces, plugins, skills and MCP
//! servers, resolved through the same entitlement matrix the bridge
//! manifest and the admin catalog use, read-only.

use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_web_admin_core::catalog_index::manifests::list_marketplace_configs;
use systemprompt_web_admin_core::catalog_index::mcp_servers::list_mcp_servers;
use systemprompt_web_admin_core::catalog_index::plugin_catalog::{
    list_plugin_catalog, list_skill_catalog,
};
use systemprompt_web_admin_core::repositories::entitlement::{
    SectionInput, filter_catalog_for_user,
};

use super::outputs::{CatalogItemOutput, CatalogSectionOutput};
use super::{CatalogListInput, CatalogOutput, ToolCtx, ToolError, ToolResult};

const SECTIONS: [&str; 4] = ["marketplace", "plugin", "skill", "mcp_server"];

fn opt(desc: String) -> Option<String> {
    if desc.trim().is_empty() {
        None
    } else {
        Some(desc)
    }
}

fn sections_for(ctx: &ToolCtx<'_>, only: &str) -> ToolResult<Vec<SectionInput>> {
    let path = &ctx.env()?.services_path;
    let wanted = |name: &str| only.is_empty() || only == name;
    let mut out: Vec<SectionInput> = Vec::new();
    if wanted("marketplace") {
        let rows = list_marketplace_configs(path)?
            .into_iter()
            .map(|m| (m.id.as_str().to_owned(), m.name, opt(m.description)))
            .collect();
        out.push(("marketplace".into(), "Workspaces".into(), rows));
    }
    if wanted("plugin") {
        let rows = list_plugin_catalog(path)?
            .into_iter()
            .map(|p| (p.id.as_str().to_owned(), p.name, opt(p.description)))
            .collect();
        out.push(("plugin".into(), "Plugins".into(), rows));
    }
    if wanted("skill") {
        let rows = list_skill_catalog(path)?
            .into_iter()
            .map(|s| (s.id.as_str().to_owned(), s.name, opt(s.description)))
            .collect();
        out.push(("skill".into(), "Skills".into(), rows));
    }
    if wanted("mcp_server") {
        let rows = list_mcp_servers(path)?
            .into_iter()
            .map(|s| {
                let id = s.id.as_str().to_owned();
                (id.clone(), id, opt(s.description))
            })
            .collect();
        out.push(("mcp_server".into(), "MCP servers".into(), rows));
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
pub struct CatalogListHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for CatalogListHandler<'_> {
    type Input = CatalogListInput;
    type Output = CatalogOutput;
    fn tool_name(&self) -> &'static str {
        "catalog_list"
    }
    fn description(&self) -> &'static str {
        "The workspaces, plugins, skills and MCP servers your access grants on this instance, with your roles and project memberships. Read-only; the same resolution the signed manifest uses."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: CatalogListInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(CatalogOutput, String), ErrorData> {
        catalog(self.ctx, &input).await.map_err(ErrorData::from)
    }
}

async fn catalog(
    ctx: &ToolCtx<'_>,
    input: &CatalogListInput,
) -> ToolResult<(CatalogOutput, String)> {
    let only = input.section.trim();
    if !only.is_empty() && !SECTIONS.contains(&only) {
        return Err(ToolError::Invalid(format!(
            "`section` must be one of {} (got {only:?})",
            SECTIONS.join(", ")
        )));
    }
    let sections_in = sections_for(ctx, only)?;
    let viewer = &ctx.viewer;
    let matrix =
        filter_catalog_for_user(ctx.pool()?, ctx.services()?, &viewer.user_id, sections_in)
            .await?
            .ok_or_else(|| {
                ToolError::Failure("the signed-in user has no entitlement record".to_owned())
            })?;
    let sections: Vec<CatalogSectionOutput> = matrix
        .sections
        .into_iter()
        .map(|sec| CatalogSectionOutput {
            entity_type: sec.entity_type,
            label: sec.label,
            items: sec
                .rows
                .into_iter()
                .filter(|r| r.effective == "allow")
                .map(|r| CatalogItemOutput {
                    id: r.entity_id,
                    name: r.entity_name,
                    description: r.description,
                })
                .collect(),
        })
        .collect();
    let granted: usize = sections.iter().map(|s| s.items.len()).sum();
    let summary = format!(
        "{granted} entities granted across {} sections",
        sections.len()
    );
    Ok((
        CatalogOutput {
            user_id: viewer.user_id.clone(),
            roles: viewer.roles.clone(),
            project_ids: viewer.project_ids.clone(),
            sections,
        },
        summary,
    ))
}
