//! Per-call logic: resolve the platform caller's Odoo credential, then route
//! to a handler.
//!
//! The two authentications are distinct and both required. The MCP transport
//! proves *which platform user* is calling (core's RBAC, run in
//! `systemprompt_mcp_shared::auth`); that user's sealed `odoo_api_key`
//! machine credential turns it into an Odoo login, and Odoo then decides what
//! that login may do. The only policy this server adds of its own is the
//! guardrail on destructive and erasure tools in [`crate::guard`].

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::mcp::{ClientProfile, McpToolExecutor, McpToolHandler};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

use super::call::OdooCall;
use super::{
    activity, attachments, calendar, channels, crm, discovery, guarded, notes, overview, partner,
    partner_write, report, sales, tasks,
};
use crate::client::OdooClient;
use crate::credential::{CredentialSource, Lookup, resolve_credentials};
use crate::error::OdooError;
use crate::tools::{
    ALL_TOOLS, TOOL_ACTIVITY_COMPLETE, TOOL_ACTIVITY_CREATE, TOOL_ACTIVITY_LIST,
    TOOL_ACTIVITY_TYPE_LIST, TOOL_ATTACHMENT_ADD, TOOL_ATTACHMENT_GET, TOOL_ATTACHMENT_LIST,
    TOOL_CALENDAR_EVENT_CREATE, TOOL_CALENDAR_EVENT_LIST, TOOL_CHANNEL_LIST, TOOL_CHANNEL_POST,
    TOOL_GDPR_ERASE_INVENTORY, TOOL_INVOICE_GET, TOOL_INVOICE_LIST, TOOL_LEAD_CONVERT,
    TOOL_LEAD_CREATE, TOOL_LEAD_DELETE, TOOL_LEAD_GET, TOOL_LEAD_MARK_LOST, TOOL_LEAD_MARK_WON,
    TOOL_LEAD_REPORT, TOOL_LEAD_SEARCH, TOOL_LEAD_UPDATE, TOOL_NOTE_ADD, TOOL_NOTE_LIST,
    TOOL_NOTE_SEARCH, TOOL_OVERVIEW, TOOL_PARTNER_ANONYMISE, TOOL_PARTNER_CREATE,
    TOOL_PARTNER_DELETE, TOOL_PARTNER_GET, TOOL_PARTNER_SEARCH, TOOL_PARTNER_UPDATE,
    TOOL_SALE_ORDER_CREATE, TOOL_SALE_ORDER_GET, TOOL_SALE_ORDER_LIST, TOOL_STAGE_LIST,
    TOOL_TASK_CREATE, TOOL_TASK_LIST, TOOL_TASK_UPDATE, TOOL_USER_LIST,
};

pub async fn build_call(
    pool: &PgPool,
    source: &CredentialSource,
    client: &Arc<OdooClient>,
    request_context: &SysRequestContext,
) -> Result<OdooCall, OdooError> {
    let lookup = Lookup {
        pool,
        source,
        user_id: request_context.user_id(),
        caller_purpose: request_context.caller_purpose(),
    };
    let creds = resolve_credentials(&lookup, client).await?;
    Ok(OdooCall {
        client: Arc::clone(client),
        creds,
    })
}

#[doc(hidden)]
pub async fn dispatch_tool(
    ctx: &Dispatch<'_>,
    call: OdooCall,
    tool_name: &str,
) -> Result<CallToolResult, McpError> {
    // Why: split by plane rather than one 40-arm match. Every arm names a
    // distinct handler type, and holding all of them in one stack frame costs
    // half a megabyte — over clippy's frame ceiling, and a real cost on a
    // server that runs one of these per request.
    if let Some(result) = Box::pin(crm_tools(ctx, call.clone(), tool_name)).await {
        return result;
    }
    if let Some(result) = Box::pin(closing_tools(ctx, call.clone(), tool_name)).await {
        return result;
    }
    if let Some(result) = Box::pin(sales_tools(ctx, call.clone(), tool_name)).await {
        return result;
    }
    if let Some(result) = Box::pin(knowledge_tools(ctx, call.clone(), tool_name)).await {
        return result;
    }
    if let Some(result) = Box::pin(work_tools(ctx, call.clone(), tool_name)).await {
        return result;
    }
    if let Some(result) = Box::pin(guarded_tools(ctx, call, tool_name)).await {
        return result;
    }
    Err(unknown_tool(tool_name))
}

#[doc(hidden)]
#[derive(Debug)]
pub struct Dispatch<'a> {
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub request_context: &'a SysRequestContext,
    pub client: &'a ClientProfile,
}

async fn crm_tools(
    ctx: &Dispatch<'_>,
    call: OdooCall,
    tool_name: &str,
) -> Option<Result<CallToolResult, McpError>> {
    Some(match tool_name {
        TOOL_LEAD_SEARCH => ctx.run(&crm::LeadSearchHandler { call }).await,
        TOOL_LEAD_GET => ctx.run(&crm::LeadGetHandler { call }).await,
        TOOL_LEAD_CREATE => ctx.run(&crm::LeadCreateHandler { call }).await,
        TOOL_LEAD_UPDATE => ctx.run(&crm::LeadUpdateHandler { call }).await,
        TOOL_LEAD_REPORT => ctx.run(&report::LeadReportHandler { call }).await,
        TOOL_PARTNER_SEARCH => ctx.run(&partner::PartnerSearchHandler { call }).await,
        TOOL_PARTNER_GET => ctx.run(&partner::PartnerGetHandler { call }).await,
        TOOL_OVERVIEW => Box::pin(ctx.run(&overview::OverviewHandler { call })).await,
        _ => return None,
    })
}

// Why: Closing a deal, writing the customer, and the id lookups.
//
// A plane of its own for the same reason the others are: each handler is a
// distinct type and they cannot all share one stack frame.
async fn closing_tools(
    ctx: &Dispatch<'_>,
    call: OdooCall,
    tool_name: &str,
) -> Option<Result<CallToolResult, McpError>> {
    Some(match tool_name {
        TOOL_LEAD_MARK_WON => ctx.run(&crm::LeadMarkWonHandler { call }).await,
        TOOL_LEAD_MARK_LOST => ctx.run(&crm::LeadMarkLostHandler { call }).await,
        TOOL_LEAD_CONVERT => ctx.run(&crm::LeadConvertHandler { call }).await,
        TOOL_PARTNER_CREATE => ctx.run(&partner_write::PartnerCreateHandler { call }).await,
        TOOL_PARTNER_UPDATE => ctx.run(&partner_write::PartnerUpdateHandler { call }).await,
        TOOL_STAGE_LIST => ctx.run(&discovery::StageListHandler { call }).await,
        TOOL_USER_LIST => ctx.run(&discovery::UserListHandler { call }).await,
        TOOL_ACTIVITY_TYPE_LIST => ctx.run(&discovery::ActivityTypeListHandler { call }).await,
        _ => return None,
    })
}

async fn sales_tools(
    ctx: &Dispatch<'_>,
    call: OdooCall,
    tool_name: &str,
) -> Option<Result<CallToolResult, McpError>> {
    Some(match tool_name {
        TOOL_SALE_ORDER_LIST => ctx.run(&sales::SaleOrderListHandler { call }).await,
        TOOL_SALE_ORDER_GET => ctx.run(&sales::SaleOrderGetHandler { call }).await,
        TOOL_SALE_ORDER_CREATE => ctx.run(&sales::SaleOrderCreateHandler { call }).await,
        TOOL_INVOICE_LIST => ctx.run(&sales::InvoiceListHandler { call }).await,
        TOOL_INVOICE_GET => ctx.run(&sales::InvoiceGetHandler { call }).await,
        _ => return None,
    })
}

async fn knowledge_tools(
    ctx: &Dispatch<'_>,
    call: OdooCall,
    tool_name: &str,
) -> Option<Result<CallToolResult, McpError>> {
    Some(match tool_name {
        TOOL_NOTE_ADD => ctx.run(&notes::NoteAddHandler { call }).await,
        TOOL_NOTE_LIST => ctx.run(&notes::NoteListHandler { call }).await,
        TOOL_NOTE_SEARCH => ctx.run(&notes::NoteSearchHandler { call }).await,
        TOOL_ATTACHMENT_ADD => ctx.run(&attachments::AttachmentAddHandler { call }).await,
        TOOL_ATTACHMENT_LIST => ctx.run(&attachments::AttachmentListHandler { call }).await,
        TOOL_ATTACHMENT_GET => ctx.run(&attachments::AttachmentGetHandler { call }).await,
        _ => return None,
    })
}

async fn work_tools(
    ctx: &Dispatch<'_>,
    call: OdooCall,
    tool_name: &str,
) -> Option<Result<CallToolResult, McpError>> {
    Some(match tool_name {
        TOOL_ACTIVITY_LIST => ctx.run(&activity::ActivityListHandler { call }).await,
        TOOL_ACTIVITY_CREATE => ctx.run(&activity::ActivityCreateHandler { call }).await,
        TOOL_ACTIVITY_COMPLETE => ctx.run(&activity::ActivityCompleteHandler { call }).await,
        TOOL_CALENDAR_EVENT_LIST => ctx.run(&calendar::CalendarEventListHandler { call }).await,
        TOOL_CALENDAR_EVENT_CREATE => {
            ctx.run(&calendar::CalendarEventCreateHandler { call })
                .await
        },
        TOOL_TASK_LIST => ctx.run(&tasks::TaskListHandler { call }).await,
        TOOL_TASK_CREATE => ctx.run(&tasks::TaskCreateHandler { call }).await,
        TOOL_TASK_UPDATE => ctx.run(&tasks::TaskUpdateHandler { call }).await,
        TOOL_CHANNEL_LIST => ctx.run(&channels::ChannelListHandler { call }).await,
        TOOL_CHANNEL_POST => ctx.run(&channels::ChannelPostHandler { call }).await,
        _ => return None,
    })
}

async fn guarded_tools(
    ctx: &Dispatch<'_>,
    call: OdooCall,
    tool_name: &str,
) -> Option<Result<CallToolResult, McpError>> {
    Some(match tool_name {
        TOOL_LEAD_DELETE => ctx.run(&guarded::LeadDeleteHandler { call }).await,
        TOOL_PARTNER_DELETE => ctx.run(&guarded::PartnerDeleteHandler { call }).await,
        TOOL_PARTNER_ANONYMISE => {
            ctx.run(&guarded::erasure::PartnerAnonymiseHandler { call })
                .await
        },
        TOOL_GDPR_ERASE_INVENTORY => {
            ctx.run(&guarded::erasure::GdprEraseInventoryHandler { call })
                .await
        },
        _ => return None,
    })
}

impl Dispatch<'_> {
    async fn run<H: McpToolHandler>(&self, handler: &H) -> Result<CallToolResult, McpError> {
        self.executor
            .execute(handler, self.request, self.request_context, self.client)
            .await
    }
}

#[doc(hidden)]
#[must_use]
pub fn unknown_tool(tool_name: &str) -> McpError {
    McpError::invalid_params(
        format!(
            "Unknown tool: '{tool_name}'. Available tools: {}.",
            ALL_TOOLS.join(", ")
        ),
        None,
    )
}
