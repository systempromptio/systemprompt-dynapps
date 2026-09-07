//! Render the same maintained assets in MCP Apps and the Cowork library.

use async_trait::async_trait;
use systemprompt::mcp::McpDomainResult;
use systemprompt::mcp::services::ui_renderer::templates::DashboardRenderer;
use systemprompt::mcp::services::ui_renderer::{UiRenderer, UiRendererRegistration, UiResource};
use systemprompt::models::a2a::{Artifact, Part};
use systemprompt::models::artifacts::ArtifactType;

struct AdminRenderer;

inventory::submit! {
    UiRendererRegistration { name: "astound-admin-reports", factory: || std::sync::Arc::new(AdminRenderer) }
}

#[async_trait]
impl UiRenderer for AdminRenderer {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Dashboard
    }

    async fn render(&self, artifact: &Artifact) -> McpDomainResult<UiResource> {
        let report = artifact.parts.iter().find_map(|part| {
            if let Part::Data(data) = part {
                // JSON: A2A data parts contain the serialized report protocol object.
                serde_json::from_value::<super::ReportOutput>(serde_json::Value::Object(
                    data.data.clone(),
                ))
                .ok()
            } else {
                None
            }
        });
        let Some(report) = report else {
            return DashboardRenderer::new().render(artifact).await;
        };
        let template = match report.report.as_str() {
            "activity" => {
                include_str!("../../../../../services/artifacts/admin-activity-requests/view.html")
            },
            "costs" => {
                include_str!("../../../../../services/artifacts/admin-usage-costs/view.html")
            },
            "users" => {
                include_str!("../../../../../services/artifacts/admin-users-directory/view.html")
            },
            "operations" => {
                include_str!("../../../../../services/artifacts/admin-ai-operations/view.html")
            },
            _ => include_str!("../../../../../services/artifacts/admin-project-delivery/view.html"),
        };
        let data = serde_json::to_string(&report)
            .map_err(|e| systemprompt::mcp::McpDomainError::Internal(e.to_string()))?
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
