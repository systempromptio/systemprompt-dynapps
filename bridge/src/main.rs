#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! Dynapps desktop bridge.
//!
//! A thin white-label wrapper over the systemprompt bridge: it defines the
//! Dynapps [`Brand`] (chrome, on-disk paths, env prefix, default gateway, and
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

static DYNAPPS_BRAND: Brand = Brand {
    app_name: "Dynapps Bridge",
    binary_name: "dynapps-bridge",
    version: env!("CARGO_PKG_VERSION"),
    vendor: "Dynapps",
    config_dir: "dynapps",
    config_file: "dynapps-bridge.toml",
    pat_file: "dynapps-bridge.pat",
    working_dir_name: "dynapps-bridge",
    workspace_dir_name: "Dynapps",
    keyring_service: "dynapps-bridge.oauth-client",
    env_prefix: "DYNAPPS_BRIDGE",
    default_gateway_url: "https://dynapps.systemprompt.io",
    // Why: the gateway mounts this page under /bridge-auth, not core's default /bridge.
    device_link_path: "/bridge-auth/device-link",
    tray_tooltip: "Dynapps Bridge",
    window_title: "Dynapps Bridge",
    app_menu_name: "Dynapps Bridge",
    sign_in_label: "Sign in",
    sign_in_hint: "Opens your browser — sign in with Microsoft Entra ID. This device is linked \
                   automatically once you approve.",
    docs_url: "https://dynapps.systemprompt.io/documentation",
    contact_email: "ai@dynapps.es",
    pitch_head: "Dynapps AI, governed.",
    pitch_body: "Claude and Odoo tooling for Dynapps teams, with every tool call audited.",
    demo_notice: None,
    schedule_label: "com.dynapps.bridge-sync",
    schedule_unit: "dynapps-bridge-sync",
    schedule_task_name: "DynappsBridgeSync",
    autostart_label: "com.dynapps.bridge-gui",
    autostart_task_name: "DynappsBridgeGui",
    aumid: "com.dynapps.bridge",
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
    systemprompt_bridge::run_with_brand(&DYNAPPS_BRAND)
}
