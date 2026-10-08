//! Typed connector failures decide credential retention independently of HTTP
//! status.

use super::Provider;
use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::AdminError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    GrantRejected,
    Configuration,
    Permission,
    Outage,
    InvalidResponse,
    RefreshUncertain,
}

impl FailureKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::GrantRejected => "grant_rejected",
            Self::Configuration => "provider_configuration",
            Self::Permission => "provider_permission_denied",
            Self::Outage => "provider_unavailable",
            Self::InvalidResponse => "provider_invalid_response",
            Self::RefreshUncertain => "refresh_outcome_unknown",
        }
    }

    pub const fn guidance(self) -> &'static str {
        match self {
            Self::GrantRejected => {
                "Salesforce or the provider rejected authorization. Reconnect to continue."
            },
            Self::Configuration => {
                "The provider rejected the app configuration. Ask the platform administrator to check its client credentials and OAuth settings."
            },
            Self::Permission => {
                "The provider refused access. Ask its administrator to check the app policy and your permissions."
            },
            Self::Outage => "The provider is temporarily unavailable. Try Test connection shortly.",
            Self::InvalidResponse => {
                "The provider returned an invalid authorization response. Try Test connection or ask the platform administrator to investigate."
            },
            Self::RefreshUncertain => {
                "The refresh response was lost or unusable; its token may have rotated. Reconnect before using this provider again."
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct FlowFailure {
    pub(crate) kind: FailureKind,
    pub(crate) error: AdminError,
}

pub(crate) type FlowResult<T> = Result<T, FlowFailure>;

impl FlowFailure {
    pub(crate) fn new(kind: FailureKind) -> Self {
        let message = kind.guidance().to_owned();
        let error = match kind {
            FailureKind::GrantRejected => AdminError::Unauthorized(message),
            FailureKind::Permission => AdminError::Forbidden(message),
            FailureKind::Configuration | FailureKind::RefreshUncertain => {
                AdminError::Unavailable(message)
            },
            FailureKind::Outage | FailureKind::InvalidResponse => AdminError::Upstream(message),
        };
        Self { kind, error }
    }

    pub(crate) const fn verification(error: AdminError) -> Self {
        let kind = match error {
            AdminError::Unauthorized(_) | AdminError::Forbidden(_) => FailureKind::Permission,
            _ => FailureKind::Outage,
        };
        Self { kind, error }
    }
}

// Why: Salesforce can reject concurrent refreshes without revoking the grant;
// Atlassian uses unauthorized_client for a specifically retired refresh token.
pub(crate) fn oauth_failure(
    provider: &Provider,
    status: u16,
    code: Option<&str>,
    description: Option<&str>,
) -> FailureKind {
    if status >= 500 || status == 429 {
        return FailureKind::Outage;
    }
    let description = description.unwrap_or_default().to_ascii_lowercase();
    if provider.is_salesforce() && description.contains("already being processed") {
        return FailureKind::Outage;
    }
    match code {
        Some("invalid_grant") => FailureKind::GrantRejected,
        Some("unauthorized_client")
            if *provider == Provider::Atlassian
                && description.contains("refresh_token is invalid") =>
        {
            FailureKind::GrantRejected
        },
        Some(
            "invalid_client"
            | "unauthorized_client"
            | "invalid_scope"
            | "unsupported_grant_type"
            | "invalid_request",
        ) => FailureKind::Configuration,
        Some("access_denied" | "insufficient_scope") => FailureKind::Permission,
        Some("server_error" | "temporarily_unavailable") => FailureKind::Outage,
        _ if status == 403 => FailureKind::Permission,
        _ => FailureKind::InvalidResponse,
    }
}

pub(crate) fn safe_oauth_code(code: Option<&str>) -> &'static str {
    match code {
        Some("invalid_grant") => "invalid_grant",
        Some("invalid_client") => "invalid_client",
        Some("unauthorized_client") => "unauthorized_client",
        Some("invalid_scope") => "invalid_scope",
        Some("unsupported_grant_type") => "unsupported_grant_type",
        Some("invalid_request") => "invalid_request",
        Some("access_denied") => "access_denied",
        Some("insufficient_scope") => "insufficient_scope",
        Some("server_error") => "server_error",
        Some("temporarily_unavailable") => "temporarily_unavailable",
        _ => "unknown",
    }
}
