//! `add_knowledge_document`: add one hand-authored document to a files
//! category and index it immediately. Open to every signed-in user — the
//! document is granted to the category's configured subjects, `role:user` by
//! default, which is exactly the audience that may call this tool.
//! Replacing a document that already exists is an administrator's act.

use super::format::format_upload_report;
use super::tool::{require_admin, to_mcp_error};
use crate::ingest::{Ingestor, UploadRequest};
use crate::principal::Principal;
use crate::tools::{ADD_DOCUMENT_TOOL, AddKnowledgeDocumentInput};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

pub(super) struct AddDocumentHandler<'a> {
    pub(super) ingestor: &'a Arc<Ingestor>,
}

impl McpToolHandler for AddDocumentHandler<'_> {
    type Input = AddKnowledgeDocumentInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        ADD_DOCUMENT_TOOL
    }

    fn description(&self) -> &'static str {
        "Add a document to a files category and index it now."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        if input.overwrite {
            require_admin(&principal, "overwriting with add_knowledge_document")
                .map_err(|e| to_mcp_error(&e))?;
        }
        let request = UploadRequest::try_from(input).map_err(|e| to_mcp_error(&e))?;
        self.ingestor
            .add_document(&principal, request)
            .await
            .map(|report| format_upload_report(&report))
            .map_err(|e| to_mcp_error(&e))
    }
}
