# Documentation MCP — read tools

The read tools, each with input, output, error shapes, and notes, plus how to
phrase retrieval queries. **Shared conventions** (namespacing, run-as-user, errors,
pagination, variants, families) are in `README.md` — read it once. For chaining tools
together see `workflows/` (index in its `README.md`); for authoring see `write-tools.md`.
Two further read-only tools — `docs_changes_status` and `docs_review_proposal` — belong to
the **publish family**, which a server exposes or does not; they are documented with it in
`pending-work.md` §6. What every server has instead, for "where does my work stand", is
`docs_sync_status`, below.

---

## docs_list_spaces

Discover the available documentations ("spaces") and the valid `space` / `lang` values
the other tools take. Filterable and paginated. Use it to orient/scope — not to "find
documentation about X" (that's `docs_search`).

Input:
```json
{ "query": "trust", "lang": "es", "limit": 50, "offset": 0 }
```
Output:
```json
{
  "spaces": [
    { "space": 1, "name": "Trust Center · main · es", "lang": "es", "version": "main", "article_count": 41 },
    { "space": 5, "name": "Platform · main · es", "lang": "es", "version": "main", "article_count": 47 }
  ],
  "total": 2, "shown": 2, "offset": 0
}
```
Notes: all args optional; no filter → the whole catalogue (paged). The `space` integer
is what you pass to `docs_search` / `docs_list` / `docs_outline` / `docs_export`.

---

## docs_search

Hybrid semantic + keyword search across the corpus, reranked and cross-lingual. Returns
short ranked passages, not whole documents. The default way to **find** relevant docs.

Input (broad):
```json
{ "query": "How often are backups taken and how long are they kept?", "limit": 5 }
```
Input (scoped to one documentation — the main lever for filtering noise):
```json
{ "query": "database backup frequency and retention", "space": 1, "limit": 5 }
```
Output:
```json
{
  "query": "How often are backups taken and how long are they kept?",
  "mode": "hybrid",
  "results": [
    { "id": 3765, "path": "resilience/backups.md", "title": "Backups",
      "lang": "es", "space": 1, "score": 0.82,
      "snippet": "# Backups We back up your environment periodically and verified…" }
  ],
  "total": 5
}
```
Input (scoped to a few documentations, and to documents nobody owns):
```json
{ "query": "how do we handle an escalation?", "space": [7, 8], "where": { "owner": false } }
```
Notes:
- `score` is the reranked relevance; results are already best-first.
- Pass `space` to restrict to one documentation **or a list of them**, `lang` to restrict to
  one language (usually leave `lang` off — cross-lingual is the point).
- **`where` filters by the typed metadata** the documentation declares — `{"owner": false}`
  for documents nobody signs, `{"status": "approved"}` for the approved ones. It restricts
  the search itself, so a narrow filter returns what matches, not what happened to survive
  the top of the ranking. An undeclared field is refused and the answer lists what exists;
  `docs_metadata_schema` is the tool that tells you beforehand.
- **A relation is filtered by its stored id**, not by the text you would write into it
  (`{"owner": 42}`, not `{"owner": "j.smith"}`) — read the id from `docs_metadata`.
- Read a hit with `docs_get` using its `path` (or `id` as `article_id`).

---

## docs_answer

Retrieve the most relevant docs and **synthesize a cited answer**. Use when the user
wants an answer, not a list. Returns the answer plus the exact sources (verifiable). If
no synthesis model is configured it returns top passages instead (`mode: "passages"`).

Input:
```json
{ "question": "Who can approve a publication, and what is the backup retention?",
  "max_sources": 6, "space": 1 }
```
Output (synthesized):
```json
{
  "answer": "Only a user with the Curator role can approve a publication [2]. Backups run every 6 hours and are kept for 30 days [1].",
  "citations": [
    { "path": "admin/backups.md", "title": "Backups", "lang": "es",
      "snippet": "# Backups … Frequency: every 6 hours. Retention: 30 days." },
    { "path": "admin/roles.md", "title": "Users and roles", "lang": "es",
      "snippet": "Only the Curator role can approve a publication…" }
  ],
  "mode": "synthesized"
}
```
Output (fallback when no synthesis model is configured):
```json
{ "answer": null,
  "citations": [ { "path": "admin/backups.md", "title": "Backups", "lang": "es", "snippet": "…" } ],
  "mode": "passages" }
```
Notes: citation numbers `[n]` map to the `citations` order. If it answers "not in the
sources" for something you believe exists, raise `max_sources`, drop the `space`/`lang`
filter, or rephrase — see query tips below.

---

## docs_get

Return the full Markdown of **one** document, by `path` **xor** `article_id`. Small docs
come back inline in `markdown`; large ones return a presigned `markdown_url` instead
(fetch out-of-band — see `workflows/authoring.md` §6). Select a variant with `lang`/`version`.

Input (inline):
```json
{ "article_id": 3884 }
```
Input (a specific language variant, by path):
```json
{ "path": "admin/backups.md", "lang": "en" }
```
Output (inline):
```json
{ "id": 3884, "path": "admin/backups.md", "title": "Backups", "lang": "es",
  "found": true,
  "children": [ { "article_id": 3885, "title": "Restore a backup", "description": "" } ],
  "variant": { "lang": "es", "version": "main", "is_default": true },
  "source_url": "https://gitlab.example.com/group/docs/-/blob/main/docs/admin/backups.md",
  "markdown": "# Backups\n\n- Frequency: every 6 hours.\n- Retention: 30 days.\n" }
```
`source_url` is the source file **of the variant served** — ask for `lang: "en"` and you get
the English `.md`, not the Spanish one. Empty for a native document (it has no repository).

**`markdown` is the canonical text of the file, and only that.** A section page's child
index is not in it; the children come back beside it, as `children` (id, title,
description). That separation is what makes read-edit-write safe: whatever you read is
what the file holds, so passing it back through `docs_update` cannot commit a generated
list into the customer's repository as if a person had written it. Use `children` to
navigate or to build a summary — never paste it into the body you send back.
Output (large → out-of-band):
```json
{ "id": 4120, "path": "guides/full-install.md", "title": "Full install", "found": true,
  "variant": { "lang": "es", "version": "main", "is_default": true },
  "markdown_url": "https://<host>/mcp/<db>/<slug>/download/<token>",
  "size_bytes": 75625, "expires_minutes": 15 }
```
Errors / edge cases:
- Neither or both of `path`/`article_id` → input-constraint error ("provide at least one
  of path or article_id"). Pass exactly one.
- Not found / no access → `{ "found": false, ... }`.
- Asked-for `lang`/`version` absent → serves the default and says so in `variant`
  (`is_default: true`, and its `lang`/`version` differ from what you asked).

---

## docs_list

Flat, paginated list of document **paths inside ONE documentation** (requires `space` or
`repo_id` — not a global dump). For the hierarchy + subtree sizes use `docs_outline`.

Input:
```json
{ "space": 1, "limit": 8, "offset": 0 }
```
Output:
```json
{ "documents": [
    { "path": "compliance/audits.md", "title": "Audits", "lang": "es" },
    { "path": "faq.md", "title": "FAQ", "lang": "es" } ],
  "total": 41, "shown": 8, "offset": 0 }
```
Input (only the documents nobody owns, inside one documentation):
```json
{ "space": 1, "where": { "owner": false } }
```
Notes: neither `space` nor `repo_id` → error (it refuses a global dump). Page with
`offset` up to `total`. `where` takes the same typed fields as in `docs_search` — see
`docs_metadata_schema` for what a documentation declares. This is the tool for
**inventory** questions ("what is left to sign"), where `docs_search` answers "what is
relevant to this question".

---

## docs_outline

Return the **structure (skeleton)** — titles and hierarchy only — from an anchor
(`space`, `doc`, or any `node` article id). Bounded so it never dumps a giant tree. This
is THE tool for discovering and navigating a tree; see the tree-discovery recipes in
`workflows/reading.md`.

Two modes:

- `descend` (default): expand downward up to `depth` (default 2; `-1` = whole subtree,
  still capped by `max_nodes`, default 200). Each node reports `has_children` /
  `child_count`. If it overflows the cap you get `truncated` / `has_more` / `hint` and
  page the anchor's direct children with `offset`, or drill into a child (`node=<id>`).
- `ascend`: walk upward from a `node` up to `max_levels` (default 6). Each ancestor
  comes with its `subtree_count` (cheap) and the response gives a `recommended_ceiling`
  = the highest ancestor still under `max_nodes`. Use it when you hold ONE document and
  must find the right "root" to work from without pulling an enormous aggregator — the
  level where subtree size jumps is the natural boundary.

Input (descend one level to see top sections):
```json
{ "space": 1, "depth": 1, "max_nodes": 50 }
```
Output (descend):
```json
{ "anchor": { "id": 3740, "name": "Trust Center" },
  "nodes": [
    { "id": 3743, "title": "The platform", "lang": "es", "depth": 1, "has_children": true, "child_count": 3 },
    { "id": 3747, "title": "Infrastructure", "lang": "es", "depth": 1, "has_children": true, "child_count": 6 } ],
  "total": 10, "shown": 10, "truncated": false, "has_more": false, "offset": 0, "hint": "" }
```
Input (ascend from one page to find its section root):
```json
{ "node": 3882, "mode": "ascend", "max_levels": 6 }
```
Output (ascend):
```json
{ "node": { "id": 3882, "name": "Install & requirements" },
  "ancestors": [
    { "id": 3881, "name": "Administration guide", "level": 1, "subtree_count": 5, "is_space_root": false },
    { "id": 3740, "name": "Trust Center", "level": 2, "subtree_count": 210, "is_space_root": true } ],
  "recommended_ceiling": { "id": 3881, "name": "Administration guide", "subtree_count": 5 },
  "hint": "Work from 'Administration guide' (5) — the parent above jumps to 210." }
```
Notes: imported docs are hard-bounded by their space root (ascend never crosses it);
native docs have no import root, so the boundary is found by size.

---

## docs_export

Export an **ENTIRE** documentation as one presigned download (bytes travel out-of-band).
`format:"index"` = a llms.txt-style table of contents; `format:"full"` = the whole
concatenated Markdown.

Input:
```json
{ "space": 1, "format": "index" }
```
Output:
```json
{ "download_url": "https://<host>/mcp/<db>/<slug>/download/<token>",
  "size_bytes": 7119, "mimetype": "text/plain", "expires_minutes": 15, "article_count": 41 }
```
Notes: prefer `docs_outline` when you only need the shape to read inline; use `export`
when you actually want the bytes (ingest a whole documentation in one fetch).

---

## docs_metadata

Internal metadata you can't infer from a search hit — imported or not (read-only),
author/editor + dates, where it lives (repo/space/version/lang), publication status,
whether it's indexed, and — crucial before a write — whether **you** can edit it
(`can_write`). Single `article_id`/`path` or a batch of `article_ids`.

Input:
```json
{ "article_ids": [3884, 3885], "limit": 50 }
```
Input (the metadata of ONE language variant):
```json
{ "article_id": 3884, "lang": "en" }
```
Output (one item):
```json
{ "items": [
    { "id": 3884, "title": "Backups", "path": "admin/backups.md", "lang": "es",
      "is_synced": false, "can_write": true, "repo": "", "space": 1, "version": "main",
      "source_url": "", "commit_sha": "", "author": "Jane Doe",
      "created": "2026-07-29T08:24:54", "last_editor": "Jane Doe", "updated": "2026-07-29T08:27:18",
      "parent_id": 3881, "child_count": 0, "variant_langs": ["es", "en"], "description": "",
      "website_published": false, "indexed": true, "chunk_count": 1,
      "metadata": { "status": "approved", "owner": 42, "reviewed_on": "2026-08-14" },
      "front_matter": { "title": "Backups", "description": "", "status": "approved",
                        "owner": "j.smith", "audience": "internal" } } ],
  "total": 1 }
```
Reading it:
- **The origin, first.** `repo` + `can_write` together say which of the three kinds of
  document you are holding, and therefore what you must do **after** writing to it:

  | `repo` | `can_write` | Origin | After a write |
  | --- | --- | --- | --- |
  | `""` | `true` | **native** | nothing to publish — it is already done |
  | a name | `false` | **mirror** | you cannot write at all; edit at the Git source |
  | a name | `true` | **managed** | saved in Odoo and **pending publication** → `pending-work.md` |

  This is the check that decides whether "done" or "saved, and it reaches the repository
  with the next publication" is the true thing to tell the user. Run it before the write,
  not after.
- `is_synced: true` → imported (so `repo` is set). **Read-only unless its repository is
  `managed`** — trust `can_write`, not this flag, before an edit.
- `can_write: true` → you may `docs_update` / `docs_move` / `docs_trash` it.
- `source_url` → the source file of the variant you asked for; with `lang`/`version` the
  answer also carries a `variant` block saying which one it served.
- `indexed` / `chunk_count` → whether it is searchable yet (a just-created doc may not
  be indexed for a moment).
- **`metadata` and `front_matter` are two layers, not two copies.** `metadata` is the
  TYPED half — only the fields the documentation declares, the only ones `where` can filter
  on, and a relation appears as its **stored id**. `front_matter` is the OPEN half: every
  key the author wrote in the header, declared or not, exactly as the file holds it, so a
  relation appears there as the **text** it travels as. Same field, two representations —
  which is why you filter with one and write with the other. What may be set, and how, is
  `docs_metadata_schema`.
- No access → the item is filtered out (`total` smaller than requested), not an error.

---

## docs_metadata_schema

**What a documentation's typed fields are, and exactly what to write into each one.** Call
it **before** setting metadata with `docs_update` / `docs_create` and before filtering with
`where`: the fields are declared **per documentation and per section**, in a `knowledge.yml`
in Git, so they cannot be inferred from a document you have read — and naming one the
descriptor does not declare **fails the whole write**, so this is the call that makes the
difference between a document created and nothing at all.

Input (the fields that apply to one document):
```json
{ "article_id": 3884 }
```
Input (everything a documentation declares, before there is a document to ask about):
```json
{ "space": 1 }
```
Output:
```json
{ "article_id": 3884, "title": "Backups",
  "fields": [
    { "name": "status", "type": "selection", "label": "Status",
      "options": ["draft", "approved"], "expects": "exactly one of 'options' -- the key, not the label" },
    { "name": "owner", "type": "many2one", "label": "Owner", "comodel": "res.users",
      "value_field": "res.users.login", "expects": "the login of a res.users" },
    { "name": "reviewed_on", "type": "date", "expects": "a date as YYYY-MM-DD" } ],
  "total": 3 }
```

**`expects` is the instruction, and every field has one** — because a type name does not say
how to write it:

| type | what to write |
| --- | --- |
| `many2one` | the text named by `value_field` for ONE record — a login, a code, a reference. **Never the display name** |
| `many2many` | a **list** of those |
| `selection` | exactly one **key** from `options` — not the label an interface shows |
| `tags` | a **list** of keys from `options` |
| `date` / `datetime` | `YYYY-MM-DD` / `YYYY-MM-DD HH:MM:SS` |
| `boolean` | true or false |
| `integer` / `float` / `monetary` | a number |
| `char` / `text` / `html` | text |

A relation whose descriptor declares no `value_field` comes back with `writable: false` and
the reason: it cannot be set from text at all, and an id written into somebody's Markdown
would mean nothing in their repository.

**What `expects` promises, and what it does not.** A relation, a selection, a tag and a
number are resolved on the write, and what does not resolve lands in `metadata_warnings`
unwritten. A **`date`** and a **free-text** field are **not checked at all**: what you send is
stored and committed to the customer's file exactly as written. So `expects: "a date as
YYYY-MM-DD"` is an instruction to you, not a guard — send `2026-08-28`, never a phrase.

**Writing and filtering do not speak the same language for a relation.** You write the
text (`docs_update {metadata}`); `where` matches the stored **id**. Both ends are correct
and they are not interchangeable — read `owner` from `docs_metadata` → `metadata` to get the
id you filter by.

Anchor it on a document (`article_id` / `path`) to get the fields that apply **there** — a
section can declare its own on top of the documentation's — or on a documentation
(`space` / `repo_id`) for the whole block.

---

## docs_asset_list

The **files** of one document — images, diagrams, PDFs, spreadsheets. A documentation is
prose *plus* the files the prose is about, and nothing else in this server ever said a page
had any: reading `![](red.svg)` in the Markdown tells you a file exists, not how to reach
it. Identify the document exactly as `docs_get` does — `article_id` **xor** `path` —
and pick a variant with `lang` / `version`.

Input:
```json
{ "article_id": 3852 }
```
Output:
```json
{ "found": true, "article_id": 3852, "document_path": "docs/network/envoy.md",
  "title": "Envoy Gateway",
  "variant": { "lang": "es", "version": "main", "is_default": true },
  "in_repository": true, "repository": "Dyncloud docs",
  "files": [
    { "attachment_id": 369846, "name": "envoy-flow.svg", "mimetype": "image/svg+xml",
      "size_bytes": 18422, "in_repository": true, "path": "docs/network/envoy-flow.svg",
      "pending_publication": false },
    { "attachment_id": 369901, "name": "manual.pdf", "mimetype": "application/pdf",
      "size_bytes": 2411903, "in_repository": false, "path": "",
      "pending_publication": true } ],
  "total": 2, "shown": 2, "offset": 0 }
```

**`in_repository` appears twice and means two different things.** At the top level it is
the **document**'s: does this page belong to a Git documentation at all. Inside a `files[]`
row it is the **file**'s: are those bytes in Git *right now*. Quoting the wrong one is how
a pending file gets reported as published.

Read a `files[]` row as a pair, because one flag could not say it:

| `in_repository` | `pending_publication` | What it is |
| --- | --- | --- |
| `true` | `false` | In Git, nothing to do |
| `false` | `true` | Attached in Odoo, **not in the repository yet** — it travels with the documentation's next publication |
| `true` | `true` | Published once and replaced since — the new bytes are still pending |
| `false` | `false` | Odoo holds it and **nothing will publish it** — typically a file of a native document |

`path` is where the file lives in the repository, relative to its root, and it is **empty
until the file has been published once**. It is also empty — deliberately — for a
documentation that keeps its languages in folders: there the importer could have taken
`<lang>/img/x.png` or `img/x.png` and the record does not say which, so it names none
rather than the wrong one. An empty `path` is never a reason to guess.

Notes: `limit` (default 100, max 500) and `offset` page `files[]`, `total` is the whole
count. A file the page **shows** without owning is listed too (identical bytes are stored
once, so one attachment serves forty pages) — which is why `docs_asset_link` takes an
`article_id`. The server's own scratch files (a big document materialised for download, an
upload in flight) are never listed.

Errors: `{"found": false, "error": "Provide exactly one of 'path' or 'article_id'."}`,
`{"found": false, "error": "No document found (or you may not read it)."}`.

---

## docs_asset_link

A **short-lived download URL** for ONE file, by the `attachment_id` `docs_asset_list` gave
you. The bytes go out of band, exactly as a large document does — a 20 MB PDF never
touches the model's context — and the URL is served with **your own** permissions.

Input:
```json
{ "attachment_id": 369846 }
```
Input (the file is shown by a page that does not own it):
```json
{ "attachment_id": 369846, "article_id": 3861 }
```
Output:
```json
{ "attachment_id": 369846, "name": "envoy-flow.svg", "mimetype": "image/svg+xml",
  "size_bytes": 18422, "in_repository": true, "path": "docs/network/envoy-flow.svg",
  "pending_publication": false, "found": true,
  "download_url": "https://<host>/mcp/<db>/<slug>/download/<token>",
  "expires_minutes": 15, "article_id": 3852, "document_path": "docs/network/envoy.md" }
```

The row is the same one `docs_asset_list` returns, plus the URL. **There is no tool that
fetches those bytes for you** — how to get the file down, and how to actually *read* it,
is `workflows/files.md` §2.

`article_id` is only needed for a file **shown** by one page and **owned** by another (the
import deduplicates identical bytes across a documentation). Pass the page you were
reading and the file is served if that page really references it.

Errors: `{"found": false, "error": "Provide the 'attachment_id' of the file
(docs_asset_list)."}` and `{"found": false, "error": "No such file in a documentation you
can read."}` — the second one covers "no such id", "not a documentation file" and "you may
not read the document it belongs to" with the same sentence, on purpose. Don't probe ids:
guessing integers here is how you would find out which ones exist.

---

## docs_sync_status

**Read only, and it is the only thing it is.** It answers **both** directions in one call, and
it is the tool every server has for "where does my work stand": is the documentation in Odoo
still in step with its **Git repository** and which files would arrive (Git → Odoo), and what
has been written in Odoo and is not in the repository yet (Odoo → Git). Cheap: it asks the
platform nothing, clones nothing and writes nothing, so call it as often as you need.

Input:
```json
{ "space": 5 }
```
Output:
```json
{ "repositories": [
  { "repo": "Platform docs", "repo_id": 7, "platform": "gitlab", "access_mode": "managed",
    "sync_mode": "manual", "up_to_date": false,
    "last_sync": "2026-08-14T02:07:11", "last_sync_state": "done", "sync_queued": false,
    "remote_ahead": true, "remote_ahead_since": "2026-08-15T09:30:02",
    "pending_since": "", "last_error": "", "webhook": "active",
    "can_synchronize": false,
    "pending_publication": { "waiting": true, "documents": 2, "files": 1, "refs": ["main"],
      "note": "Held in Odoo and carried by the documentation's next publication — which is scheduled, and which whoever administers the repository can also launch." },
    "incoming": { "known": true, "file_count": 2, "refs": ["main"], "modified_only": true,
      "files": [
        { "path": "docs/setup/install.md", "document": "Installation", "article_id": 1234 },
        { "path": "docs/setup/limits.md" } ] } } ] }
```

How to read it:

- **`pending_publication`** is the **Odoo → Git** half, and the one that answers "did my
  change go anywhere". `waiting` is counted the moment work is written — not when a
  publication first notices it — `documents` / `files` say how much, `refs` on which branch,
  and `note` says what carries it. **A mirror always answers `waiting: false`**, correctly:
  nothing leaves a mirror.
- **`waiting: false` is not "it is live".** Work already handed to a change proposal stops
  counting as pending while the proposal is still open, so this field means *"nothing is
  waiting to be sent"* and never *"it is in the documentation everybody reads"*
  (`pending-work.md` §1).
- **`up_to_date`** answers the **Git → Odoo** direction only: true when nothing is arriving
  and the last read succeeded. It says nothing about work waiting to go the other way, and
  `up_to_date: true` next to `pending_publication.waiting: true` is a normal, consistent pair.
- **`pending_since`** is stamped by the publication and answers *"how long has this gone
  unpublished"* — a backlog clock, not the answer to "is something waiting". Empty is not
  "nothing pending"; read `pending_publication` for that.
- **`incoming`** is what you report of the other direction. `files` names each path and, when Odoo already holds
  that page, the **document** it would rewrite — say *"the Installation page would change"*,
  not *"two files"*. A path with no `document` is a file being **added**, or not a document
  at all (an image, the tree descriptor). `modified_only: false` means files are being added
  or **removed**, and a removal never travels inside the word "changes".
- **`incoming.known: false`** means nothing described the change — read `note`, it says
  whether the repository was simply never read, or moved in a way no push event recorded.
  Do not turn that into "nothing is coming".
- **`sync_mode: "manual"`** is normal, not a fault: those repositories import only when
  somebody asks. `webhook: "none"` / `"failed"` is likewise normal — the repository is
  polled instead, which is slower, not broken.
- **`last_error`** is one redacted line. It is the sentence to relay when a circuit is stuck;
  the full log lives in Odoo.

**You cannot synchronise from here, and neither can any other tool of this MCP.** Reading a
repository into Odoo is a button on the repository's form in Odoo, and it belongs to whoever
administers repositories — `can_synchronize` says whether the person you are talking to is
one of them. When something has to be imported, report what `incoming` holds and who has to
press it. Never offer to do it, and never look for a way around a `can_synchronize: false`:
on a documentation Odoo owns, an import **replaces what Odoo holds**.

**And an import is not what rescues pending work.** It is the other direction. Somebody who
reads `pending_publication.waiting: true` and reaches for a synchronisation is about to
overwrite the very work they were trying to save.

Errors: `{"error": "No documentation repository in scope: …"}` when nothing in scope is
imported from Git or you can read no document of it.

---

## Formulating retrieval queries

The biggest lever on answer quality. Retrieval is hybrid, reranked, cross-lingual — so
phrase for **meaning** and scope on purpose.

Ask full questions, not keyword bags:

| Weak (keyword bag) | Strong (full question) |
| --- | --- |
| `backup frequency retention` | `How often are backups taken and how long are they kept?` |
| `slack webhook port` | `How does the platform notify Slack, and on which port?` |
| `postgres version ram` | `What PostgreSQL version and how much RAM does it require?` |
| `who approves publish` | `Who is allowed to approve and publish a document?` |

Pick the tool for the need:
- **the answer** → `docs_answer` (synthesized + cited).
- **find/browse** the right docs → `docs_search`, then `docs_get`.
- **the structure** → `docs_outline`. **the whole documentation as a file** → `docs_export`.

Scope deliberately: don't guess a `space` — `docs_list_spaces` first, then pass `space`
to narrow. Leave `lang` off unless you truly want to restrict the language.

Iterate with intent (usually one reformulation is enough):
- "not in the sources" but it should exist → raise `max_sources`, drop the
  `space`/`lang` filter, or rephrase around the concept.
- too many off-target hits → add a `space` filter or more specific entities/terms.
- exact string / code / identifier → keyword terms are fine; the hybrid arm matches
  exact tokens too.
