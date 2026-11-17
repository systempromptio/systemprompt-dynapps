//! The single task page's view models: the task, its result, its logs.

use crate::client::FarmApiError;
use crate::models::{TaskLogs, TaskOut, TaskResult};
use crate::usage::SpTaskView;
use crate::views::{
    KeyValue, NONE, agent_label, format_duration, format_time, list_href, opt_num, opt_text,
    short_id, state_tone,
};
use serde::Serialize;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

#[derive(Debug, Serialize)]
pub struct ResultView {
    pub summary: Option<String>,
    pub note: Option<String>,
    pub facts: Vec<KeyValue>,
}

#[derive(Debug, Serialize)]
pub struct LogsView {
    pub text: String,
    pub has_lines: bool,
    pub truncated: bool,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskDetailView {
    #[serde(rename = "task_id")]
    pub id: String,
    pub state: String,
    pub state_tone: &'static str,
    pub failure_reason: Option<String>,
    pub agent: String,
    pub owner: String,
    pub tenant: String,
    pub identity: Vec<KeyValue>,
    pub times: Vec<KeyValue>,
    pub limits: Vec<KeyValue>,
    pub usage: Vec<KeyValue>,
}

#[derive(Debug, Serialize)]
pub struct TaskPageContext {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub back_url: String,
    pub task: Option<TaskDetailView>,
    pub result: Option<ResultView>,
    pub logs: Option<LogsView>,
    pub sp: Option<SpTaskView>,
}

pub fn identity(task: &TaskOut) -> Vec<KeyValue> {
    vec![
        KeyValue {
            label: "Task",
            value: task.id.clone(),
        },
        KeyValue {
            label: "Scope",
            value: opt_text(task.scope.as_deref()),
        },
        KeyValue {
            label: "Origin",
            value: opt_text(task.origin.as_deref()),
        },
        KeyValue {
            label: "Session",
            value: opt_text(task.session_ref.as_deref()),
        },
        KeyValue {
            label: "Resumable",
            value: task.resumable.map_or_else(
                || NONE.to_owned(),
                |r| if r { "yes" } else { "no" }.to_owned(),
            ),
        },
        KeyValue {
            label: "Queue position",
            value: opt_num(task.queue_position),
        },
    ]
}

pub fn times(task: &TaskOut) -> Vec<KeyValue> {
    vec![
        KeyValue {
            label: "Created",
            value: format_time(task.created_at.as_deref()),
        },
        KeyValue {
            label: "Started",
            value: format_time(task.started_at.as_deref()),
        },
        KeyValue {
            label: "Finished",
            value: format_time(task.finished_at.as_deref()),
        },
        KeyValue {
            label: "Updated",
            value: format_time(task.updated_at.as_deref()),
        },
    ]
}

pub fn limits(task: &TaskOut) -> Vec<KeyValue> {
    let l = task.limits.clone().unwrap_or_default();
    vec![
        KeyValue {
            label: "CPU",
            value: opt_text(l.cpu.as_deref()),
        },
        KeyValue {
            label: "Memory",
            value: opt_text(l.memory.as_deref()),
        },
        KeyValue {
            label: "Timeout",
            value: format_duration(l.timeout_s.map(|v| v as f64)),
        },
        KeyValue {
            label: "TTL",
            value: format_duration(l.ttl_s.map(|v| v as f64)),
        },
    ]
}

pub fn usage(task: &TaskOut) -> Vec<KeyValue> {
    let u = task.usage.clone().unwrap_or_default();
    vec![
        KeyValue {
            label: "Input tokens",
            value: opt_num(u.input_tokens),
        },
        KeyValue {
            label: "Output tokens",
            value: opt_num(u.output_tokens),
        },
        KeyValue {
            label: "Duration",
            value: format_duration(u.duration_s),
        },
        KeyValue {
            label: "Gateway session",
            value: opt_text(u.gateway_session_id.as_deref()),
        },
    ]
}

pub fn task_detail(task: &TaskOut, tenant: &str) -> TaskDetailView {
    let (agent, _) = agent_label(task);
    TaskDetailView {
        id: task.id.clone(),
        state_tone: state_tone(&task.state),
        state: task.state.clone(),
        failure_reason: task.failure_reason.clone(),
        agent,
        owner: opt_text(task.owner.as_deref()),
        tenant: task.tenant.clone().unwrap_or_else(|| tenant.to_owned()),
        identity: identity(task),
        times: times(task),
        limits: limits(task),
        usage: usage(task),
    }
}

pub fn result_note(error: &FarmApiError) -> Option<&'static str> {
    match error.code() {
        Some("task_not_terminal") => {
            Some("The task has not finished yet: its result appears when it does.")
        },
        Some("task_has_no_result") => {
            Some("The task finished without producing a result. The failure reason above says why.")
        },
        Some("result_expired") => Some("The result was deleted by the retention policy."),
        _ => None,
    }
}

pub fn result_view(result: &TaskResult) -> ResultView {
    let object = result.result.clone().unwrap_or_default();
    ResultView {
        summary: result.summary.clone().filter(|s| !s.trim().is_empty()),
        note: None,
        facts: vec![
            KeyValue {
                label: "Size",
                value: object
                    .size_bytes
                    .map_or_else(|| NONE.to_owned(), |b| format!("{b} bytes")),
            },
            KeyValue {
                label: "SHA-256",
                value: opt_text(object.sha256.as_deref()),
            },
        ],
    }
}

pub fn result_note_view(note: &str) -> ResultView {
    ResultView {
        summary: None,
        note: Some(note.to_owned()),
        facts: Vec::new(),
    }
}

pub fn logs_view(logs: &TaskLogs) -> LogsView {
    LogsView {
        has_lines: !logs.lines.is_empty(),
        text: logs.lines.join("\n"),
        truncated: logs.truncated,
        note: logs.lines.is_empty().then(|| {
            "No log lines: the pod has not started yet, or the task left no preview.".to_owned()
        }),
    }
}

#[derive(Debug)]
pub struct TaskInput<'a> {
    pub task_id: &'a str,
    pub tenant: &'a str,
    pub task: Option<&'a TaskOut>,
    pub result: Option<ResultView>,
    pub logs: Option<LogsView>,
    pub sp: Option<SpTaskView>,
}

pub fn task_page(input: TaskInput<'_>) -> TaskPageContext {
    let short = short_id(input.task_id);
    let back_url = list_href(input.tenant, None);
    TaskPageContext {
        page: "farm-task",
        title: format!("Task {short}"),
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::link("Tasks", back_url.clone()),
            BreadcrumbView::current(format!("Task {short}")),
        ],
        back_url,
        task: input.task.map(|t| task_detail(t, input.tenant)),
        result: input.result,
        logs: input.logs,
        sp: input.sp,
    }
}
