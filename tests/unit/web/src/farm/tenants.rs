//! The Tenants page against a fake farm-api that follows 16a's contract
//! (`tenant=*` is every tenant the credential can see).


use std::sync::Arc;

use axum::http::StatusCode;
use super::common::{FakeSp, Mode, SpMode, TOKEN, TestResult, dead_url, fake_farm, get_page, get_page_sp};
use systemprompt_web_admin_agent_farm::client::FarmApi;

// The cells of the table row that mentions `slug`.
fn row(html: &str, slug: &str) -> String {
    let marker = format!(r#"<code class="sp-code-inline">{slug}</code>"#);
    let at = html.find(&marker).unwrap_or(0);
    let start = html[..at].rfind("<tr>").unwrap_or(0);
    let end = html[at..].find("</tr>").map_or(html.len(), |e| at + e);
    html[start..end]
        .replace("&#x3D;", "=")
        .replace("&amp;", "&")
}

#[tokio::test]
async fn tenants_page_has_a_row_per_tenant_with_counts_states_and_cost() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/tenants", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("2 tenants"));
    // Name and slug.
    assert!(page.has("Dynapps internal"));
    assert!(page.has("Lab B"));

    // dynapps-internal: 3 agents, 3 schedules, 5 tasks in the period (all
    // succeeded), $2.00 in systemprompt.
    let internal = row(&page.html, "dynapps-internal");
    assert!(
        internal.contains("/admin/farm/agents?tenant=dynapps-internal\">3<"),
        "{internal}"
    );
    assert!(
        internal.contains("/admin/farm/schedules?tenant=dynapps-internal\">3<"),
        "{internal}"
    );
    assert!(
        internal.contains("/admin/farm/tasks?tenant=dynapps-internal\">5<"),
        "{internal}"
    );
    assert!(internal.contains("5 succeeded"), "{internal}");
    assert!(internal.contains("$2.00"), "{internal}");

    // lab-b: 1 agent, 1 schedule, 1 task, $4.00.
    let lab = row(&page.html, "lab-b");
    assert!(
        lab.contains("/admin/farm/agents?tenant=lab-b\">1<"),
        "{lab}"
    );
    assert!(
        lab.contains("/admin/farm/schedules?tenant=lab-b\">1<"),
        "{lab}"
    );
    assert!(lab.contains("/admin/farm/tasks?tenant=lab-b\">1<"), "{lab}");
    assert!(lab.contains("1 succeeded"), "{lab}");
    assert!(lab.contains("$4.00"), "{lab}");

    // Each row links to the other views filtered by its tenant.
    for tenant in ["dynapps-internal", "lab-b"] {
        let r = row(&page.html, tenant);
        for view in ["", "/tasks", "/agents", "/schedules", "/credentials"] {
            assert!(
                r.contains(&format!("href=\"/admin/farm{view}?tenant={tenant}\"")),
                "{tenant} {view}: {r}"
            );
        }
        assert!(
            r.contains(&format!("href=\"/admin/farm/cost?tenant={tenant}&from=")),
            "{tenant} cost: {r}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn the_farm_api_budget_does_not_grow_with_the_tenants() -> TestResult {
    let farm = fake_farm(Mode::Endless).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/tenants", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    let seen = farm.seen();
    // whoami, one agents list, one schedules list, at most 3 task pages.
    assert!(seen.len() <= 6, "{} calls: {seen:?}", seen.len());
    assert!(
        seen.iter()
            .filter(|r| !r.contains("/whoami"))
            .all(|r| r.contains("tenant=*")),
        "never one read per tenant: {seen:?}"
    );
    assert!(
        page.has("Partial"),
        "a period longer than the budget is partial"
    );
    Ok(())
}

#[tokio::test]
async fn systemprompt_failing_leaves_the_rows_with_a_dash_for_cost() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page_sp(
        FarmApi::new(&farm.base, TOKEN),
        Arc::new(FakeSp::new(SpMode::Broken)),
        "/farm/tenants",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    let internal = row(&page.html, "dynapps-internal");
    assert!(internal.contains("5 succeeded"), "{internal}");
    assert!(!internal.contains("$2.00"), "{internal}");
    Ok(())
}

#[tokio::test]
async fn tenants_page_survives_farm_api_failing() -> TestResult {
    let dead = dead_url().await?;
    let page = get_page(FarmApi::new(&dead, TOKEN), "/farm/tenants", true).await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("Tenants unavailable"));

    let farm = fake_farm(Mode::NoPlatformScope).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/tenants", true).await?;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    // The tenants are known from whoami: the rows stay, their numbers are dashes.
    assert!(page.has("Lab B"));
    let lab = row(&page.html, "lab-b");
    assert!(lab.contains("agents?tenant=lab-b\">—<"), "{lab}");
    assert!(lab.contains("schedules?tenant=lab-b\">—<"), "{lab}");
    assert!(lab.contains("tasks?tenant=lab-b\">—<"), "{lab}");
    Ok(())
}

#[tokio::test]
async fn tenants_page_is_for_admins_only() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    let page = get_page_sp(
        FarmApi::new(&farm.base, TOKEN),
        Arc::clone(&sp),
        "/farm/tenants",
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
