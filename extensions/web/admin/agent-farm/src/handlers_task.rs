//! The single task page: the task, its result and logs from farm-api, and its
//! calls and conversation from systemprompt.

use crate::client::{FarmApi, FarmApiError};
use crate::handlers::{SP_FAULT, TenantQuery, identify, pick_tenant, require_console};
use crate::models::TaskOut;
use crate::sp::{SharedSp, SpError};
use crate::usage::{self, CallsRead, ContextsRead, SpTaskView};
use crate::views::{self, TaskInput};
use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::Response;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;

pub async fn task_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Extension(sp): Extension<SharedSp>,
    Path(task_id): Path<String>,
    Query(query): Query<TenantQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let mut faults = Faults::new();
    let who = identify(&api, &mut faults).await;
    let tenant = who.as_ref().map_or_else(
        || query.tenant().unwrap_or_default().to_owned(),
        |w| pick_tenant(&query, w),
    );
    let (task, result, logs) = if who.is_some() {
        read_task(&api, &tenant, &task_id, &mut faults).await
    } else {
        (None, None, None)
    };
    let sp_view = match task.as_ref() {
        Some(task) => Some(read_sp(&sp, task, &mut faults).await),
        None => None,
    };
    let ctx = views::task_page(TaskInput {
        task_id: &task_id,
        tenant: &tenant,
        task: task.as_ref(),
        result,
        logs,
        sp: sp_view,
    });
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-task",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

pub async fn read_sp(sp: &SharedSp, task: &TaskOut, faults: &mut Faults) -> SpTaskView {
    let gateway = usage::gateway_session_id(
        task.usage
            .as_ref()
            .and_then(|u| u.gateway_session_id.as_deref()),
    );
    let client = usage::client_session_id(task.session_ref.as_deref());
    let calls = async {
        let Some(id) = gateway.as_deref() else {
            return Ok(None);
        };
        let (calls, totals) = tokio::join!(sp.calls(id), sp.totals_by_model(id));
        Ok::<_, SpError>(Some((calls?, totals?)))
    };
    let contexts = async {
        match client.as_deref() {
            Some(id) => sp.contexts(id).await.map(Some),
            None => Ok(None),
        }
    };
    let (calls, contexts) = tokio::join!(calls, contexts);
    let calls = match calls {
        Ok(None) => CallsRead::NoSession,
        Ok(Some((calls, totals))) => CallsRead::Read { calls, totals },
        Err(e) => {
            faults.record(SP_FAULT, &e);
            CallsRead::Failed
        },
    };
    let contexts = match (contexts, client) {
        (Ok(Some(contexts)), Some(client_session_id)) => ContextsRead::Read {
            client_session_id,
            contexts,
        },
        (Ok(_), _) => ContextsRead::NoSession,
        (Err(e), _) => {
            if !matches!(calls, CallsRead::Failed) {
                faults.record(SP_FAULT, &e);
            }
            ContextsRead::Failed
        },
    };
    usage::sp_task_view(gateway.as_deref(), calls, contexts)
}

pub type TaskRead = (
    Option<TaskOut>,
    Option<views::ResultView>,
    Option<views::LogsView>,
);

pub async fn read_task(api: &FarmApi, tenant: &str, id: &str, faults: &mut Faults) -> TaskRead {
    let task = match api.get_task(tenant, id).await {
        Ok(task) => task,
        Err(e) => {
            faults.record("Task", &e);
            return (None, None, None);
        },
    };
    let (result, logs) = tokio::join!(api.get_result(tenant, id), api.get_logs(tenant, id));
    let result = match result {
        Ok(r) => Some(views::result_view(&r)),
        Err(e) => result_or_fault(&e, faults),
    };
    let logs = match logs {
        Ok(l) => Some(views::logs_view(&l)),
        Err(e) => {
            faults.record("Logs", &e);
            None
        },
    };
    (Some(task), result, logs)
}

pub fn result_or_fault(error: &FarmApiError, faults: &mut Faults) -> Option<views::ResultView> {
    if let Some(note) = views::result_note(error) {
        return Some(views::result_note_view(note));
    }
    faults.record("Result", error);
    None
}
