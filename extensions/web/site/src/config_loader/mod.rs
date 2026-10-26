//! Loading of the `services/web/` YAML tree into one [`SiteConfig`].
//!
//! Lives beside the config types it deserialises, so adding a YAML-backed
//! page section is one edit in this crate. The web extension loads the tree
//! from the active services root its context hands it: once when the router is
//! built, which attaches it as an axum `Extension` for handlers, and once per
//! content-pipeline hook. A section whose file is absent is `None`; one whose
//! file does not parse is logged and served as absent, never a crash.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;

#[doc(hidden)]
pub mod skills;

use skills::load_skills_page_config;

use crate::homepage::HomepageConfig;
use crate::navigation::{BrandingConfig, NavigationConfig};
use crate::skills_page::SkillsPageConfig;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to parse {config_name}: {source}")]
    Parse {
        config_name: String,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Every `services/web/` section the public site renders from.
#[derive(Debug, Clone, Default)]
pub struct SiteConfig {
    pub branding: Option<BrandingConfig>,
    pub navigation: Option<Arc<NavigationConfig>>,
    pub homepage: Option<Arc<HomepageConfig>>,
    pub skills_page: Option<Arc<SkillsPageConfig>>,
}

impl SiteConfig {
    #[must_use]
    pub fn load_from(services: &Path) -> Self {
        Self {
            branding: section("branding", load_branding_config(services)),
            navigation: section("navigation", load_navigation_config(services)),
            homepage: section("homepage", load_homepage_config(services)),
            skills_page: section("skills page", load_skills_page_config(services)),
        }
    }
}

fn section<T>(name: &str, loaded: Result<Option<T>, ConfigError>) -> Option<T> {
    match loaded {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(
                %error,
                config = name,
                "Config failed to load; its pages and sections will not render"
            );
            None
        },
    }
}

fn parse_section<T: serde::de::DeserializeOwned>(
    config_name: &str,
    value: serde_yaml::Value,
) -> Result<T, ConfigError> {
    serde_yaml::from_value(value).map_err(|source| ConfigError::Parse {
        config_name: config_name.to_owned(),
        source,
    })
}

fn load_navigation_config(services: &Path) -> Result<Option<Arc<NavigationConfig>>, ConfigError> {
    let Some(value) = load_config_section(services, "navigation.yaml")? else {
        return Ok(None);
    };
    let config: NavigationConfig = parse_section("navigation.yaml", value)?;
    tracing::info!("Loaded navigation config from config/navigation.yaml");
    Ok(Some(Arc::new(config)))
}

fn load_homepage_config(services: &Path) -> Result<Option<Arc<HomepageConfig>>, ConfigError> {
    let Some(value) = load_config_section(services, "homepage.yaml")? else {
        return Ok(None);
    };
    let config: HomepageConfig = parse_section("homepage.yaml", value)?;
    tracing::info!("Loaded homepage config from config/homepage.yaml");
    Ok(Some(Arc::new(config)))
}

fn load_branding_config(services: &Path) -> Result<Option<BrandingConfig>, ConfigError> {
    let Some(theme) = load_config_section(services, "theme.yaml")? else {
        return Ok(None);
    };
    let Some(branding) = theme.get("branding") else {
        return Ok(None);
    };
    let config: BrandingConfig = parse_section("theme.yaml (branding section)", branding.clone())?;
    tracing::info!("Loaded branding config from config/theme.yaml");
    Ok(Some(config))
}

pub fn load_config_section(
    services: &Path,
    filename: &str,
) -> Result<Option<serde_yaml::Value>, ConfigError> {
    let config_path = services.join(format!("web/config/{filename}"));
    let yaml_content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(
                path = %config_path.display(),
                "Config file does not exist"
            );
            return Ok(None);
        },
        Err(source) => {
            return Err(ConfigError::Read {
                path: config_path,
                source,
            });
        },
    };
    serde_yaml::from_str(&yaml_content)
        .map(Some)
        .map_err(|source| ConfigError::Parse {
            config_name: filename.to_owned(),
            source,
        })
}
