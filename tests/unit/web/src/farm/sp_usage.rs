//! What systemprompt recorded for the farm's tasks (model calls, cost in USD,
//! conversation), on the task, task list and cost pages, against a fake
//! farm-api and a fake systemprompt usage source.


use std::sync::Arc;

use axum::http::StatusCode;
use super::common::{
    CLIENT_TASK_OK, CONTEXT_TASK_OK, FakeSp, GW_TASK_2, GW_TASK_OK, Mode, SpMode, TASK_FAILED,
    TASK_OK, TOKEN, TestResult, dead_url, fake_farm, get_page, get_page_sp,
};
use systemprompt_web_admin_agent_farm::client::FarmApi;

#[tokio::test]
async fn task_page_lists_sp_calls_totals_and_the_conversation() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    let uri = format!("/farm/tasks/{TASK_OK}?tenant=dynapps-internal");
    let page = get_page_sp(FarmApi::new(&farm.base, TOKEN), Arc::clone(&sp), &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    // The calls: time, model, status, tokens (cache included), cost, latency.
    assert!(page.has("Model calls"), "calls section");
    assert!(page.has("2026-10-26 08:26:26 UTC"));
    assert!(page.has("claude-haiku-5-5"));
    assert!(page.has("claude-sonnet-5"));
    assert!(page.has("failed"));
    assert!(page.has("12000"), "cache-read tokens");
    assert!(page.has("3000"), "cache-creation tokens");
    // Cost in USD, as systemprompt stores it: 2374 µ$ = $0.0024.
    assert!(page.has("$0.0024"));
    assert!(page.has("2 calls"), "the task's total");
    // Totals by model.
    assert!(page.has("Totals by model"));
    // The conversation is a link to the existing reader, not rendered here.
    assert!(page.has(&format!("/admin/contexts/{CONTEXT_TASK_OK}")));
    // farm-api's cost_eur is not shown any more.
    assert!(!page.has("€"));

    let seen = sp.seen();
    assert!(seen.contains(&format!("calls {GW_TASK_OK}")), "{seen:?}");
    assert!(
        seen.contains(&format!("totals_by_model {GW_TASK_OK}")),
        "{seen:?}"
    );
    assert!(
        seen.contains(&format!("contexts {CLIENT_TASK_OK}")),
        "the conversation is found by session_ref without sess_: {seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn task_without_a_gateway_session_says_so() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    let uri = format!("/farm/tasks/{TASK_FAILED}?tenant=dynapps-internal");
    let page = get_page_sp(FarmApi::new(&farm.base, TOKEN), Arc::clone(&sp), &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(
        !page.has_fault_notice(),
        "a missing link is a note, not a fault"
    );
    assert!(page.has("no systemprompt gateway session"));
    let seen = sp.seen();
    assert!(
        seen.iter().all(|s| !s.starts_with("calls")),
        "no session, no query: {seen:?}"
    );
    Ok(())
}

#[tokio::test]
async fn task_with_no_sp_rows_says_so() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Empty));
    let uri = format!("/farm/tasks/{TASK_OK}?tenant=dynapps-internal");
    let page = get_page_sp(FarmApi::new(&farm.base, TOKEN), sp, &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "no rows is a note, not a fault");
    assert!(page.has("has no model calls for gateway session"));
    assert!(page.has(GW_TASK_OK));
    assert!(page.has("No conversation"));
    assert!(!page.has("/admin/contexts/"));
    Ok(())
}

#[tokio::test]
async fn systemprompt_down_is_a_notice_not_a_500() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    for uri in [
        format!("/farm/tasks/{TASK_OK}?tenant=dynapps-internal"),
        "/farm/tasks".to_owned(),
        "/farm/cost?tenant=dynapps-internal&from=2026-10-20&to=2026-10-27".to_owned(),
    ] {
        let sp = Arc::new(FakeSp::new(SpMode::Broken));
        let page = get_page_sp(FarmApi::new(&farm.base, TOKEN), sp, &uri, true).await?;
        assert_eq!(page.status, StatusCode::OK, "{uri}");
        assert!(page.has_fault_notice(), "{uri}");
        assert!(page.has("systemprompt usage could not be read"), "{uri}");
    }
    Ok(())
}

#[tokio::test]
async fn task_list_shows_sp_cost_in_usd_from_one_query() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    let page = get_page_sp(
        FarmApi::new(&farm.base, TOKEN),
        Arc::clone(&sp),
        "/farm/tasks",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("$0.0024"), "TASK_OK: 2374 µ$");
    assert!(page.has("$0.0046"), "01a17c32: 4557 µ$");
    assert!(!page.has("€"));
    let totals: Vec<String> = sp
        .seen()
        .into_iter()
        .filter(|s| s.starts_with("session_totals"))
        .collect();
    assert_eq!(totals.len(), 1, "one query for the whole page: {totals:?}");
    assert!(totals[0].contains(GW_TASK_OK) && totals[0].contains(GW_TASK_2));
    // The rows still come from farm-api.
    assert!(
        farm.seen()
            .iter()
            .any(|r| r.starts_with("/farm/api/v1/tasks?")),
        "{:?}",
        farm.seen()
    );
    Ok(())
}

#[tokio::test]
async fn cost_page_reads_systemprompt_in_one_query_and_counts_unlinked_tasks() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    let page = get_page_sp(
        FarmApi::new(&farm.base, TOKEN),
        Arc::clone(&sp),
        "/farm/cost?tenant=dynapps-internal&from=2026-10-20&to=2026-10-27",
        true,
    )
    .await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(!page.has_fault_notice(), "unexpected fault notice");
    assert!(page.has("1 without SP link"));
    assert!(!page.has("€"), "farm-api's cost_eur is never used");
    let totals: Vec<String> = sp
        .seen()
        .into_iter()
        .filter(|s| s.starts_with("session_totals"))
        .collect();
    assert_eq!(totals.len(), 1, "one query for the range: {totals:?}");
    for n in ["0001", "0002", "0004", "0005"] {
        assert!(
            totals[0].contains(&format!("8000-00000000{n}")),
            "{totals:?}"
        );
    }
    assert!(!totals[0].contains("8000-000000000003"), "{totals:?}");
    Ok(())
}

#[tokio::test]
async fn task_page_shows_latency_for_humans() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let uri = format!("/farm/tasks/{TASK_OK}?tenant=dynapps-internal");
    let page = get_page(FarmApi::new(&farm.base, TOKEN), &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has("8.2 s"), "latency over a second");
    assert!(page.has("450 ms"), "latency under a second");
    Ok(())
}

#[tokio::test]
async fn farm_api_down_means_systemprompt_is_not_asked() -> TestResult {
    let base = dead_url().await?;
    let sp = Arc::new(FakeSp::new(SpMode::Normal));
    let uri = format!("/farm/tasks/{TASK_OK}");
    let page = get_page_sp(FarmApi::new(&base, TOKEN), Arc::clone(&sp), &uri, true).await?;

    assert_eq!(page.status, StatusCode::OK);
    assert!(page.has_fault_notice());
    assert!(
        sp.seen().is_empty(),
        "no task, no session to look up: {:?}",
        sp.seen()
    );
    Ok(())
}
