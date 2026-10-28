//! The shared document store end to end, against the installation running.
//!
//! One throwaway database carries the full schema. The runtime boots on a
//! profile over it with the document store on, the API serves from that
//! runtime on a loopback socket, and the `systemprompt-mcp-documents`
//! binary runs as its own process on the same profile. The agent's steps
//! are MCP tool calls to that binary (`flow`, `review`, `lifecycle`), the
//! people's steps are REST calls to the API (`access`), and the document
//! service is read only to check what each step left behind. A refusal is
//! checked in the shape each surface gives it: a refused tool result whose
//! structured content carries the `code`, and the HTTP error envelope whose
//! `error_key` is that code. The run also holds the documents tools to their
//! size budget: `tools/list` as serialised, and a tool result as the model
//! receives it, without the `_meta` execution envelope. A missing database
//! or binary fails the run.

mod access;
mod flow;
mod lifecycle;
mod mcp;
mod rest;
mod review;
mod stack;

use serde_json::Value;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::{AuthenticatedUser, Permission};

use crate::{globals, principal};

const TOOLS_LIST_MAX_BYTES: usize = 6144;
const ONE_OP_EDIT_MAX_BYTES: usize = 400;
const HISTORY_MAX_BYTES: usize = 800;

#[derive(Debug, Default)]
pub(super) struct Measured {
    tools_list_bytes: usize,
    one_op_edit_bytes: usize,
    one_op_edit_wire_bytes: usize,
    history_bytes: usize,
    history_wire_bytes: usize,
    memo_in_one_create: bool,
    replace_and_next_in_one_edit: bool,
}

impl Measured {
    fn assert_within_budget(&self) {
        assert!(
            self.tools_list_bytes <= TOOLS_LIST_MAX_BYTES,
            "tools/list is {} bytes",
            self.tools_list_bytes
        );
        assert!(
            self.one_op_edit_bytes <= ONE_OP_EDIT_MAX_BYTES,
            "a one-op edit result is {} bytes",
            self.one_op_edit_bytes
        );
        assert!(
            self.history_bytes <= HISTORY_MAX_BYTES,
            "a three-revision history is {} bytes",
            self.history_bytes
        );
        assert!(
            self.memo_in_one_create,
            "the memo took more than one create"
        );
        assert!(
            self.replace_and_next_in_one_edit,
            "the replace and the next paragraph took more than one edit"
        );
    }
}

fn person(id: &UserId) -> AuthenticatedUser {
    AuthenticatedUser::new(
        id.clone(),
        id.as_str().to_owned(),
        format!("{}@contract.test", id.as_str()),
        vec![Permission::User],
    )
}

fn tools_list_bytes(list: &Value) -> usize {
    let tools = list["tools"].as_array().expect("tools/list lists tools");
    assert_eq!(
        tools.len(),
        systemprompt::runtime::documents::mcp::TOOL_NAMES.len(),
        "the binary lists core's document tools"
    );
    serde_json::to_string(list)
        .expect("tools/list serialises")
        .len()
}

#[tokio::test(flavor = "multi_thread")]
async fn docs_smoke_the_document_store_end_to_end() {
    assert!(globals::init(), "the contract fixture profile boots");
    stack::binary_path();
    let db = crate::tempdb::TempDb::create()
        .await
        .expect("docs-smoke needs SYSTEMPROMPT_TEST_DATABASE_URL");
    let credentials = principal::provision(&db.pool).await;
    stack::seed_system_admin(&db.pool).await;
    stack::seed_documents_access(&db.pool).await;
    let stack = stack::start(&db.url()).await;

    let owner_id = credentials.knowledge_worker_user_id.clone();
    let owner = person(&owner_id);
    let owner_token = stack.token(&db.pool, &owner_id).await;
    let member_token = stack.token(&db.pool, &credentials.non_admin_user_id).await;
    let outsider_token = stack.token(&db.pool, &credentials.admin_user_id).await;
    let viewers = access::Viewers {
        owner: &owner_token,
        member: &member_token,
        outsider: &outsider_token,
    };
    let rest = rest::Rest::new(&stack.api);
    let service = stack.ctx.documents().service();
    let mut mcp = mcp::McpClient::connect(&stack.mcp, &owner_token, owner_id.as_str()).await;
    let mut measured = Measured {
        tools_list_bytes: tools_list_bytes(&mcp.list_tools().await),
        ..Measured::default()
    };
    let document = flow::create_memo(&mut mcp, &mut measured).await;
    flow::guarded_edits(&mut mcp, service, &owner, &document, &mut measured).await;
    review::comment_and_resolve(&mut mcp, &document).await;
    access::share_to_group(&rest, &viewers, &document).await;
    review::propose_and_accept(&mut mcp, &rest, &owner_token, service, &owner).await;
    access::drive_pages(&rest, &viewers, &document).await;
    let owner_side = access::Owner {
        service,
        user: &owner,
        token: &owner_token,
    };
    access::bridge_round_trip(&rest, &mut mcp, &owner_side, &document).await;
    lifecycle::note_from_provider(&mut mcp).await;
    lifecycle::trash_and_untrash(&mut mcp, &document).await;

    println!("docs-smoke measurements: {measured:?}");
    measured.assert_within_budget();
    drop(stack);
    db.cleanup().await;
}
