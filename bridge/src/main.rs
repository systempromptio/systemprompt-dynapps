#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! Astound Digital desktop bridge.
//!
//! A thin white-label wrapper over the systemprompt bridge: it defines the
//! Astound [`Brand`] (chrome, on-disk paths, env prefix, default gateway, and
//! embedded GUI assets) and hands it to
//! [`systemprompt_bridge::run_with_brand`]. All behaviour lives in the shared
//! core library — this file is intentionally tiny so a new client bridge is
//! "copy this crate, swap `assets/`, edit the const below". See `README.md` for
//! the recipe. The brand ships a single dark palette with no light theme, so
//! `force_dark` keeps a light desktop from drawing a light title bar over it.

use std::process::ExitCode;

use systemprompt_bridge::brand::{Brand, BrandAssets};

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod registry;

// Why: core's stock `/v1/bridge/whoami` answers only id, email, name and roles;
// this envelope adds the ADFS issuer, AD groups, department and seat, and core
// forwards the unknown keys verbatim.
systemprompt_bridge::register_identity_source!("/api/public/bridge/whoami");

static ASTOUND_BRAND: Brand = Brand {
    app_name: "Astound Bridge",
    binary_name: "astound-bridge",
    version: env!("CARGO_PKG_VERSION"),
    vendor: "Astound Digital",
    config_dir: "astound",
    config_file: "astound-bridge.toml",
    pat_file: "astound-bridge.pat",
    working_dir_name: "astound-bridge",
    workspace_dir_name: "Astound",
    keyring_service: "astound-bridge.oauth-client",
    env_prefix: "ASTOUND_BRIDGE",
    default_gateway_url: "https://sp-dev.astound.digital",
    // Why: the gateway mounts this page under /bridge-auth, not core's default /bridge.
    device_link_path: "/bridge-auth/device-link",
    tray_tooltip: "Astound Bridge",
    window_title: "Astound Bridge",
    app_menu_name: "Astound Bridge",
    sign_in_label: "Sign in",
    sign_in_hint: "Opens your browser — sign in with Astound SSO. This device is linked \
                   automatically once you approve.",
    docs_url: "https://astound.systemprompt.io/documentation",
    contact_email: "ed@systemprompt.io",
    pitch_head: "Govern every coding agent.",
    pitch_body: "One gateway. Every agent. Every tool call audited.",
    schedule_label: "com.astounddigital.bridge-sync",
    schedule_unit: "astound-bridge-sync",
    schedule_task_name: "AstoundBridgeSync",
    autostart_label: "com.astounddigital.bridge-gui",
    autostart_task_name: "AstoundBridgeGui",
    aumid: "com.astounddigital.bridge",
    force_dark: true,
    // Why: cargo/sccache only re-embed a regenerated asset read from OUT_DIR,
    // where build.rs copies them.
    assets: BrandAssets {
        icon_svg: include_str!(concat!(env!("OUT_DIR"), "/icon.svg")),
        logo_svg: include_str!(concat!(env!("OUT_DIR"), "/logo.svg")),
        window_icon_png: include_bytes!(concat!(env!("OUT_DIR"), "/window-icon-1024.png")),
        tray_icon_png: include_bytes!(concat!(env!("OUT_DIR"), "/tray-icon.png")),
        app_icon_ico: include_bytes!(concat!(env!("OUT_DIR"), "/app-icon.ico")),
        theme_css: include_str!(concat!(env!("OUT_DIR"), "/theme.css")),
    },
};

fn main() -> ExitCode {
    systemprompt_bridge::run_with_brand(&ASTOUND_BRAND)
}
