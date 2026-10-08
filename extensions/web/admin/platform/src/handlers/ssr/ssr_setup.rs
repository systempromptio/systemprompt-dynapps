//! The four setup phases the Connect page draws, and the redirects that
//! fold the two older onboarding pages into it.
//!
//! `/admin/setup` and `/bridge-auth/setup` each used to be a third way of
//! telling someone how to connect, with their own step markup. Both now land
//! on `/admin/connect`; the phase list — whose completion is derived from
//! what actually reaches the account, plugin and skill counts, not from a
//! checkbox — is what this module still owns.

use axum::extract::Query;
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::types::MarketplaceContext;

pub(crate) const CONNECT_URL: &str = "/admin/connect";

#[derive(Debug, Serialize)]
// Why: Handlebars — the layout partial's `title=` hash shadows a `title` field
// inside every block, hence `phase_title`.
pub(super) struct SetupPhase {
    pub number: u8,
    pub phase_title: String,
    pub description: &'static str,
    pub guide_url: &'static str,
    pub action_url: &'static str,
    pub action_label: &'static str,
    pub complete: bool,
    pub current: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
}

#[derive(Deserialize, Debug)]
pub(crate) struct SetupQuery {
    #[serde(default)]
    verified: Option<String>,
}

pub(super) fn setup_phases(mkt_ctx: &MarketplaceContext) -> Vec<SetupPhase> {
    let phase1_complete = mkt_ctx.total_plugins > 0;
    let phase2_complete = phase1_complete;
    let phase3_complete = phase2_complete && mkt_ctx.total_skills > 0;
    let mut phases = vec![
        SetupPhase {
            number: 1,
            phase_title: format!("Connect Claude to {}", mkt_ctx.site_url),
            description: "The essential first step. Connect your Claude surface so skills, plugins, and analytics actually work. Without this, nothing else matters.",
            guide_url: "/documentation/connect-claude-code",
            action_url: "",
            action_label: "",
            complete: phase1_complete,
            current: !phase1_complete,
            status_label: "",
            status_tone: "",
        },
        SetupPhase {
            number: 2,
            phase_title: String::from("Browse and Fork Plugins"),
            description: "Explore the plugin catalogue. Fork industry-specific plugins to build your personalised skill library with proven defaults.",
            guide_url: "/documentation/enterprise-tool-governance",
            action_url: "/admin/plugins",
            action_label: "Browse Plugins",
            complete: phase2_complete,
            current: phase1_complete && !phase2_complete,
            status_label: "",
            status_tone: "",
        },
        SetupPhase {
            number: 3,
            phase_title: String::from("Customize Your Skills"),
            description: "Use the Skill Manager MCP server to edit forked skills, create new ones, and build a library that matches how your team works.",
            guide_url: "/documentation/skills-lifecycle",
            action_url: "/admin/skills",
            action_label: "My Skills",
            complete: phase3_complete,
            current: phase2_complete && !phase3_complete,
            status_label: "",
            status_tone: "",
        },
        SetupPhase {
            number: 4,
            phase_title: String::from("Monitor, Report, and Improve"),
            description: "Track skill effectiveness with the CLI. Identify what is working, retire what is not, and iterate your way to a world-class skill library.",
            guide_url: "/documentation/dashboard",
            action_url: "/admin/users",
            action_label: "Open Admin",
            complete: false,
            current: phase3_complete,
            status_label: "",
            status_tone: "",
        },
    ];
    for phase in &mut phases {
        let (label, tone) = match (phase.complete, phase.current) {
            (true, _) => ("Complete", "ok"),
            (false, true) => ("Current", "accent"),
            (false, false) => ("Not started", "muted"),
        };
        phase.status_label = label;
        phase.status_tone = tone;
    }
    phases
}

pub(crate) async fn setup_page(Query(query): Query<SetupQuery>) -> AdminHtmlResult<Response> {
    if query.verified.is_some() {
        return Ok(redirect_with_flash(
            CONNECT_URL,
            FlashTone::Ok,
            "Connected: this machine is signed in to the gateway. Phase 1 is done.",
        )
        .into_response());
    }
    Ok(Redirect::permanent(CONNECT_URL).into_response())
}

pub(crate) async fn bridge_setup_page() -> Redirect {
    Redirect::permanent(CONNECT_URL)
}
