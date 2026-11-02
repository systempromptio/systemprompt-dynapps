//! The instance's name, colour and logo for the consent page, read once from
//! `web/config/theme.yaml` under the services root.
//!
//! Anything that would not be safe to put into the page as it stands is
//! dropped for the default: the colour must be a hex colour and the logo a
//! same-origin path of plain characters. A missing or unreadable file gives
//! the defaults.

use std::path::Path;

use serde::Deserialize;

const DEFAULT_NAME: &str = "systemprompt";
const DEFAULT_COLOR: &str = "#1f2937";
const MAX_NAME_CHARS: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branding {
    pub display_name: String,
    pub theme_color: String,
    pub logo: Option<String>,
}

impl Default for Branding {
    fn default() -> Self {
        Self {
            display_name: DEFAULT_NAME.to_owned(),
            theme_color: DEFAULT_COLOR.to_owned(),
            logo: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    branding: Option<ThemeBranding>,
}

#[derive(Debug, Deserialize)]
struct ThemeBranding {
    display_name: Option<String>,
    theme_color: Option<String>,
    logo_light: Option<String>,
}

fn hex_color(value: &str) -> bool {
    value.strip_prefix('#').is_some_and(|hex| {
        matches!(hex.len(), 3 | 6) && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn same_origin_path(value: &str) -> bool {
    value.starts_with('/')
        && !value.starts_with("//")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-'))
}

impl Branding {
    pub fn from_theme_yaml(text: &str) -> Self {
        let defaults = Self::default();
        let Some(theme) = serde_yaml::from_str::<ThemeFile>(text)
            .inspect_err(|error| tracing::warn!(%error, "dynwork mobile: theme.yaml unreadable"))
            .ok()
            .and_then(|file| file.branding)
        else {
            return defaults;
        };
        let display_name = theme
            .display_name
            .map(|name| name.trim().to_owned())
            .filter(|name| {
                !name.is_empty()
                    && name.chars().count() <= MAX_NAME_CHARS
                    && !name.chars().any(char::is_control)
            })
            .unwrap_or(defaults.display_name);
        Self {
            display_name,
            theme_color: theme
                .theme_color
                .filter(|color| hex_color(color))
                .unwrap_or(defaults.theme_color),
            logo: theme.logo_light.filter(|logo| same_origin_path(logo)),
        }
    }

    pub fn load(services_root: &Path) -> Self {
        let path = services_root.join("web/config/theme.yaml");
        std::fs::read_to_string(&path).map_or_else(
            |error| {
                tracing::warn!(%error, path = %path.display(), "dynwork mobile: no theme.yaml");
                Self::default()
            },
            |text| Self::from_theme_yaml(&text),
        )
    }
}
