//! The Tasks list and Task detail pages against a fake farm-api.


use axum::http::StatusCode;
use std::sync::Arc;

use super::common::{
    FakeSp, Mode, SpMode, TASK_FAILED, TASK_OK, TOKEN, TestResult, dead_url, fake_farm, get_page,
    get_page_sp,
};
use systemprompt_web_admin_agent_farm::client::FarmApi;

#[tokio::test]
async fn tasks_page_lists_every_tenant_mixed_by_default() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/tasks", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    // Rows from the recorded page: ids, agent, owner, state.
    assert!(page.has("01a17c32"));
    assert!(page.has(TASK_OK));
    assert!(page.has("developer-skills:code-reviewer"));
    assert!(page.has("alpha-test-person3"));
    assert!(page.has("runtime_error"));
    // ... and the other tenant's task, in the same list.
    assert!(page.has("01c00000"));
    assert!(page.has("lab-helper"));
    // Each row links to its detail page with the row's own tenant.
    assert!(page.has(&format!(
        "/admin/farm/tasks/{TASK_OK}?tenant=dynapps-internal"
    )));
    assert!(page.has("/admin/farm/tasks/01c00000-0001-7000-8000-00000000000b?tenant=lab-b"));
    // "All tenants" is the default and sits first; both tenants follow.
    assert!(page.has(r#"<option value="" selected>All tenants</option>"#));
    assert!(page.has(r#"<option value="lab-b">"#));
    assert!(page.has(r#"<option value="dynapps-internal">"#));
    // Cursor pagination from `next_cursor`, with no tenant in the URL.
    assert!(page.has(r#"href="/admin/farm/tasks?cursor=159""#));
    // farm-api was asked for every tenant in one stream, never tenant by tenant.
    let seen = farm.seen();
    assert!(
        seen.iter()
            .any(|r| r.starts_with("/farm/api/v1/tasks?") && r.contains("tenant=*")),
        "{seen:?}"
    );
    assert!(
        seen.iter()
            .all(|r| !r.contains("tenant=lab-b") && !r.contains("tenant=dynapps-internal")),
        "{seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn tasks_page_filters_to_the_chosen_tenant_and_keeps_the_cursor() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/tasks?tenant=lab-b&cursor=159",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has(r#"<option value="lab-b" selected"#));
    assert!(page.has("01c00000"));
    assert!(!page.has(TASK_OK), "another tenant's task is filtered out");
    let seen = farm.seen();
    assert!(
        seen.iter().any(|r| r.starts_with("/farm/api/v1/tasks?")
            && r.contains("tenant=lab-b")
            && r.contains("cursor=159")),
        "{seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn a_blank_or_star_tenant_is_all_tenants() -> TestResult {
    for uri in [
        "/farm/tasks?tenant=",
        "/farm/tasks?tenant=%2A",
        "/farm/tasks?tenant=all",
    ] {
        let farm = fake_farm(Mode::Normal).await?;
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(
            page.has(r#"<option value="" selected>All tenants</option>"#),
            "{uri}"
        );
        assert!(page.has(TASK_OK) && page.has("01c00000"), "{uri}");
        assert!(
            farm.seen()
                .iter()
                .any(|r| r.contains("/tasks?") && r.contains("tenant=*")),
            "{uri}: {:?}",
            farm.seen()
        );
    }
    Ok(())
}

#[tokio::test]
async fn a_task_without_a_tenant_in_the_url_is_read_from_the_home_tenant() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let uri = format!("/farm/tasks/{TASK_OK}");
    let page = get_page(FarmApi::new(&farm.base, TOKEN), &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    let seen = farm.seen();
    assert!(
        seen.iter().all(|r| !r.contains("tenant=*")),
        "a detail read must name one tenant: {seen:?}"
    );
    assert!(
        seen.iter()
            .any(|r| r.contains(&format!("/tasks/{TASK_OK}?"))
                && r.contains("tenant=dynapps-internal")),
        "{seen:?}"
    );
    // The way back goes to the list of that tenant.
    assert!(page.has("/admin/farm/tasks?tenant=dynapps-internal"));
    Ok(())
}

#[tokio::test]
async fn task_page_shows_state_limits_usage_and_result() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let uri = format!("/farm/tasks/{TASK_OK}?tenant=dynapps-internal");
    let page = get_page(FarmApi::new(&farm.base, TOKEN), &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has(TASK_OK));
    assert!(page.has("succeeded"));
    assert!(page.has("developer-skills:code-reviewer"));
    assert!(page.has("1Gi"));
    assert!(page.has("10 min 0 s"));
    assert!(page.has("4315"));
    assert!(page.has("22 s"));
    assert!(page.has("2026-10-26 08:26:18 UTC"));
    assert!(page.has("5802 bytes"));
    // The presigned download link is a credential: never on the page.
    assert!(!page.has("objects.invalid"));
    let seen = farm.seen();
    for suffix in ["", "/result", "/logs"] {
        let path = format!("/farm/api/v1/tasks/{TASK_OK}{suffix}?");
        assert!(
            seen.iter()
                .any(|r| r.starts_with(&path) && r.contains("tenant=dynapps-internal")),
            "{path} not requested: {seen:?}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn failed_task_shows_reason_logs_and_no_result_note() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let uri = format!("/farm/tasks/{TASK_FAILED}?tenant=dynapps-internal");
    let page = get_page(FarmApi::new(&farm.base, TOKEN), &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    // 409 task_has_no_result is expected, not a fault.
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("runtime_error"));
    assert!(page.has("finished without producing a result"));
    assert!(page.has("[run-agent] ERROR: no model to request"));
    Ok(())
}

#[tokio::test]
async fn unknown_task_is_a_notice_not_a_500() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let uri = "/farm/tasks/00000000-0000-7000-8000-000000000000";
    let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("not_found"));
    Ok(())
}

#[tokio::test]
async fn a_task_id_that_is_not_a_uuid_never_reaches_farm_api() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/tasks/..%2Fcredentials",
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
async fn unset_credential_shows_a_notice() -> TestResult {
    for uri in ["/farm/tasks", &format!("/farm/tasks/{TASK_OK}")] {
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
    for uri in ["/farm/tasks", &format!("/farm/tasks/{TASK_OK}")] {
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
    for uri in ["/farm/tasks", &format!("/farm/tasks/{TASK_OK}")] {
        let page = get_page(FarmApi::new(&farm.base, TOKEN), uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(page.has("500"), "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn rejected_token_shows_a_notice_without_the_token() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let page = get_page(FarmApi::new(&farm.base, "wrong-token"), "/farm/tasks", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("401"));
    assert!(page.has("token_invalid"));
    assert!(!page.has("wrong-token"));
    Ok(())
}

#[tokio::test]
async fn missing_scope_shows_a_notice() -> TestResult {
    let farm = fake_farm(Mode::NoPlatformScope).await?;
    let page = get_page(
        FarmApi::new(&farm.base, TOKEN),
        "/farm/tasks?tenant=lab-b",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("insufficient_scope"));
    Ok(())
}

#[tokio::test]
async fn missing_scope_on_the_default_all_view_shows_the_same_notice() -> TestResult {
    let farm = fake_farm(Mode::NoPlatformScope).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/tasks", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("insufficient_scope"));
    Ok(())
}

// farm-api before 16a does not know `tenant=*`. The console does not paper
// over that by merging tenants itself: it shows the notice, with a 200.
#[tokio::test]
async fn a_farm_api_without_the_star_tenant_gives_a_notice_not_a_500_or_a_merge() -> TestResult {
    let farm = fake_farm(Mode::NoWildcard).await?;
    let page = get_page(FarmApi::new(&farm.base, TOKEN), "/farm/tasks", true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(page.has("invalid_tenant"));
    assert!(!page.has(TASK_OK));
    let seen = farm.seen();
    let list_reads: Vec<&String> = seen.iter().filter(|r| r.contains("/tasks?")).collect();
    assert_eq!(list_reads.len(), 1, "no fallback reads: {seen:?}");
    assert!(list_reads[0].contains("tenant=*"), "{seen:?}");
    Ok(())
}

#[tokio::test]
async fn non_admins_are_refused() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    for uri in ["/farm/tasks", &format!("/farm/tasks/{TASK_OK}")] {
        let page =
            get_page_sp(FarmApi::new(&farm.base, TOKEN), Arc::clone(&sp), uri, false).await?;
        assert_eq!(page.status, StatusCode::FORBIDDEN, "{uri}");
        assert!(!page.has(TASK_OK), "{uri}");
    }
    assert!(farm.seen().is_empty(), "{:?}", farm.seen());
    assert!(
        sp.seen().is_empty(),
        "systemprompt never read: {:?}",
        sp.seen()
    );
    Ok(())
}
