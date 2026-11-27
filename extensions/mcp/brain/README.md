# `brain` MCP server

The Brain's operations as MCP tools, served to signed-in people as themselves: `systemprompt-mcp-brain`, port 5014,
configured in `services/mcp/brain.yaml` (`oauth.required`, no toolsets: never the core's own `brain` toolset).

The tools carry no prefix; the client adds the server's name (DynWork lists them as `brain_search`, `brain_get`...).
Every call is authenticated, recorded in the MCP access ledger and run as the person the request names, through the
`Brain` trait of `extensions/dynbrain`. The server has no logic of its own: arguments become the operation's arguments
(the field names are the `docs_*` ones), and what the operation answers is returned as structured JSON. A call that
carries no person is refused (`Forbidden`) before any operation runs.

Descriptions and schemas keep the guidance of the `docs_*` tools (`confidence: "low"` in `search`, `markdown_url` in
`get`, the `[n]` citations of `answer`) and name neither the source system nor its product.

| Tool | Equals | Arguments (`*` required) |
|---|---|---|
| `search` | `docs_search` | query*, limit, lang, space, where |
| `answer` | `docs_answer` | question*, lang, space, max_sources |
| `get` | `docs_get` | article_id or path, lang |
| `list_spaces` | `docs_list_spaces` | query, lang, limit, offset |
| `list` | `docs_list` | space*, lang, limit, offset, where |
| `outline` | `docs_outline` | node or space, mode, depth, max_nodes, max_levels, lang, offset |
| `export` | `docs_export` | space*, format, lang |
| `metadata` | `docs_metadata` | article_id, path or article_ids; limit, lang |
| `metadata_schema` | `docs_metadata_schema` | article_id, path or space |
| `create` | `docs_create` | title*, parent_id or space, markdown, position, front_matter, metadata, lang |
| `create_tree` | `docs_create_tree` | nodes*, parent_id or space, lang |
| `update` | `docs_update` | article_id*, title, markdown, front_matter, metadata, lang |
| `move` | `docs_move` | article_id*, new_parent_id*, position |
| `trash` | `docs_trash` | article_id* |
| `request_upload` | `docs_request_upload` | article_id or path, filename |
| `asset_list` | `docs_asset_list` | article_id or path, lang, limit, offset |
| `asset_link` | `docs_asset_link` | attachment_id*, article_id |
| `asset_upload` | `docs_asset_upload` | article_id or path, filename, upload_id, lang |

Left out: the Git tools (`docs_changes_status`, `docs_propose_changes`, `docs_review_proposal`, `docs_merge_proposal`,
`docs_sync_status`) and `version`, which native spaces do not have.

Differences from the `docs_*` contract:

- `create` and `create_tree` take `space` for a top-level article, and no `content_upload_id`: the Markdown goes inline.
- Files are native: `request_upload` needs the article and the file's name, the bytes are POSTed as the raw body
  (never a multipart form), and the second call of `asset_upload` names the `upload_id`, not an `attachment_id`.
- A `where` naming a property no space declares answers `{error, results: [], total: 0}` (`documents` for `list`),
  not a tool error, as the `docs_*` do.
- The process holds no manifest signing seed, so without it the file tools answer that files are not available (see
  `extensions/dynbrain/README.md`).

The query embedder of `search` is filled by this server from an AI service built on the host's granted secrets. Tests:
`tests/unit/mcp-brain`, `tests/integration/mcp-brain`.
