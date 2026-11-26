# Dynapps AI MCP — Knowledge

Exposes documentation to AI assistants through the **Dynapps MCP server**
([`dyn_odoo_mcp`](https://gitlab.dynapps.be/dynapps-apps/dyn_odoo_mcp)), so a client
like Claude can **discover, search, read and author** documentation as MCP tools —
**running as the calling user**, with Odoo's permissions applied on every request.

The tools are built to handle **very large documentation corpora** and to serve them
**without burning the caller's context/token budget**: search returns short passages,
and whole documents or whole documentations are handed off through **short-lived
presigned download URLs** that travel out-of-band (never through the LLM channel).

**23 tools = 12 read + 7 write + 4 publish.** All three groups are **off by default**
and gated behind separate opt-ins (`enable_knowledge_read` / `enable_knowledge_write` /
`enable_knowledge_publish`), so installing this module never changes the tool surface of
pre-existing MCP servers. Enable read (and, if wanted, write and/or publish) explicitly
on the server meant to serve documentation. Write and publish are deliberately separate
switches: authoring a document and publishing it — committing, pushing, merging — are
different levels of trust, and a server that may edit need not be one that may also
merge (docs/mcp.md §4).

## Read tools

Twelve generic, corpus-neutral read tools. Every result carries a `lang` metadata field;
search is **cross-lingual** (a query in one language finds relevant documents in any
language, ranked best-first, without re-mapping).

| Tool                   | Purpose                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | Key params                                                                             |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `docs_search`          | Hybrid **semantic + keyword** search across the whole corpus. Ask a full natural-language question; conceptually related passages match even without shared words. Cross-lingual. Returns short passages, not whole documents.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | `query` (req), `limit` (≤50), `lang`, `space`                                          |
| `docs_get`             | Return the full Markdown of **one** document, by `path` or `article_id`. Small documents come back inline in `markdown`; large ones return a presigned `markdown_url` + `size_bytes` instead, so big documents never enter the context.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | `path` **xor** `article_id`                                                            |
| `docs_list_spaces`     | Filterable, paginated catalogue of the available documentations ('spaces'). Use it to discover the valid `space` / `lang` values the other tools accept. Reports `total` so the caller refines instead of enumerating.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | `query`, `lang`, `limit`, `offset`                                                     |
| `docs_list`            | Paginated **flat** list of document paths **inside ONE** documentation (requires `space` or `repo_id` — not a global dump). For the hierarchy + subtree sizes use `docs_outline`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | `space` **or** `repo_id`, `limit`, `offset`                                            |
| `docs_outline`         | Return the **STRUCTURE (skeleton)** of the tree — titles and hierarchy only — from an anchor (`node` / `space` / `doc`). `descend` (default) expands downward up to `depth`, capped at `max_nodes` (`has_children`/`child_count` per node, `truncated`/`has_more`/`hint`). `ascend` walks upward from a `node` returning each ancestor with its `subtree_count` and a `recommended_ceiling` (the highest ancestor still under `max_nodes`) so you find the right root without pulling an enormous aggregator. Imports are hard-bounded by their space root; natives find the boundary by size. Bounded, inline — never a whole-tree dump.                                                                                                                                                                                                     | `node` **/** `space` **/** `doc`, `mode`, `depth`, `max_nodes`, `max_levels`, `offset` |
| `docs_export`          | Export an **ENTIRE** documentation as a single presigned download (`format='index'` for a llms.txt-style table of contents, `format='full'` for the whole concatenated Markdown). One `download_url`, out-of-band.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | `space` **or** `repo_id`, `format`, `lang`                                             |
| `docs_answer`          | Retrieve the most relevant documentation and **synthesise a cited answer**. Returns the answer plus the exact source documents. If no synthesis model is configured, returns the top passages (`mode: 'passages'`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | `question` (req), `lang`, `space`, `max_sources`                                       |
| `docs_metadata`        | Return **internal metadata** for one or a batch of documents — the fields you cannot infer from a search hit: `is_synced` (imported → read-only), **`can_write`** (may YOU edit it), author/editor + timestamps, repo/space/version/lang, source URL, commit sha, publication status, whether it is indexed (`indexed`/`chunk_count`), parent, child count, variant languages, description. Check this before any write.                                                                                                                                                                                                                                                                                                                                                                                                                      | `article_id` **/** `path` **/** `article_ids`, `limit`                                 |
| `docs_metadata_schema` | Describe the **typed metadata fields** of a documentation: what exists, of what type, and **what text to write into it**. `expects` on a relation is the one that matters — it says to write the login and not the display name, which is the text the value travels as in the repository. Anchor it on a document for the fields that apply there, or on a documentation to see everything it declares.                                                                                                                                                                                                                                                                                                                                                                                                                                      | `article_id` **/** `path` **/** `space` **/** `repo_id`                                |
| `docs_asset_list`      | List the **files** of one document — images, diagrams, PDFs, spreadsheets — with `attachment_id`, `name`, `mimetype`, `size_bytes` and where each one stands with the repository: `in_repository` + `path`, and `pending_publication` for a file attached in Odoo that Git has not seen yet.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | `article_id` **xor** `path`, `lang`, `version`, `limit`, `offset`                      |
| `docs_asset_link`      | Short-lived, presigned **download URL** for ONE file (out-of-band, served with the caller's own permissions). Only files of documents the caller may read.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | `attachment_id` (req), `article_id`                                                    |
| `docs_sync_status`     | **Git → Odoo**: is the imported documentation still in step with its repository, and **which files would arrive** if it were read again. Per repository: `sync_mode` (`auto`/`manual`), `last_sync`/`last_sync_state`, `remote_ahead` + `remote_ahead_since`, `pending_since`, `last_error`, `webhook`, `up_to_date`, and `incoming:{known, file_count, refs, modified_only, files:[{path, document, article_id}]}` read off the push events already delivered — no clone, no API call. **Synchronizing is not possible from the MCP**: it is a button on the repository form, for repository managers (`can_synchronize`). This is the Git → Odoo direction only: what has been edited in Odoo and is waiting to be published is summarised here by `pending_since`, and reported in full by the publish tools on a server that offers them. | `space`, `repo_id`                                                                     |

## Write (authoring) tools

Seven tools that let an assistant **author native documentation** as the calling user.
They are advertised only when **`enable_knowledge_write`** is on. Content is authored as
**Markdown**: it is stored verbatim as the canonical source (`dynapps_markdown`) and
rendered to the document body in the same write, so the RAG indexes exactly what was
authored. **Externally-managed (imported) documents are always protected** — any write
whose target _or_ parent is `dynapps_is_synced=True` is rejected with "externally
managed, edit it at source".

| Tool                  | Purpose                                                                                                                                                                                                                                                                                                                                                                                                                          | Key params                                                                                                      |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `docs_create`         | Create a **new native document** under a native, writable parent. Small body inline in `markdown`; large body out-of-band via `content_upload_id`. `title`, `front_matter` and `metadata` may be set directly, without pasting a YAML header inside the body — a pasted `.md` with its own header still works exactly as before.                                                                                                 | `parent_id` (req), `title` (req), `markdown` **\|** `content_upload_id`, `front_matter`, `metadata`, `position` |
| `docs_create_tree`    | Create a **whole nested subtree** in one call and get back the created ids. `nodes` is a nested list of `{title, markdown?, front_matter?, metadata?, children?}` — `front_matter`/`metadata` apply PER NODE, the same as a single `docs_create`. Capped at 200 nodes/call.                                                                                                                                                      | `parent_id` (req), `nodes` (req)                                                                                |
| `docs_update`         | Update a native document's `title` and/or content (Markdown → canonical + body). Rejects imported documents.                                                                                                                                                                                                                                                                                                                     | `article_id` (req), `title`, `markdown` **\|** `content_upload_id`                                              |
| `docs_move`           | Re-parent a native document under another native parent (rejects imported target/parent; blocks cycles).                                                                                                                                                                                                                                                                                                                         | `article_id` (req), `new_parent_id` (req), `position`                                                           |
| `docs_trash`          | Send a native document to the Knowledge trash — **soft, reversible** (no hard delete via MCP). `destructiveHint`.                                                                                                                                                                                                                                                                                                                | `article_id` (req)                                                                                              |
| `docs_request_upload` | Issue a one-time, short-lived **upload URL** so a large Markdown body goes to the server out-of-band. POST the `.md`, read `attachment_id` from the response, pass it as `content_upload_id`. Use for bodies > ~48 KB; inline `markdown` otherwise.                                                                                                                                                                              | _(none)_                                                                                                        |
| `docs_asset_upload`   | Add a **file** (image, diagram, PDF, spreadsheet) to a document, in two calls: `filename` → `upload_url`; POST the file; call again with the returned `attachment_id`. 25 MB, no executables, stored byte for byte. On a managed documentation the file is left pending publication and reaches the repository with the documentation's next publication — the nightly cron, or whoever administers the repository launching it. | `article_id` **xor** `path`, `filename`, `attachment_id` (2nd call)                                             |

## Publish tools

Four tools that move edits **to and through Git**, on a Git-managed documentation.
Advertised only when **`enable_knowledge_publish`** is on — a separate switch from
`enable_knowledge_write` (docs/mcp.md §4): a server may author documents without ever
being allowed to publish or merge them.

| Tool                   | Purpose                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               | Key params                                                   |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| `docs_changes_status`  | Read-only: what has been edited in Odoo and is not yet in Git (yours and other people's), and every change proposal already open, with its state and `waiting_because`/`waiting_note` when it has not landed.                                                                                                                                                                                                                                                                                         | `space`, `repo_id`, `mine_only`, `include_merged`            |
| `docs_propose_changes` | Commit your pending edits, push a branch and open (or extend) a change proposal (a MERGE REQUEST on GitLab, a PULL REQUEST on GitHub). Only your own edits, unless `include_other_authors`. Ask the user before calling it.                                                                                                                                                                                                                                                                           | `space`, `repo_id`, `commit_message`, `title`, `description` |
| `docs_review_proposal` | Review a proposal without opening the platform: totals and a file list by default; `paths`/`include_diff` to read text, capped by `max_diff_bytes`.                                                                                                                                                                                                                                                                                                                                                   | `proposal_id` **\|** `url`, `paths`, `include_diff`          |
| `docs_merge_proposal`  | Merge a proposal into the documentation's branch. Ask the user for an explicit go-ahead first. Refuses (`state: 'held'`) and does **not** merge, however many times it is called, when the proposal needs a person outside this tool: it carries an agent's work (`awaiting_review`), or it is larger than this documentation merges unattended. That state does not resolve itself the way `'open'`/`'checking'` does — only a person, from Odoo's own screen or the platform directly, moves it on. | `proposal_id` **\|** `url`                                   |

The **procedence mark** (`awaiting_review`, docs/mcp.md §2) is set on the row/asset/repo
by every MCP write, regardless of whether a co-author is configured, and inherited by
whatever proposal that work lands in — however it is published (this MCP, the nightly
cron, or the backend button). It is never cleared by publishing over it or by a person
merely calling a tool: only a person acting outside the MCP (the documentation's own
screen in Odoo, or approving directly on the platform) moves a marked proposal on.

## Cross-lingual search

Search recurses over the **whole corpus in every language** — the vector arm uses
multilingual embeddings, so a question asked in one language surfaces the best matches
regardless of their language. Results are **not** re-mapped to the query language: each
hit is tagged with its `lang` and the client reads it directly. The optional `lang`
parameter hard-restricts to a single language (it expands to that language's spaces);
without it, the best results win, mixed.

## Out-of-band downloads (token efficiency)

`docs_get` (large documents) and `docs_export` never stream bytes through the LLM. They
materialise the Markdown as a temporary `ir.attachment` — created with the **caller's
own ACL** — and return a **presigned, short-lived download URL** (HMAC, stateless, TTL a
few minutes) served by the MCP server's binary-download endpoint. The bytes go client ↔
Odoo directly. An `@api.autovacuum` GC removes the temporary attachments once their TTL
has passed. The inline/URL threshold for `docs_get` is ~12 KB.

## Authoring (canonical Markdown)

When a write tool creates or updates a document it writes the **verbatim Markdown** to
`dynapps_markdown` and the **rendered HTML** to `body` in the _same_ write. A small
adjustment in `dynapps_knowledge_base` recognises this and does **not** flag the article
for a body→Markdown reconversion, so the authored Markdown stays the canonical source
the RAG indexes. Native articles created without an explicit Markdown keep the previous
behaviour (the base cron derives Markdown from the body).

**Out-of-band upload.** Large Markdown bodies (> ~48 KB, near the MCP argument ceiling)
are not passed inline. The caller requests a one-time upload URL (`docs_request_upload`,
reusing the base `mcp.upload.token`: 256-bit, single-use, short TTL, the caller's ACL),
POSTs the `.md` file to it out-of-band, and passes the returned `attachment_id` as
`content_upload_id`. The write tool reads the attachment's bytes as UTF-8 Markdown and
deletes the temporary attachment once consumed.

`docs_request_upload` needs **both** `enable_knowledge_write` **and** the base module's
`enable_attachments`: it is the documentation tools' door to out-of-band file transfer,
which is the switch `enable_attachments` owns.

`content_upload_id` accepts **only** an attachment this server materialised for this
user (proven by the `mcp.upload.token` row that created it). Any other id — including
one the caller can perfectly well read — is refused, and refused before anything is read
or deleted: consuming an upload means unlinking it, and a wrong id must never cost
somebody a real file. An upload no write ever consumes is swept by an `@api.autovacuum`
a day after its URL expires.

**Files (assets).** A documentation is prose plus the files the prose is about, and all
three tools above run over the same out-of-band channel as the Markdown one, with two
decisions worth stating.

_The upload is not re-encoded._ Odoo resizes any image over
`base.image_autoresize_max_px` (1920 px) as it stores it; a documentation file is
uploaded to be read, so the asset flow materialises it under `image_no_postprocess` and
what reaches the repository is the file that was sent, byte for byte.

_An SVG keeps its mimetype only if it cannot do anything._ Odoo rewrites SVG/XML uploads
to `text/plain` unless the uploader may write `ir.ui.view` — a rule this module does not
argue with in general, but one that costs a documentation permanently: the import
deduplicates by checksum, so a mislabelled diagram comes back mislabelled from Git and
never renders again. The label is therefore restored for an SVG with no script, event
handler, `javascript:` URL, `<foreignObject>` or declared entity in it, and withheld for
anything else — which is stored and published all the same, just not called a picture.
Unlike the import, this content comes from a person rather than from a reviewed
repository, so the check is on the bytes and not on their origin.

_Authorisation is the documentation's own._ `docs_asset_link` does not use the base
module's `_authorize_attachment`: that one serves a file the caller uploaded through the
server or one whose parent record is reachable by ORM, and a documentation's files are
neither (they were written by the sync, and `knowledge.article` is not an exposed ORM
model on a documentation server), so it would refuse the entire imported corpus. A file
is served when it belongs to a document **this caller may read**, checked through
Knowledge's access control.

**Import guard.** Every write tool refuses when the target (or, for create/move, the
parent) is `dynapps_is_synced=True`: imported documentation is edited at its source,
never through the MCP. `docs_metadata` exposes `is_synced` / `can_write` so a client can
check before attempting a write.

## Typed metadata, end to end

A documentation declares its own typed fields in `knowledge.yml`; the values live in
each document's front matter. Three tools close the loop, and the order matters:

1. **`docs_metadata_schema`** — what may be set, and in what shape. A relation reports
   `expects`: the field of the record whose text the value travels as. Without it an
   agent writes a display name, the lookup finds nothing and the field silently stays
   empty.
2. **`docs_update {metadata}`** — set it, in that same vocabulary. Values are cast by
   the importer's own code, so a value entering here and one entering through a file are
   the same value. What cannot be resolved comes back in `metadata_warnings` and is
   **not** written — never guessed, because the first match for an ambiguous owner files
   somebody's document under a colleague's name.
3. **`docs_metadata` / `where`** — read it back and filter on it, immediately.

The third step is the one that used to fail. Setting metadata wrote the header alone,
and the typed layer only caught up when the value had gone out to Git and come back
through an import — on a native document, never. Now the typed layer is written and the
standing machinery does the rest: the header is updated, the row is flagged, and the
write-back carries it to the repository under whoever set it.

`false` clears a field, and a cleared field leaves the header rather than sitting in it
as `null`: a header says a field is empty by not carrying it.

## How it plugs in

The Dynapps MCP server discovers extra tools through overlay hooks. This module
implements them on `mcp.server.config`:

- `_get_extra_tools` — advertises the enabled tools (with annotations; the `docs_trash`
  write tool carries a `destructiveHint`).
- `_dispatch_extra_tool` — routes a call to the right handler.
- `_resolve_tool_schema` — returns each tool's JSON schema.
- `_knowledge_tool_enabled` — gates each tool: read tools require
  `enable_knowledge_read`, write tools require `enable_knowledge_write`, publish tools
  require `enable_knowledge_publish`.

Tool logic lives in `models/_knowledge_tools.py` as a `KNOWLEDGE_TOOLS` registry (the
same `handler(config, user, args)` pattern as the base server's `CORE_TOOLS`); each
entry carries a `write` flag and, for the four publish tools, a `publish` flag instead.
Every call executes in the calling user's environment (`config.env(user=user.id)`) —
**never `sudo`** — so a caller only ever reads or writes documentation they are allowed
to. Read tools are advertised `read_only`; write and publish tools are not
(`docs_changes_status` and `docs_review_proposal` are themselves read-only in effect —
they change nothing — but gated with the publish family, because a server that may not
publish has no proposals to report on).

## Configuration

- **Enable the read tools.** On the MCP server configuration (`mcp.server.config`) set
  **`enable_knowledge_read = True`** (default **False**). The twelve read tools then
  appear automatically in that server's `tools/list`. It is off by default so that
  installing this module never adds documentation tools to MCP servers that already
  exist for other purposes — enable it only on the server(s) that should serve docs.
- **Enable the write tools.** Set **`enable_knowledge_write = True`** (default
  **False**) to expose the seven authoring tools. The module is read-only until this is
  turned on explicitly.
- **Enable the publish tools.** Set **`enable_knowledge_publish = True`** (default
  **False**) to expose the four tools that commit, push and merge to Git. Separate from
  write on purpose: a server may author documents without being allowed to publish or
  merge them. A server upgraded from before this switch existed keeps publishing on its
  existing `enable_knowledge_write` toggle — the migration turns this one on wherever
  that one already was.
- **Grant access.** A server with no allowed groups and no allowed users is PRIVATE. Add
  the intended group (e.g. Internal Users) and/or specific users. Leave
  `allowed_client_ids` empty to accept any authenticated OAuth client, or restrict it to
  specific clients.
- **Semantic search** relies on `dynapps_knowledge_rag`: configure the embedding
  provider and key and index the corpus. AI providers are resolved per capability by
  `dynapps.ai.config` under the `dynapps_ai.*` namespace — embeddings come from
  `dynapps_ai.embed_provider` + `dynapps_ai.key.embed.<provider>` (or the shared
  `dynapps_ai.key.<provider>`). The legacy `dynapps_knowledge_rag.provider` /
  `dynapps_knowledge_rag.<provider>_key` parameters still work as a fallback. Without an
  embedding key, `docs_search` / `docs_answer` fall back to a keyword (`ilike`) search.
- **`docs_answer` synthesis** uses a chat model, resolved the same way:
  `dynapps_ai.chat_provider`, `dynapps_ai.key.chat.<provider>` (or the shared
  `dynapps_ai.key.<provider>`), and optional `dynapps_ai.chat_model` — with the legacy
  `dynapps_knowledge.ai_provider` / `ai_<provider>_key` / `ai_model` still honoured.
  Without a chat key, `docs_answer` degrades gracefully to returning the retrieved
  passages (`mode: 'passages'`) rather than a synthesised answer.

## Dependencies

`dynapps_mcp_server` (Dynapps MCP) + `dynapps_knowledge_rag` (semantic search).

## Technologies referenced

[Model Context Protocol](https://modelcontextprotocol.io/) · Dynapps MCP server · the
RAG hybrid search.
