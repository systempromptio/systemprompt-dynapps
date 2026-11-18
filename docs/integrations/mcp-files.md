# Files in MCP tools: the shared file layer

Any MCP server in this repo can take files from and hand files to the model
without the bytes passing through the model's context. The one exception is
text a server chooses to return inline because it is small enough for the
model to read; the layer audits it like any other download.
`systemprompt-mcp-shared::files` (`extensions/mcp/shared/src/files/`) does it
on top of core's document store. A file goes into a tool as a
**`FileRef`** and comes out as a **`FileOut`**. The bytes travel out of band,
through signed upload and download links. The layer knows nothing about the
upstream a server talks to, so the same code serves any MCP server.

It follows the pattern the Odoo MCP server already uses (`dyn_odoo_mcp`,
`odoo_request_upload`). The model asks for a single-use upload URL, the
client sends the file there, and the file is then referenced by id. Base64
in the arguments is only accepted for small files.

## The four operations

| Operation | What it does |
|---|---|
| `McpFiles::request_upload(call, &UploadRequest)` | Opens a resumable upload owned by the caller (`name`, `mime`, `size`, `sha256`) and returns an `UploadTicket`: the `parts_url` / `complete_url` / `status_url` links, the `upload_token` with the header to send it in, `part_size` / `part_count`, `expires_at`, plain-text `instructions` and the `file_ref` (`{"upload_id": …}`) that names the file once the upload is complete. Called again with only `upload_id`, it renews the token of an unfinished upload. |
| `McpFiles::resolve_ref(call, &FileRef)` | Turns a reference into a `ResolvedFile` (name, MIME type, size, byte stream). The stream is capped at the size limit: `into_stream()` and `read_all()` both stop with an error at the first byte past it. A file whose size the store does not report is read under the limit before `resolve_ref` returns, so a refusal for size is audited like any other. |
| `McpFiles::store_download(call, DownloadedFile)` | Keeps bytes the server fetched from its upstream as a `file` document of the caller. It returns a `FileOut` with the document reference and, when the API signs one, a download link valid for 10 minutes. |
| `McpFiles::return_inline(call, size_bytes)` | Audits bytes the server fetched and returns to the model itself instead of keeping them (small text). It writes the same `imported` row as `store_download`, labelled `inline`. A call with no signed-in user is refused, and the server then returns nothing. |
| `McpFiles::decode_inline(&str)` | Decodes base64 of up to `inline_max_bytes` (64 KB by default). Anything larger is refused with an error that names the server's upload tool. |

A `FileRef` names exactly one source:

- `upload_id`: from the server's upload tool, once the client has completed the upload.
- `blob_ref`: from `documents_upload`.
- `document_id`, with an optional `revision_id`: one of the caller's own documents.
- `content_base64`: small files only.

`name` and `mime` override the stored ones. The type is a flat object of
strings and is always inlined into the schema that embeds it, so a tool that
takes `attachments: Vec<FileRef>` stays declarable to every provider (no
`$ref`, no type lists).

## Rules the layer enforces, once, for every server

- **Ownership.** A reference resolves only for the user who holds it: an
  upload or blob they uploaded, or a document **they own**. A document shared
  with the caller is refused. The caller can copy it with `documents_copy`
  and pass the copy. A refused reference reads as *not found*, so the error
  never confirms that another user's file exists. Core checks first: the
  upload service filters by owner, and the document service authorizes the
  read as the caller and the calling agent. The layer then checks ownership
  again before any byte is read.
- **Expiry.** The API signs the links: upload tokens last 15 minutes, download
  links 10. A ticket's `expires_at` is the earlier of the token's expiry and
  the session's. An unfinished upload answers *not ready*. An expired,
  aborted or refused upload, or a completed one older than `ref_ttl`
  (24 h by default, core's grace period for unimported blobs), answers
  *no longer available* and points to the upload tool. `ref_ttl` applies to
  every reference the store dates, and `DocumentsFileStore` dates only
  upload ids. A `blob_ref` stays valid for as long as core keeps the blob.
  Core records when the upload behind it finished, but its upload service
  answers a blob's owner and type only (`BlobOrigin`), not that time.
  Aligning it with the `upload_id` needs that time from core. In practice
  core's blob garbage collector removes an unimported blob
  `blob_gc_grace_hours` (24 h by default) after it was stored. A blob that
  lives longer is also held by a document (a revision, an edit session or
  an embedded image). Either way, only the user
  who uploaded it can resolve it.
- **Limits.** Inline base64 is capped at 64 KB, and each file at
  `max_file_bytes` (25 MiB by default). Core's own `maxUploadBytes` (64 MiB
  by default) and daily quota still apply.
- **Names and types.** A name is a single file name, with no path separators
  or control characters. A downloaded file keeps only the last part of its
  name. If the name is already taken in the caller's root folder, the layer
  retries as `name (2).ext`, `name (3).ext` and `name (4).ext`, then once
  with a random 8-character tag. A retried name never exceeds 255 bytes.
  MIME types are normalized and checked with the same grammar core admits.
  `allowed_mime` (with `type/*` wildcards) can narrow them per server. The
  stored type must be admitted even when the caller passes another `mime`,
  so a file cannot be relabelled past the list. A download with an unusable
  type is stored as `application/octet-stream`. Core sniffs every file's
  bytes against its declared type.
- **Audit.** Every operation writes one `user_activity` row in the `mcp_access`
  category, with `entity_type = file`, including refusals. The row's action
  is `created` for an upload, `used` for a read, `imported` for a download
  (stored, or returned inline) and `rejected` for a refusal. It names the file by reference only
  (`upload:<id>`, `blob:<hash>`, `document:<id>`, `inline`), and a refusal by
  its error code, never by file name or address. A refusal of a call with no
  signed-in user is attributed to the anonymous principal, as
  `record_mcp_access_rejected` does.
- **Logs.** The `Debug` output of `FileRef`, `UploadTicket`, `FileOut`,
  `NewStoredFile` and `SignedDownload` leaves out inline content, the upload
  token and the signed URL.

## Using it from a new MCP server

1. Depend on `systemprompt-mcp-shared` (every bundled server already does).
2. Build the layer once in `main.rs`, next to the server's other state:

   ```rust
   use systemprompt_mcp_shared::files::{FilePolicy, McpFiles};

   let files = McpFiles::from_host(
       &ctx,
       service_id.clone(),
       FilePolicy::new("myserver_request_upload"),
   )?;
   ```

   `FilePolicy::new` takes the name of the server's own upload tool, which
   the errors point the model to. The other limits are public fields.
3. In each tool, build a `FileCall` from the request the server already
   authenticated (`authenticate_tool_request`). It also needs the tool name,
   the execution id and the hop token (`auth::verified_hop_token(&ctx)`),
   which the API requires to sign a link:

   ```rust
   let call = FileCall { ctx: &request_context, tool: "myserver_send", exec_id, hop: hop.as_ref() };
   let file = files.resolve_ref(call, &input.attachment).await?;   // FilesError -> rmcp::ErrorData via `?`
   let bytes = file.read_all().await?;
   ```

4. Expose an upload tool (`<server>_request_upload`) whose input is
   `UploadRequest` and whose output is `UploadTicket`. Tools that take files
   accept `FileRef`, and tools that return files return `FileOut` instead of
   base64.
5. Declare the `documents` toolset on the server in its `services/mcp/<name>.yaml`:

   ```yaml
   mcp_servers:
     myserver:
       toolsets:
         - id: documents
   ```

   The API signs a document link only for a server that hosts that toolset
   (core's `ensure_hosted`). Without it, `request_upload` fails with
   `link_unavailable`, and `store_download` still stores the file but returns
   no `download_url`. For a server built from its own binary, the field only
   affects that check.

## What the client does with a ticket

A client that can make HTTP requests, such as Claude Code, sends the parts
itself. It puts each part to `parts_url` with `{n}` replaced by the part
number, then posts to `complete_url`. Every request carries `upload_token` in
the `token_header` header. It then passes `file_ref` to the tool that needs
the file. Clients that cannot make HTTP requests (a web or desktop chat) can
only use small files inline, or documents the user already has in the
document store.

## Testing

The layer's decisions are tested without Postgres:
`tests/unit/mcp/src/files_*.rs` run `McpFiles` over an in-memory `FileStore`
and an audit sink that records the rows. `DocumentsFileStore`, the core-backed
store, is exercised end to end by the local simulation.
