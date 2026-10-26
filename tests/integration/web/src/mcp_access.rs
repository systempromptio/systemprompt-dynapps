use std::sync::Arc;

use systemprompt::models::auth::TokenAuthority;
use systemprompt_mcp_agent::CliLocation;

pub(crate) fn token_authority() -> Arc<dyn TokenAuthority> {
    Arc::new(systemprompt::security::jwt::DeploymentTokenAuthority::new(
        dynapps_test_common::ephemeral_signing_keys(),
        crate::jobs_context::test_config().jwt_issuer.clone(),
    ))
}

pub(crate) fn cli() -> CliLocation {
    CliLocation {
        bin: std::path::PathBuf::from("systemprompt"),
        workdir: std::env::temp_dir(),
    }
}
