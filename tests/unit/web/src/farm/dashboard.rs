//! The Farm dashboard, the sidebar and names, the tenant selector without its
//! button, and the spacing rule, against a fake farm-api.


use std::path::PathBuf;
use std::sync::Arc;

use super::common::{
    FakeSp, Mode, SpMode, TASK_OK, TOKEN, TestResult, dead_url, fake_farm, get_page, get_page_sp,
};
use axum::http::StatusCode;
use systemprompt_web_admin_agent_farm::client::FarmApi;

fn repo_file(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| format!("missing {relative}: {e}"))
}

// -------------------------------------------------------------- dashboard

#[tokio::test]
async fn dashboard_summarises_every_tenant_and_links_every_view() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    // Every block leads to its view; with all tenants shown the links carry
    // no tenant, and the Tenants tile leads to the Tenants page.
    for href in [
        "/admin/farm/tenants",
        "/admin/farm/tasks",
        "/admin/farm/agents",
        "/admin/farm/schedules",
        "/admin/farm/credentials",
    ] {
        assert!(page.has(&format!(r#"href="{href}""#)), "{href}");
    }
    assert!(page.has(r#"href="/admin/farm/cost?from="#));
    assert!(page.has("showing all"));
    assert!(page.has("Tenants"));
    assert!(page.has("Agents"));
    assert!(page.has("Schedules"));
    assert!(page.has("Credentials"));
    assert!(page.has("3 active"), "schedules note");
    assert!(page.has("succeeded"));
    assert!(page.has(r#"<option value="" selected>All tenants</option>"#));
    assert!(page.has(r#"<option value="lab-b">"#));
    // Only names and counts: nothing a credential could hold.
    assert!(!page.has("SECRET"));
    // farm-api was asked for every tenant at once.
    let seen = farm.seen();
    assert!(
        seen.iter()
            .filter(|r| !r.contains("/whoami"))
            .all(|r| r.contains("tenant=*")),
        "{seen:?}"
    );
    Ok(())
}

fn tile_values(html: &str) -> Vec<String> {
    html.split("sp-kpi__value\">")
        .skip(1)
        .map(|s| s.chars().take_while(|c| *c != '<').collect())
        .collect()
}

#[tokio::test]
async fn dashboard_counts_are_the_counts_of_each_list() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    // Tenants, Agents, Schedules, Credentials, Cost, then the tasks of the
    // period (total, then one tile per state).
    for (uri, expected) in [
        ("/farm", ["2", "4", "4", "10", "$6.00", "6", "6"]),
        (
            "/farm?tenant=dynapps-internal",
            ["2", "3", "3", "9", "$2.00", "5", "5"],
        ),
        (
            "/farm?tenant=lab-b",
            ["2", "1", "1", "1", "$4.00", "1", "1"],
        ),
    ] {
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(tile_values(&page.html), expected, "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn dashboard_links_keep_the_chosen_tenant() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm?tenant=lab-b", true).await?;

    for href in [
        "/admin/farm/tasks?tenant=lab-b",
        "/admin/farm/agents?tenant=lab-b",
        "/admin/farm/schedules?tenant=lab-b",
        "/admin/farm/credentials?tenant=lab-b",
        "/admin/farm/cost?tenant=lab-b&from=",
    ] {
        assert!(page.has(&format!(r#"href="{href}"#)), "{href}");
    }
    assert!(page.has(r#"<option value="lab-b" selected"#));
    assert!(page.has("showing lab-b"));
    Ok(())
}

#[tokio::test]
async fn dashboard_spends_a_bounded_number_of_farm_api_calls() -> TestResult {
    let farm = fake_farm(Mode::Endless).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    let seen = farm.seen();
    // whoami, agents, schedules, 2 credential scopes and at most 3 task pages.
    assert!(seen.len() <= 8, "{} calls: {seen:?}", seen.len());
    assert!(
        page.has("Partial"),
        "a period longer than the budget must say its numbers are partial"
    );
    Ok(())
}

#[tokio::test]
async fn dashboard_survives_farm_api_failures() -> TestResult {
    let dead = dead_url().await?;
    let page = get_page(FarmApi::new(&dead, TOKEN), "/farm", true).await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());

    let farm = fake_farm(Mode::Broken).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm", true).await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());

    let farm = fake_farm(Mode::NoPlatformScope).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm", true).await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    // Tasks that could not be read are a dash, not "0 tasks".
    assert!(page.has("tasks unavailable"));

    // farm-api before 16a: no `tenant=*`. A notice, and the page stands.
    let farm = fake_farm(Mode::NoWildcard).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm", true).await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("invalid_tenant"));
    Ok(())
}

#[tokio::test]
async fn dashboard_survives_systemprompt_failing() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page_sp(
        FarmApi::new(&farm.base, TOKEN),
        Arc::new(FakeSp::new(SpMode::Broken)),
        "/farm",
        true,
    )
    .await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    // The farm's own blocks are still there.
    assert!(page.has("Agents"));
    assert!(!page.has("$2.00"));
    Ok(())
}

#[tokio::test]
async fn dashboard_is_for_admins_only() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    let page = get_page_sp(
        FarmApi::new(&farm.base, TOKEN),
        Arc::clone(&sp),
        "/farm",
        false,
    )
    .await?;
    assert_eq!(page.status, StatusCode::FORBIDDEN);
    assert!(farm.seen().is_empty(), "farm-api was read for a non-admin");
    assert!(
        sp.seen().is_empty(),
        "systemprompt was read for a non-admin"
    );
    Ok(())
}

#[test]
fn the_sidebar_group_is_farm_and_opens_with_dashboard_then_tenants() {
    let sidebar = repo_file("storage/files/admin/partials/sidebar.hbs");
    let group = sidebar
        .split("data-nav-group=\"farm\"")
        .nth(1)
        .unwrap_or_default();
    let group = group.split("</section>").next().unwrap_or_default();
    assert!(group.contains(r#"<span class="sp-nav-group__title">Farm</span>"#));
    let hrefs: Vec<&str> = group
        .split("nav-item href=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .collect();
    assert_eq!(
        hrefs,
        [
            "/admin/farm",
            "/admin/farm/tenants",
            "/admin/farm/tasks",
            "/admin/farm/agents",
            "/admin/farm/schedules",
            "/admin/farm/credentials",
            "/admin/farm/cost",
        ]
    );
    assert!(
        !sidebar.contains("agent-farm"),
        "no old name in the sidebar"
    );
    assert!(
        !sidebar.contains("Agent Farm"),
        "no old name in the sidebar"
    );
}

#[tokio::test]
async fn no_page_calls_itself_agent_farm() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    for uri in [
        "/farm",
        "/farm/tenants",
        "/farm/tasks",
        &format!("/farm/tasks/{TASK_OK}?tenant=dynapps-internal"),
        "/farm/agents",
        "/farm/agents/release-notes?tenant=dynapps-internal",
        "/farm/schedules",
        "/farm/credentials",
        "/farm/cost",
    ] {
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(!page.html.contains("Agent Farm"), "{uri}");
        assert!(!page.html.contains("/admin/agent-farm"), "{uri}");
        assert!(!page.html.contains("agent-farm"), "{uri}");
        assert!(page.has("Farm"), "{uri}");
    }
    // The failure notice does not use the old name either.
    let dead = dead_url().await?;
    let page = get_page(FarmApi::new(&dead, TOKEN), "/farm", true).await?;
    assert!(page.has_fault_notice());
    assert!(!page.html.contains("Agent Farm"));
    Ok(())
}

#[test]
fn the_templates_and_script_carry_the_new_name() {
    let dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../storage/files/admin/templates");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| n.contains("farm"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    assert_eq!(
        names,
        [
            "farm-agent.hbs",
            "farm-agents.hbs",
            "farm-cost.hbs",
            "farm-credentials.hbs",
            "farm-schedules.hbs",
            "farm-task.hbs",
            "farm-tasks.hbs",
            "farm-tenants.hbs",
            "farm.hbs",
        ]
    );
}

// --------------------------------------------------------- tenant selector

#[tokio::test]
async fn tenant_selector_reloads_on_change_and_keeps_a_no_js_button() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    for uri in [
        "/farm",
        "/farm/tasks",
        "/farm/agents",
        "/farm/schedules",
        "/farm/credentials",
        "/farm/cost",
    ] {
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(
            page.has("data-autosubmit"),
            "{uri}: select must auto-submit"
        );
        assert!(
            page.has("/js/pages/admin-farm.js"),
            "{uri}: the page script is not loaded"
        );
        // The button is only for readers without JS.
        assert!(
            page.has("<noscript>"),
            "{uri}: the Show button needs a <noscript> fallback"
        );
        assert!(
            !page
                .html
                .contains("<div class=\"sp-toolbar__actions\"><button type=\"submit\""),
            "{uri}: the Show button must not be visible with JS"
        );
        // No inline handlers: the console's script policy forbids them.
        assert!(!page.html.contains("onchange="), "{uri}");
    }
    Ok(())
}

#[test]
fn the_page_script_submits_the_form_of_an_autosubmit_control() {
    let js = repo_file("storage/files/js/pages/admin-farm.js");
    assert!(js.contains("[data-autosubmit]"), "{js}");
    assert!(js.contains("'change'"), "{js}");
    assert!(js.contains("submit"), "{js}");
    let assets = repo_file("extensions/web/shared/admin-core/src/assets.rs");
    assert!(
        assets.contains(r#"page_js!(&pages, "admin-farm.js")"#),
        "the script must be in the admin asset manifest"
    );
}

// ----------------------------------------------------------------- spacing

#[test]
fn farm_pages_put_space_between_a_section_title_and_its_table() {
    let css = repo_file("storage/files/css/admin/04-section.css");
    assert!(
        css.contains(r#"[data-section^="farm"] .sp-section__title"#),
        "the farm pages' section titles need space below"
    );
    assert!(css.contains("margin-bottom: var(--sp-space-3)"));
}
