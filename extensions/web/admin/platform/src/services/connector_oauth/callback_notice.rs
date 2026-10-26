//! Credential-free callback notices with bounded, typed fields for the SSR
//! page.

use super::Provider;
use super::failure::FailureKind;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize, Default)]
pub struct CallbackNotice {
    #[serde(rename = "connection_provider")]
    provider: Option<String>,
    #[serde(rename = "connection_stage")]
    stage: Option<String>,
    #[serde(rename = "connection_failure")]
    failure: Option<FailureKind>,
    #[serde(rename = "connection_ref")]
    reference: Option<Uuid>,
}

impl CallbackNotice {
    pub fn message(
        &self,
        services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    ) -> Option<String> {
        let provider = Provider::try_from(self.provider.clone()?).ok()?;
        let stage = self.stage.as_deref()?;
        let title = match stage {
            "token" => "authorization exchange",
            "initialize" => "MCP initialization",
            "tools" => "MCP tools check",
            "identity" => "provider identity check",
            _ => return None,
        };
        let kind = self.failure?;
        let reference = self.reference?;
        Some(format!(
            "{} failed during {title}. {} Reference: {reference}",
            provider.display_name(&services.config()),
            kind.guidance()
        ))
    }
}

pub(crate) fn redirect(
    provider: &Provider,
    stage: &str,
    kind: FailureKind,
    reference: Uuid,
) -> String {
    let failure = serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default();
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("connection_provider", provider.slug())
        .append_pair("connection_stage", stage)
        .append_pair("connection_failure", &failure)
        .append_pair("connection_ref", &reference.to_string())
        .finish();
    format!("/admin/connectors?{query}#connector-{}", provider.slug())
}
