//! One tool call end to end, after core has authenticated the caller:
//! resolve who they are, gate the guarded tools on their live role, resolve
//! their Odoo credential, run the handler, then audit the outcome and leave
//! the attribution note. Every exit, allowed or refused, lands an audit row.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock};
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::mcp::{ClientProfile, McpToolExecutor};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt_mcp_shared::record_mcp_access;

use super::gate::{authorise, post_attribution, refuse, settle};
use super::tool::{Dispatch, build_call, dispatch_tool};
use crate::audit::CallAudit;
use crate::caller::load_caller;
use crate::client::OdooClient;
use crate::credential::CredentialSource;
use crate::error::OdooError;

#[derive(Debug)]
pub(super) struct Serve<'a> {
    pub(super) db_pool: &'a DbPool,
    pub(super) executor: &'a McpToolExecutor,
    pub(super) client: &'a Arc<OdooClient>,
    pub(super) source: &'a CredentialSource,
    pub(super) request: &'a CallToolRequestParams,
    pub(super) request_context: &'a SysRequestContext,
    pub(super) profile: &'a ClientProfile,
    pub(super) server_name: &'a str,
}

// Why: MCP bridges reject a JSON-RPC error before rendering, so credential and
// setup states return as isError results.
const fn surfaces_as_result(err: &OdooError) -> bool {
    matches!(
        err,
        OdooError::NotLinked(_)
            | OdooError::NotConfigured(_)
            | OdooError::AppMissing(_)
            | OdooError::AccessDenied(_)
    )
}

pub(super) async fn serve(s: &Serve<'_>) -> Result<CallToolResult, McpError> {
    let tool = s.request.name.as_ref();
    let pool: Arc<PgPool> = s.db_pool.pool();
    let caller = match load_caller(
        &pool,
        s.request_context.user_id(),
        s.request_context.caller_purpose(),
    )
    .await
    {
        Ok(caller) => caller,
        Err(err) => {
            return Ok(CallToolResult::error(vec![ContentBlock::text(
                err.to_string(),
            )]));
        },
    };
    let audit = CallAudit {
        caller: &caller,
        context: s.request_context,
        tool,
        arguments: s.request.arguments.as_ref(),
    };
    if let Err(refused) = authorise(&pool, audit).await {
        return Ok(refused);
    }

    let mut call = match build_call(&pool, s.source, s.client, s.request_context).await {
        Ok(call) => call,
        Err(err) if surfaces_as_result(&err) => {
            return Ok(refuse(&pool, audit, &err.to_string()).await);
        },
        Err(other) => return Err(other.into()),
    };
    call.creds.footer = Some(caller.footer());

    record_mcp_access(
        s.db_pool,
        s.request_context.user_id(),
        s.server_name,
        tool,
        "used",
    )
    .await;

    let dispatch = Dispatch {
        executor: s.executor,
        request: s.request,
        request_context: s.request_context,
        client: s.profile,
    };
    let result = Box::pin(dispatch_tool(&dispatch, call.clone(), tool)).await;
    let failure = failure_detail(&result);
    let succeeded = failure.is_none();
    settle(&pool, audit, failure.as_deref()).await;
    if succeeded {
        post_attribution(&call, tool, s.request.arguments.as_ref()).await;
    }
    result
}

fn failure_detail(result: &Result<CallToolResult, McpError>) -> Option<String> {
    match result {
        Err(error) => Some(error.message.to_string()),
        Ok(r) if r.is_error == Some(true) => Some(
            r.content
                .iter()
                .filter_map(|block| block.as_text().map(|t| t.text.clone()))
                .collect::<Vec<_>>()
                .join(" "),
        ),
        Ok(_) => None,
    }
}
