# Documentation MCP — reference index

The tools of a documentation MCP server, grouped by what you're doing. Read the file for
your task; the **conventions below apply to every tool**, so the individual files don't
repeat them.

- **`read-tools.md`** — search, retrieve, navigate: `docs_search`, `docs_answer`,
  `docs_get`, `docs_list_spaces`, `docs_list`, `docs_outline`, `docs_export`,
  `docs_metadata`, `docs_metadata_schema`, `docs_asset_list`, `docs_asset_link`,
  `docs_sync_status`, plus how to phrase retrieval queries and how to filter by typed
  metadata (`where`).
- **`write-tools.md`** — author / edit: `docs_create`, `docs_create_tree`,
  `docs_update`, `docs_move`, `docs_trash`, `docs_request_upload`, `docs_asset_upload` —
  and the three arguments (`title`, `metadata`, `front_matter`) every one of them takes.
- **`pending-work.md`** — what happens to a change to an **imported** document after you
  make it: the four states, what a repository's format changes, why anything an assistant
  wrote waits for a person, and — for the servers that expose them — the four publish tools.
- **`workflows/`** — how to **chain** tools together, one file per kind of task, with
  `README.md` as the index: **one line per workflow so you can pick without opening
  anything**, plus the composition cheatsheet. The files are `reading.md` (find, read,
  verify), `authoring.md` (create, edit, reorganize, retire, large content), `files.md`
  (the images, diagrams and PDFs a documentation carries), `pending.md` (saying where the
  work stands) and `troubleshooting.md` (when something does not add up).

## Families, and why a tool you read about may not be there

A server exposes the **families** it was granted, so two servers of the same product do not
offer the same tools. Three families:

| Family | Tools |
| --- | --- |
| **read** | `docs_search`, `docs_answer`, `docs_get`, `docs_list_spaces`, `docs_list`, `docs_outline`, `docs_export`, `docs_metadata`, `docs_metadata_schema`, `docs_asset_list`, `docs_asset_link`, `docs_sync_status` |
| **write** | `docs_create`, `docs_create_tree`, `docs_update`, `docs_move`, `docs_trash`, `docs_asset_upload`, `docs_request_upload` |
| **publish** — a server exposes them or does not, so **check your own tool list** | `docs_changes_status`, `docs_propose_changes`, `docs_review_proposal`, `docs_merge_proposal` |

**The publish family is off by default, and off on the servers that write documentation by
day and let a scheduled pass publish at night.** Read your own tool list before promising a
step, and never call a tool you cannot see: told to call one it does not have, a model
retries, invents a substitute, or reports work as published when it is not. The tools
themselves are careful about this — no answer from the read or write families ever names a
publish tool — so if you find yourself about to say `docs_propose_changes`, the instruction
came from you, not from the server.

## Conventions (apply to every tool)

- **Namespaced call:** the real callable is `mcp__<server-name>__docs_search`, etc. The
  files use the short names (`docs_search`). Resolve the server as in `SKILL.md` — never
  hardcode a name; identify the documentation MCP by its `docs_*` tool family, and ask
  the user which server to use when it is ambiguous.
- **Runs as the calling user**, honoring Knowledge permissions (never superuser). Empty
  retrieval can mean "no access", not "nothing there" — but it no longer means "your
  documents lost the ranking to somebody else's". The reader's permissions are applied
  **inside** the search, so a customer with fifty documents is ranked among their own
  fifty and not against the whole corpus.
- **Errors** come back as `{ "error": "<message>" }` — a normal result, not an
  exception. Read it and adjust; don't blindly retry the same call. A **refused write
  unwinds whole**: no half-created document, no half-applied metadata, and an upload token
  it would have consumed stays usable.
- **Pagination:** list-style tools return `total` (all matches), `shown`/`limit`, and
  `offset`; `docs_outline` adds `has_more` / `truncated` / `hint`. Page by advancing
  `offset`; refine rather than enumerate everything.
- **Variants:** a document can exist per `(lang, version)`. Read/author a specific one
  with `lang`/`version`; `docs_get` reports the `variant` actually served (it falls back
  to the default when the asked-for one is absent). Retrieval is cross-lingual regardless.
- **Bytes never travel through this channel.** A large document, a whole export, a PDF or
  a diagram comes back as a **short-lived signed URL** (`~15 min`, `expires_minutes` in
  the answer), and one goes in over a **single-use** upload URL. No tool of yours fetches
  or posts those bytes: that is your shell's job, or the user's browser
  (`workflows/files.md`).
- **Three origins, three duties.** `docs_metadata` → `repo` / `is_synced` / `can_write`
  tells you which one you are holding: **native** (no `repo`) is writable and there is
  nothing to publish afterwards; **managed** (`repo` + `can_write: true`) is writable and
  the change stays pending until the documentation's next publication carries it
  (`pending-work.md`); **mirror** (`repo` + `can_write: false`) is read-only and edited at
  its Git source. Check it **before** you write — the table and the reasoning are in
  `SKILL.md`.
- **The answer is written for the server you are on.** `publish_required`, `next_step` and
  the `message` of an asset upload are composed per call, and they take the repository's
  format and your own tool list into account. Read them out; do not substitute a step you
  remember from somewhere else.
- **After a server deployment, reconnect the MCP** (`/mcp`). The tool list and their
  descriptions are negotiated when the session opens, so a session started before the
  deployment keeps calling the old tools with the old behaviour — including bugs that were
  fixed hours ago. If a documented field is simply not there, reconnect before debugging.

## Scaling this reference

One file per tool group, and — since the recipes outgrew a single page — one file per kind
of task under `workflows/`, indexed by its `README.md`. The rule that produced both: a
reference is only useful if you can reach the part you need **without reading the rest**.
When a page stops passing that test, split it and leave an index behind. If a single tool's
section grows large (many params, many worked examples), split just that tool into its own
file under a `read/` or `write/` subfolder and link it from the relevant group file —
without turning every tool into a separate file prematurely.
