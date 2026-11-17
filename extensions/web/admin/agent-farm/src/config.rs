//! `services/web/config/farm.yaml`: where farm-api answers and which secret
//! holds the console's farm-api credential.
//!
//! The file names the secret, never its value: the token is read from the
//! secrets store on every call, so a rotated secret is used without a restart.
//! A missing file is a console without a farm (every page says so), never a
//! boot error; a file that does not parse is logged and treated the same way.

use std::path::Path;

use serde::Deserialize;

pub const CONFIG_FILE: &str = "web/config/farm.yaml";

pub const DEFAULT_TOKEN_SECRET: &str = "farm_api_token";

fn default_token_secret() -> String {
    DEFAULT_TOKEN_SECRET.to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FarmConfig {
    #[serde(default)]
    pub api_url: String,
    #[serde(default = "default_token_secret")]
    pub token_secret: String,
}

impl Default for FarmConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            token_secret: default_token_secret(),
        }
    }
}

impl FarmConfig {
    pub fn parse(text: &str) -> Result<Self, serde_yaml::Error> {
        if text.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_yaml::from_str(text)
    }

    pub fn load(services_root: &Path) -> Self {
        let path = services_root.join(CONFIG_FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "Farm console config could not be read; the console has no farm");
                return Self::default();
            },
        };
        Self::parse(&text).unwrap_or_else(|e| {
            tracing::warn!(path = %path.display(), error = %e, "Farm console config does not parse; the console has no farm");
            Self::default()
        })
    }
}
