//! `request_log`: AI requests newest first, filtered by user, paged by cursor.

use super::{PagedOutput, RequestLogInput, flag_value, page_limit, read_rows};
use crate::cli::CliLocation;
use rmcp::ErrorData;
// JSON: rows are the CLI's own row objects.
use serde_json::Value;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

#[derive(Debug, Clone, Copy)]
pub struct RequestLogHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

pub fn command(input: &RequestLogInput) -> Result<String, ErrorData> {
    let mut command = format!(
        "infra logs request list --since {} --limit {}",
        flag_value("since", &input.since)?,
        page_limit(input.limit, 50)
    );
    for (flag, value) in [
        ("until", &input.until),
        ("user", &input.user),
        ("model", &input.model),
        ("provider", &input.provider),
        ("before", &input.cursor),
    ] {
        let value = flag_value(flag, value)?;
        if !value.is_empty() {
            command.push_str(&format!(" --{flag} {value}"));
        }
    }
    Ok(command)
}

// Why: the last row's own cursor is the next page's `--before`; a short page
// has no next page, so it yields none.
pub(crate) fn next_cursor(rows: &[Value], page_full: bool) -> String {
    if !page_full {
        return String::new();
    }
    rows.last()
        .and_then(|row| row.get("cursor"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

impl McpToolHandler for RequestLogHandler<'_> {
    type Input = RequestLogInput;
    type Output = PagedOutput;
    fn tool_name(&self) -> &'static str {
        "request_log"
    }
    fn description(&self) -> &'static str {
        "List AI gateway requests newest first: request_id, timestamp, user_id, model, tokens (`input+cachedc/output(reasoningr)`), cost, latency and status. Filter by `user`, `model`, `provider` and a `since`/`until` window; page backwards by passing the returned `next_cursor` as `cursor`. Pass a row's `request_id` to `conversation_audit` to read what was said."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: RequestLogInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(PagedOutput, String), ErrorData> {
        let command = command(&input)?;
        let rows = read_rows(self.cli, self.token, &command).await?;
        let page_full = rows.len() >= usize::from(page_limit(input.limit, 50));
        let cursor = next_cursor(&rows, page_full);
        let mut output = PagedOutput::new(command, rows);
        output.truncated = input.limit > super::MAX_PAGE;
        output.hint = if cursor.is_empty() {
            "Last page for these filters.".to_owned()
        } else {
            "Older rows exist: call again with `cursor` set to `next_cursor`.".to_owned()
        };
        output.next_cursor = cursor;
        let summary = output.summary();
        Ok((output, summary))
    }
}
