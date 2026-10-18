//! Render the same maintained assets in MCP Apps and the Cowork library.

use systemprompt::mcp::McpDomainResult;
use systemprompt::mcp::services::ui_renderer::templates::DashboardRenderer;
use systemprompt::mcp::services::ui_renderer::{UiRenderer, UiRendererRegistration, UiResource};
use systemprompt::models::a2a::{Artifact, PartContent};
use systemprompt::models::artifacts::ArtifactType;

struct AdminRenderer;

inventory::submit! {
    UiRendererRegistration { name: "dynapps-admin-reports", factory: || std::sync::Arc::new(AdminRenderer) }
}

impl UiRenderer for AdminRenderer {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Dashboard
    }

    fn render(&self, artifact: &Artifact) -> McpDomainResult<UiResource> {
        // Why: core drops the resource on a renderer error; surface parse failures.
        let mut failure: Option<serde_json::Error> = None;
        let report = artifact.parts.iter().find_map(|part| {
            let PartContent::Data(data) = &part.content else {
                return None;
            };
            // JSON: protocol boundary
            match serde_json::from_value::<super::ReportOutput>(data.clone()) {
                Ok(report) => Some(report),
                Err(error) => {
                    failure = Some(error);
                    None
                },
            }
        });
        let Some(report) = report else {
            if let Some(error) = failure {
                return Err(systemprompt::mcp::McpDomainError::operation(
                    "admin report artifact does not match the ReportOutput contract",
                    error,
                ));
            }
            return DashboardRenderer::new().render(artifact);
        };
        let template = include_str!("../../../../../services/artifacts/admin-ai-usage/view.html");
        let data = serde_json::to_string(&report)
            .map_err(|e| systemprompt::mcp::McpDomainError::operation("serialize admin report", e))?
            .replace('<', "\\u003c")
            .replace('>', "\\u003e")
            .replace('&', "\\u0026");
        Ok(UiResource::new(
            template.replace("/*REPORT_DATA*/null", &data).replace(
                "/*MCP_BRIDGE*/",
                include_str!("../../../../../storage/files/js/admin-report-bridge.js"),
            ),
        ))
    }
}

#[must_use]
pub fn admin_artifact_shell() -> String {
    systemprompt::mcp::artifact_shell_template().replace(
        "    request(MCP_UI.INITIALIZE, {",
        &format!(
            "{}\n    request(MCP_UI.INITIALIZE, {{",
            include_str!("../../../../../storage/files/js/admin-report-shell.js")
        ),
    )
}
