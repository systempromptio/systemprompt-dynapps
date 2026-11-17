//! The Agents, Schedules, Credentials and Cost pages against a fake farm-api.


use std::sync::Arc;

use super::common::{
    FakeSp, Mode, SpMode, TOKEN, TestResult, dead_url, fake_farm, get_page, get_page_sp,
};
use axum::http::StatusCode;
use systemprompt_web_admin_agent_farm::client::FarmApi;

const SECRET: &str = "SECRET-VALUE-MUST-NOT-SHOW";
const FINGERPRINT: &str = "FINGERPRINT-MUST-NOT-SHOW";

fn decoded(seen: &[String]) -> Vec<String> {
    seen.iter().map(|r| r.replace("%3A", ":")).collect()
}

// ---------------------------------------------------------------- agents

#[tokio::test]
async fn agents_page_lists_every_tenants_catalogue() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/agents", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    for slug in ["code-reviewer", "release-notes", "my-helper"] {
        assert!(page.has(slug), "{slug}");
    }
    assert!(page.has("platform"));
    assert!(page.has("team"));
    assert!(page.has("personal"));
    assert!(page.has("developer-skills:code-reviewer"));
    assert!(page.has("haiku"));
    // The other tenant's agent is in the same list, with a tenant column and
    // a detail link that names the row's own tenant.
    assert!(page.has("lab-helper"));
    assert!(page.has("<td>lab-b</td>"));
    assert!(page.has("<td>dynapps-internal</td>"));
    assert!(page.has("/admin/farm/agents/release-notes?tenant=dynapps-internal"));
    assert!(page.has("/admin/farm/agents/lab-helper?tenant=lab-b"));
    assert!(page.has(r#"<option value="" selected>All tenants</option>"#));
    let seen = farm.seen();
    assert!(
        seen.iter()
            .any(|r| r.starts_with("/farm/api/v1/agents?") && r.contains("tenant=*")),
        "{seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn agents_page_filters_to_one_tenant() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/agents?tenant=lab-b",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("lab-helper"));
    assert!(!page.has("code-reviewer"));
    assert!(page.has(r#"<option value="lab-b" selected"#));
    let seen = farm.seen();
    assert!(
        seen.iter()
            .any(|r| r.starts_with("/farm/api/v1/agents?") && r.contains("tenant=lab-b")),
        "{seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn an_agent_row_leads_to_a_detail_read_of_its_own_tenant() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/agents/lab-helper?tenant=lab-b",
        true,
    )
    .await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("Lab B instructions."));
    let seen = farm.seen();
    assert!(
        seen.iter().all(|r| !r.contains("tenant=*")),
        "a detail read must name one tenant: {seen:?}"
    );
    // Without a tenant in the URL the detail falls back to the home tenant,
    // never to `*`.
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/agents/release-notes",
        true,
    )
    .await?;
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(
        farm.seen()
            .iter()
            .any(|r| r.contains("/agents/release-notes?") && r.contains("tenant=dynapps-internal")),
        "{:?}",
        farm.seen()
    );
    Ok(())
}

#[tokio::test]
async fn agent_detail_shows_configuration_without_secrets() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/agents/release-notes?tenant=dynapps-internal",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("Write release notes in English, grouped by module."));
    assert!(page.has("github.com"));
    assert!(page.has("sonnet"));
    // Credentials appear by name only.
    assert!(page.has("GITLAB_TOKEN"));
    assert!(page.has("shared:team-gitlab"));
    assert!(page.has("mcp-odoo-dynapps"));
    assert!(page.has("4Gi"));
    assert!(page.has("10 min 0 s"));
    Ok(())
}

#[tokio::test]
async fn a_personal_agents_instructions_stay_private() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/agents/my-helper?tenant=dynapps-internal",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("my-helper"));
    assert!(!page.has("PERSONAL-INSTRUCTIONS-MUST-NOT-SHOW"));
    Ok(())
}

#[tokio::test]
async fn agent_slug_that_is_not_a_slug_never_reaches_farm_api() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/agents/..%2Fcredentials",
        true,
    )
    .await?;

    assert_ne!(page.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        farm.seen().iter().all(|r| !r.contains("credentials")),
        "{:?}",
        farm.seen()
    );
    Ok(())
}

#[tokio::test]
async fn unknown_agent_is_a_notice() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/agents/nope", true).await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("not_found"));
    Ok(())
}

// ------------------------------------------------------------- schedules

#[tokio::test]
async fn schedules_page_shows_cron_next_fire_and_state() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/schedules", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("0 6 * * 1-5"));
    assert!(page.has("Europe/Madrid"));
    assert!(page.has("2026-10-28 04:00:00 UTC"));
    assert!(page.has("active"));
    assert!(page.has("paused"));
    assert!(page.has("skipped_quota"));
    assert!(page.has("tenant capacity was full"));
    assert!(page.has("credential_not_found: mcp-odoo-dynapps"));
    assert!(page.has("code-reviewer"));
    assert!(page.has("person.two"));
    // The other tenant's schedule is in the same list, with its tenant.
    assert!(page.has("15 3 * * *"));
    assert!(page.has("person.three"));
    assert!(page.has("<td>lab-b</td>"));
    assert!(page.has("<td>dynapps-internal</td>"));
    assert!(page.has(r#"<option value="" selected>All tenants</option>"#));
    // A paused schedule has no next firing.
    assert!(page.has("—"));
    // The stored prompt is customer data: never on the page.
    assert!(!page.has("SCHEDULE-PROMPT-MUST-NOT-SHOW"));
    let seen = farm.seen();
    assert!(
        seen.iter()
            .any(|r| r.starts_with("/farm/api/v1/schedules?") && r.contains("tenant=*")),
        "{seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn schedules_page_filters_to_one_tenant() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/schedules?tenant=lab-b",
        true,
    )
    .await?;
    assert!(page.has("15 3 * * *"));
    assert!(!page.has("0 6 * * 1-5"));
    assert!(
        farm.seen()
            .iter()
            .any(|r| r.starts_with("/farm/api/v1/schedules?") && r.contains("tenant=lab-b")),
        "{:?}",
        farm.seen()
    );
    Ok(())
}

// ----------------------------------------------------------- credentials

#[tokio::test]
async fn credentials_page_shows_metadata_only() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/credentials", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    for name in [
        "api-coverage-test",
        "deepwiki",
        "mcp-github-readonly",
        "team-gitlab",
    ] {
        assert!(page.has(name), "{name}");
    }
    assert!(page.has("git_token"));
    assert!(page.has("mcp_url"));
    assert!(page.has("shared"));
    assert!(page.has("personal"));
    assert!(page.has("Deepwiki MCP endpoint"));
    assert!(page.has("2027-01-15 00:00:00 UTC"));
    assert!(page.has("dynapps-internal"));
    // The other tenant's credential (metadata only) is in the same list.
    assert!(page.has("lab-b-token"));
    assert!(page.has("<td>lab-b</td>"));
    assert!(page.has(r#"<option value="" selected>All tenants</option>"#));
    // Personal and shared are both read, for every tenant.
    let seen = farm.seen();
    assert!(
        seen.iter()
            .any(|r| r.starts_with("/farm/api/v1/credentials?") && r.contains("scope=shared")),
        "{seen:?}"
    );
    assert!(
        seen.iter()
            .any(|r| r.starts_with("/farm/api/v1/credentials?") && r.contains("scope=personal")),
        "{seen:?}"
    );
    assert!(
        seen.iter()
            .filter(|r| r.contains("/credentials?"))
            .all(|r| r.contains("tenant=*")),
        "{seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn credentials_page_filters_to_one_tenant() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/credentials?tenant=lab-b",
        true,
    )
    .await?;
    assert!(page.has("lab-b-token"));
    assert!(!page.has("team-gitlab"));
    assert!(!page.has(FINGERPRINT));
    Ok(())
}

// The fixture is hostile on purpose: farm-api never sends a `value`, but if
// it ever did (or sent the fingerprint or metadata), the page must not paint
// it, and must hold no input to type one into.
#[tokio::test]
async fn no_credentials_template_renders_a_value_field() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/credentials", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has(SECRET));
    assert!(!page.has(FINGERPRINT));
    assert!(!page.has("example.invalid"), "metadata is not shown");
    // The shell around the page has its own search box; the page's own section
    // is what must hold no input.
    let start = page
        .html
        .find(r#"aria-label="Credentials""#)
        .ok_or("credentials section not found")?;
    let end = page.html[start..]
        .find("</section>")
        .ok_or("credentials section not closed")?;
    let section = page.html[start..start + end].to_lowercase();
    assert!(
        !section.contains("<input"),
        "a credentials page has no input"
    );
    let html = page.html.to_lowercase();
    assert!(!html.contains("<textarea"));
    assert!(!html.contains("type=\"password\""));
    assert!(!html.contains("name=\"value\""));
    assert!(!html.contains("name=\"secret\""));
    assert!(!html.contains(">value<") && !html.contains(">secret<"));
    assert!(!html.contains("fingerprint"));

    // The same holds for the template source itself.
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../storage/files/admin/templates");
    let mut checked = 0;
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        if !name.starts_with("farm-credential") {
            continue;
        }
        let source = std::fs::read_to_string(&path)?.to_lowercase();
        for banned in [
            "<input",
            "<textarea",
            "password",
            "{{value",
            "{{secret",
            "{{fingerprint",
            "{{metadata",
        ] {
            assert!(!source.contains(banned), "{name} contains {banned}");
        }
        checked += 1;
    }
    assert!(checked >= 1, "no credentials template found");
    Ok(())
}

// ------------------------------------------------------------------ cost

#[tokio::test]
async fn cost_page_sums_sp_usage_by_agent_tenant_and_person() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/cost?tenant=dynapps-internal&from=2026-10-20&to=2026-10-27",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    // systemprompt's cost of the linked tasks, in USD: 0.50 + 0.25 + 1.25 +
    // 0.0025 over two farm-api pages (task 0003 has no gateway session).
    assert!(page.has("$2.00"), "total");
    assert!(page.has("5 tasks"));
    assert!(page.has("1 without SP link"));
    // By agent: code-reviewer 0.7525, free prompt 1.25.
    assert!(page.has("code-reviewer"));
    assert!(page.has("$0.75"));
    assert!(page.has("free prompt"));
    assert!(page.has("$1.25"));
    // By person: person.one 1.75, person.two 0.25, unknown 0.0025.
    assert!(page.has("person.one"));
    assert!(page.has("$1.75"));
    assert!(page.has("person.two"));
    assert!(page.has("$0.25"));
    assert!(page.has("unknown"));
    assert!(page.has("$0.0025"));
    // The chosen range is echoed.
    assert!(page.has("2026-10-20"));
    assert!(page.has("2026-10-27"));
    assert!(!page.has("Partial"), "all pages were read");

    // The range reached farm-api as [from 00:00, to+1 00:00), 100 per page,
    // following the cursor.
    let seen = decoded(&farm.seen());
    let dated: Vec<&String> = seen.iter().filter(|r| r.contains("since=")).collect();
    assert_eq!(dated.len(), 2, "{seen:?}");
    assert!(dated[0].contains("tenant=dynapps-internal"), "{dated:?}");
    assert!(dated[0].contains("since=2026-10-20T00:00:00Z"), "{dated:?}");
    assert!(dated[0].contains("until=2026-10-28T00:00:00Z"), "{dated:?}");
    assert!(dated[0].contains("limit=100"), "{dated:?}");
    assert!(dated[1].contains("cursor=c2"), "{dated:?}");
    Ok(())
}

#[tokio::test]
async fn cost_page_covers_every_tenant_by_default_in_one_stream() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/cost?from=2026-10-20&to=2026-10-27",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    // 2.0025 + 4.00
    assert!(page.has("$6.00"), "total");
    assert!(page.has("6 tasks"));
    assert!(page.has("lab-b"));
    assert!(page.has("$4.00"));
    assert!(page.has("release-notes"));
    assert!(page.has("person.three"));
    assert!(page.has(r#"<option value="" selected>All tenants</option>"#));
    // One cursor-chained stream with `tenant=*`: three pages, never a read per
    // tenant.
    let dated: Vec<String> = decoded(&farm.seen())
        .into_iter()
        .filter(|r| r.contains("since="))
        .collect();
    assert_eq!(dated.len(), 3, "{dated:?}");
    assert!(dated.iter().all(|r| r.contains("tenant=*")), "{dated:?}");
    assert!(dated[1].contains("cursor=c2"), "{dated:?}");
    assert!(dated[2].contains("cursor=c3"), "{dated:?}");
    Ok(())
}

#[tokio::test]
async fn cost_page_still_reads_the_old_all_value_as_all() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/cost?tenant=all&from=2026-10-20&to=2026-10-27",
        true,
    )
    .await?;
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("$6.00"), "total");
    Ok(())
}

#[tokio::test]
async fn cost_page_defaults_to_the_last_seven_days() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/cost", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    let seen = decoded(&farm.seen());
    assert!(
        seen.iter()
            .any(|r| r.contains("since=") && r.contains("T00:00:00Z") && r.contains("until=")),
        "{seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn cost_page_flags_a_partial_total_when_the_range_is_too_long() -> TestResult {
    let farm = fake_farm(Mode::Endless).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/cost?tenant=dynapps-internal&from=2026-01-01&to=2026-10-27",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("Partial"), "the total must say it is partial");
    let dated = farm.seen().iter().filter(|r| r.contains("since=")).count();
    assert_eq!(dated, 20, "the page budget bounds the farm-api calls");
    Ok(())
}

#[tokio::test]
async fn cost_page_ignores_a_bad_date_and_says_so() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/cost?from=yesterday&to=2026-10-27",
        true,
    )
    .await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("not a date"));
    Ok(())
}

// ------------------------------------------------------------ all pages

const PAGES: [&str; 6] = [
    "/farm/tenants",
    "/farm/agents",
    "/farm/agents/release-notes",
    "/farm/schedules",
    "/farm/credentials",
    "/farm/cost",
];

#[tokio::test]
async fn unset_credential_shows_a_notice() -> TestResult {
    for uri in PAGES {
        let page = get_page(FarmApi::disabled(), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(page.has("farm.yaml"), "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn farm_api_down_shows_a_notice() -> TestResult {
    let base = dead_url().await?;
    for uri in PAGES {
        let page = get_page(FarmApi::new(&base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(page.has("did not answer"), "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn farm_api_500_shows_a_notice() -> TestResult {
    let farm = fake_farm(Mode::Broken).await?;
    for uri in PAGES {
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(page.has("500"), "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn missing_scope_shows_a_notice() -> TestResult {
    let farm = fake_farm(Mode::NoPlatformScope).await?;
    for uri in PAGES {
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(page.has("insufficient_scope"), "{uri}");
    }
    Ok(())
}

// farm-api before 16a answers 422 to `tenant=*`: every list page says so in
// the usual notice, with a 200, and does not read tenant by tenant instead.
#[tokio::test]
async fn a_farm_api_without_the_star_tenant_gives_a_notice_on_every_list_page() -> TestResult {
    for uri in [
        "/farm/tenants",
        "/farm/agents",
        "/farm/schedules",
        "/farm/credentials",
        "/farm/cost",
    ] {
        let farm = fake_farm(Mode::NoWildcard).await?;
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(page.has("invalid_tenant"), "{uri}");
        let seen = farm.seen();
        assert!(
            seen.iter()
                .filter(|r| !r.contains("/whoami"))
                .all(|r| r.contains("tenant=*")),
            "{uri}: no fallback to per-tenant reads: {seen:?}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn rejected_token_never_appears() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    for uri in PAGES {
        let page = get_page(FarmApi::new(&farm.base, "wrong-token"), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(!page.has("wrong-token"), "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn non_admins_are_refused_and_farm_api_is_never_called() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    for uri in PAGES {
        let page =
            get_page_sp(FarmApi::new(&farm.base, TOKEN), Arc::clone(&sp), uri, false).await?;
        assert_eq!(page.status, StatusCode::FORBIDDEN, "{uri}");
        assert!(!page.has("person.one"), "{uri}");
    }
    assert!(farm.seen().is_empty(), "{:?}", farm.seen());
    assert!(
        sp.seen().is_empty(),
        "systemprompt never read: {:?}",
        sp.seen()
    );
    Ok(())
}
