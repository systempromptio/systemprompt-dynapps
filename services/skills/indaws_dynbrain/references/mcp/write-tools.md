# Documentation MCP — write (authoring) tools

The authoring tools (create / edit / move / trash / upload / attach a file), each with input,
output and error shapes. **Shared conventions** (run-as-user, Markdown-canonical, variants,
error shape, tool families) are in `README.md`. For read tools see `read-tools.md`; for
chaining tools together see `workflows/`. What happens to a change after you make it — and why
publishing is not your step — is **`pending-work.md`**.

**Where the document comes from — the one check before any write, and the one that decides
what you say after it.** `docs_metadata` → `repo` / `is_synced` / `can_write`:

- **Native** (no `repo`) — writable, and **that is the whole job**. There is no commit, no
  proposal and no URL: the document only ever lived in Knowledge. Announcing a publication
  step here sends the user looking for something that does not exist.
- **Managed** (`repo` + `can_write: true`) — an imported document whose repository accepts
  writes. You may edit it; the change is live in Odoo and **held pending** until the
  documentation's next publication carries it (`pending-work.md`).
- **Mirror** (`repo` + `can_write: false`) — imported, read-only. The tools refuse it; it
  is edited at its Git source.
- `can_write` already folds all of this in (true for a native doc **or** an imported doc on
  a managed repo), so trust it rather than assuming "imported = read-only". What it does
  *not* tell you is which of the two writable kinds you have — that is `repo`, and it is
  the difference between a finished task and a change nobody has received yet.

The same three cases govern `docs_create`, `docs_update`, `docs_move` and `docs_trash`
alike. The table lives in `SKILL.md`; the reasoning behind it, in `read-tools.md` →
`docs_metadata`.

## Three doors into a document, and each one owns something

A document carries three kinds of data, and every write tool takes all three **as arguments**.
Nobody hand-writes a YAML block to reach any of them any more:

| What | Argument | Examples |
| --- | --- | --- |
| The **title** | `title` | — |
| The documentation's **typed fields** | `metadata` | whatever `docs_metadata_schema` lists |
| Every **other header key** | `front_matter` | `description`, `icon` |

`docs_create`, `docs_create_tree` (**per node**) and `docs_update` all accept `front_matter`
and `metadata`. A pasted `.md` with its own header still works and lands in the same places;
**an explicit argument wins** over the same key inside a pasted header.

**Two refusals fail the WHOLE call**, on a document that belongs to a repository — nothing is
created, nothing is written, and the message names the key that stopped it:

- **A `metadata` key the documentation does not declare.** Nobody writes
  `metadata: {owner: ana}` without meaning to set `owner`, so a caller told "not written" as
  one warning among many could easily miss it. No tool can declare a field: it is added to
  the repository's `knowledge.yml`, in Git, and the error says so rather than pointing at a
  tool that does not exist.
- **A header key outside what the documentation manages.** What Odoo may write into a
  Git-backed header is **`title`, `icon`, `description` plus the declared typed fields** —
  and nothing else. A `weight:` or an `audience:` the descriptor never declared is refused,
  because the published file is composed from that closed set: accepting the key would store
  it in Odoo where it would sit forever and never reach the file.

On a **native** document neither applies. There is no descriptor to answer to, so an
undeclared name comes back as a warning and the header stays as permissive as a `.md` is.

---

## docs_create

Create ONE document under an existing parent you can write. `position` places it
first/last among siblings (default last).

Input:
```json
{ "parent_id": 3879, "title": "Backups",
  "markdown": "# Backups\n\n- Frequency: every 6 hours.\n- Retention: 30 days.\n",
  "front_matter": { "description": "How often we back up and how long we keep it" },
  "metadata": { "status": "draft", "owner": "j.smith" },
  "position": "last" }
```
Output:
```json
{ "article_id": 3884, "title": "Backups", "parent_id": 3879,
  "front_matter": ["description"], "front_matter_warnings": [],
  "metadata": ["owner", "status"], "metadata_warnings": [] }
```
Input (author an English variant):
```json
{ "parent_id": 3879, "title": "Backups", "lang": "en", "markdown": "# Backups\n\n- Every 6 hours, kept 30 days.\n" }
```
Output (with variant):
```json
{ "article_id": 3884, "title": "Backups", "parent_id": 3879,
  "variant": { "lang": "en", "version": "main", "is_default": false } }
```

**`lang` on a create does not make a translation.** Inside a Git documentation the document
is created as the repository's own base row with its canonical file, whatever `lang` you
pass — an addition translates nothing, because there is no text yet to translate — and the
answer reports the variant it actually made. A translation is `docs_update {article_id, lang}`
afterwards, on the same document.

Errors: `{"error":"Provide 'parent_id'."}`, `{"error":"Provide a 'title'."}`,
`{"error":"Parent not found or not accessible."}`, mirrored-parent rejection,
`{"error":"position must be 'first' or 'last'."}`, and the two whole-call refusals above.
A refused call leaves **nothing** behind — not a half-built document, and not a consumed
upload token.

### Creating inside a Git-managed documentation

When the parent belongs to a **managed** repository, the document is created **inside that
repository**: it gets a file, a place in the documentation tree, and it is held **pending
publication**. Nothing about the call changes; what changes is what comes back.

Input (let it choose the file):
```json
{ "parent_id": 3879, "title": "Backups", "markdown": "# Backups\n\nEvery 6 hours.\n" }
```
Output:
```json
{ "article_id": 3884, "title": "Backups", "parent_id": 3879,
  "path": "guides/install/backups.md",
  "source_path": "docs/guides/install/backups.md",
  "repo": "Dyncloud", "in_repository": true, "pending_publication": true,
  "variant": { "lang": "es", "version": "main", "is_default": true } }
```

**`path` — where the file goes.** Optional, **relative to the documentation's own folder**
and ending in `.md`: `guides/backups.md`, never `docs/guides/backups.md`. Omit it and one is
derived — **and the answer always tells you which one it used**, so read `path` back instead
of guessing it. The derivation: the slug of the title, inside the parent's folder
(an `index.md` page shares its folder with its children; any other page gets a folder named
after itself; a section without a file uses its own id as the folder), numbered on collision
(`backups.md`, then `backups-2.md`).

**Refusals, all of them one sentence you can act on:** a `path` with `..`, an absolute
`path`, one that does not end in `.md`, one that repeats the `docs/` prefix, a path already
taken by another document (it tells you which), a `path` passed for a parent that is not in a
Git-managed documentation, and a documentation whose structure is described in a **read-only
format** (mkdocs / RST) — there the file could be committed but never added to the
navigation, so it is refused rather than silently dropped by the next import.

**A title-only create still produces a file** (a `# Title` stub), because a tree entry
pointing at a file that does not exist is removed by the next import.

**The page is live in Odoo and in no repository yet.** Say that when you report back; do not
report it as published (`pending-work.md`).

---

## docs_create_tree

Create a whole subtree (a section with nested pages) in ONE call. `nodes` is a nested list of
`{title, markdown?, content_upload_id?, front_matter?, metadata?, path?, children?}` — the
same doors as `docs_create`, **per node**. Capped at 200 nodes per call. Returns the mirror
tree with the created ids. Best way to lay down a documentation skeleton in one shot.

Input:
```json
{ "parent_id": 3879,
  "nodes": [
    { "title": "Administration guide", "markdown": "# Administration guide",
      "front_matter": { "description": "Everything an administrator has to do" },
      "children": [
        { "title": "Install & requirements", "markdown": "# Install & requirements\n\n- PostgreSQL 15\n- 4 GB RAM\n" },
        { "title": "Backups", "markdown": "# Backups\n\n- Every 6 hours, kept 30 days.\n",
          "metadata": { "status": "draft" } }
      ] } ] }
```
Output:
```json
{ "parent_id": 3879,
  "nodes": [
    { "title": "Administration guide", "article_id": 3881,
      "children": [
        { "title": "Install & requirements", "article_id": 3882, "children": [] },
        { "title": "Backups", "article_id": 3884, "children": [] } ] } ],
  "created": 3 }
```
Notes: `lang`/`version` apply to the WHOLE subtree (one call authors one variant); a node
whose Markdown carried a YAML header, or that passed `front_matter` / `metadata`, reports its
own `front_matter` / `front_matter_warnings` / `metadata_warnings` in its entry. Errors:
missing/empty `nodes`, over-cap (`Too many nodes (N); the cap is 200 — split it.`), a
node with an empty `title`, mirrored parent.

Inside a **managed** documentation the whole subtree is built inside the repository and each
node may carry its own `path` (derived from its title in **its own parent's** folder when
omitted, and reported back). **One refused node rolls back the entire call** — including a
`metadata` key the descriptor does not declare — so you never get half a subtree.

---

## docs_update

Update a document's `title`, content, header and/or typed `metadata`. Provide at least one.
Content (Markdown) replaces the canonical source and re-renders the body.

Input:
```json
{ "article_id": 3882,
  "markdown": "# Install & requirements\n\n- PostgreSQL 15\n- 4 GB RAM\n- Redis 7 for cache\n" }
```
Output:
```json
{ "article_id": 3882, "title": "Install & requirements", "updated": true }
```
Input (fix ONE header key, without resending the body):
```json
{ "article_id": 3882, "front_matter": { "description": "What the platform needs to run" } }
```
Input (edit ONLY the English variant, leave the default untouched):
```json
{ "article_id": 3882, "lang": "en", "markdown": "# Install & requirements\n\n- PostgreSQL 15\n- 4 GB RAM\n" }
```
Errors: `{"error":"Provide 'article_id'."}`, `{"error":"Document not found or not
accessible."}`, `{"error":"Nothing to update: provide 'title', content and/or 'metadata'."}`,
mirrored-document rejection.

On a **managed** document the answer also carries `path`, `source_path` and
`pending_publication: true`. Read that flag rather than assuming: on a native document it
is simply absent, and its absence is what lets you say "done" honestly.

### Setting typed metadata

A documentation declares typed fields in its `knowledge.yml` (status, owner, review date…)
and the values live in each document's header. `metadata` sets them **by name**, in the
same vocabulary the header uses:

```json
{ "article_id": 3882, "metadata": { "status": "approved", "owner": "j.smith" } }
```
```json
{ "article_id": 3882, "title": "Install & requirements", "updated": true,
  "metadata": ["owner", "status"], "metadata_warnings": [] }
```

**Ask `docs_metadata_schema` first, anchored on the document (or on the parent you are
creating under).** It names the fields that apply *there* and gives an `expects` for each — a
relation takes a text (the login, never the display name), a selection takes a KEY and not the
label, a date is `YYYY-MM-DD`, tags and many2many take a **list**.

Four things worth knowing, and the third is the one that surprises people:

- **A field the documentation does not declare fails the whole call.** Nothing is written, not
  even the keys that were fine. That is deliberate — writing half a request and answering
  `updated: true` is worse — and the message names the key and sends you to the
  `knowledge.yml` in Git, because no tool can declare a field.
- **`metadata_warnings` means those fields were NOT written**, even though the answer says
  `updated: true` — a relation, a selection or a tag that does not resolve, a login matching
  nobody **or two people**, a number that is not one. An ambiguous lookup is refused rather
  than guessed: filing a document under a colleague's name is worse than leaving it unowned.
  Pass the list to the user.
- **A `date` and a free-text field are NOT CHECKED.** Whatever you send is stored and
  committed to the customer's file exactly as written. Send `"2026-08-28"` and never a phrase
  like "yesterday": nothing refuses it, nothing warns, and Odoo's own form does not even paint
  a date property holding text — so nobody will notice it either.
- **`false` clears a field**, and a cleared field **leaves the header** instead of sitting
  in it as `null` — a header says a field is empty by not carrying it. Clearing works even on
  a document with no repository to resolve against.

Setting metadata is an edit like any other: on a managed documentation it reaches the file's
header under your name and is pending publication, just as a paragraph would be.

### `front_matter` — the header keys that are not typed fields

```jsonc
docs_update { "article_id": <id>, "front_matter": { "description": "…", "icon": "🚨" } }
```

It writes header keys **without resending the body**, validated and reported like a pasted
header, and a key given there **beats** the same key in a header pasted in the same call. It
is the answer to "I need to fix one line of the header": before it existed, that meant
reposting the whole document.

**On a Git-backed document the set is closed**: `title`, `icon`, `description`, plus whatever
the documentation declares as a typed field. Anything else is refused and the call fails —
`weight`, `audience`, a repository's own convention. That is not a gap: a key Odoo does not
manage belongs to whoever wrote the file, travels through untouched on import, and would
never be written back out of Odoo. On a **native** document there is no such set and anything
goes.

**PER LANGUAGE when you pass `lang`.** The header belongs to the FILE, and a document in two
languages is two files, so a translation carries its own `description` — which is what search
reads. Typed fields are the exception: they belong to the DOCUMENT, are refused from a
translation's header and reported in `metadata_warnings`.

### The icon: a header key, not a title character

The emoji Knowledge paints beside a document is the article's `icon`. It is **not** a
declared property, so `docs_metadata_schema` will never mention it and `metadata` cannot
set it — it travels in the header, next to the title:

```yaml
---
title: "When a rescue pack is NOT sold"
icon: "🚨"
---
```

**Never put the emoji in the `title`.** The title reaches the embedding of every chunk of
the document; an emoji in there is noise in the vector, and it buys nothing that the `icon`
field does not already give. The header key exists so the title stays clean.

Three behaviours worth knowing, because they are not guessable:

- **It is painted on the write.** The column is set from the header there and then, with the
  same rules the import follows, so the tree shows it immediately.
- **It belongs to the DOCUMENT, not to a language.** An `icon:` in a `.en.md` is ignored:
  the article is one record for every language and the base language decides.
- **Absence is silence, not an instruction.** A document whose header carries no `icon:`
  leaves whatever is there alone — which is what stops a synchronisation from wiping every
  emoji anybody set by hand. Removing one is writing `icon: ""`.

### A language is a VARIANT, never a second document

**One `docs_create` per document. The other language is `docs_update {article_id, lang}` on
the SAME `article_id`.**

```jsonc
docs_create { "parent_id": 3879, "title": "Backups", "markdown": "# Backups\n" }
//  -> { "article_id": 3884, "path": "guides/install/backups.md", … }   // the default variant

docs_update { "article_id": 3884, "lang": "en", "markdown": "<the same page, in English>" }
//  -> { "article_id": 3884, "updated": true,
//       "variant": { "lang": "en", "version": "main", "is_default": false },
//       "source_path": "docs/guides/install/backups.en.md" }
```

Calling `docs_create` twice gives you **two unrelated documents** — two titles, two paths,
two entries in the tree, two files — and nothing will warn you: both calls succeed. Neither
document knows about the other, the language switcher shows nothing, and the repository ends
up with a duplicate somebody has to merge by hand. This is the single easiest mistake to make
with these tools, and passing `lang` to `docs_create` does **not** avoid it: inside a
repository that call gives you the base row, not a translation.

The variant's row is created on the spot, with the file it will be committed to
(`source_path` in the answer). A documentation that keeps **one file per document** (no
language layout configured) refuses the second language rather than let two texts claim the
same file — say so and let the user configure the documentation.

### What belongs to the variant, and what belongs to the document

This is the line that decides every multilingual write:

| | Belongs to | Set with |
| --- | --- | --- |
| `title`, `description` and the rest of the header | the **VARIANT** — the file | `docs_update {lang, title}` / `{lang, front_matter}` |
| The documentation's **typed fields** (`doc_type`, `territory`, `categories`, `owner`, …) | the **DOCUMENT** | `docs_update {metadata}` — **without `lang`** |

Two files, two headers; one article, one place for the typed fields. So a translation carries
its own title and description, and a typed field sent inside a translation is **refused and
reported** in `metadata_warnings` — correctly, because there is a single record to write it
to and the last language edited would otherwise win.

```jsonc
docs_update { "article_id": 3884, "lang": "en", "markdown": "<the English body>" }
docs_update { "article_id": 3884, "lang": "en", "front_matter": { "description": "…" } }
docs_update { "article_id": 3884, "metadata": { "owner": "j.smith" } }   // no lang: the document
```

`docs_update {lang, title}` names **that translation**, and the answer reports **that
translation's file** — the same call without `lang` renames the document. A translation that
declares no title of its own gets one **derived from its first heading**, the way the import
does it.

**A translation with no `description` is not broken — it is the gap you are there to fill.**
There is deliberately no inheritance: the description is what the search index reads, so
lending it the original's would make the page look described while being unfindable in its own
language. An empty one is the honest state, and filling it is the job.

---

## docs_move

Re-parent a document under a different parent — **this is how a documentation gets
reorganised**. Both ends must be writable: a document of a `managed` repository moves fine,
only a `mirror` is refused. Cannot move a document under itself or its own descendant.
`position` optionally places it first/last among the new siblings.

Input:
```json
{ "article_id": 3886, "new_parent_id": 3879, "position": "last" }
```
Output (a `knowledge` documentation):
```json
{ "article_id": 3886, "new_parent_id": 3879, "moved": true, "repo_id": 4,
  "publish_required": true,
  "next_step": "\"Backups\" was moved inside the \"Dyncloud docs\" documentation in Odoo only. Its file has NOT moved … the new tree is held PENDING publication, and reaches the repository with the documentation's next publication — which is scheduled, and which whoever administers the repository can also launch." }
```
Errors: missing ids, either end not writable, moving under self/descendant.

Two consequences on a repository, and the second one is format-dependent:

- **The move does not rename the file.** It changes where the page sits in the
  documentation's descriptor; the folder and the file name stay as they were. Say so when a
  user expects the path to follow the page — the `.md` staying put is by design, and every
  reader of the documentation sees the new place.
- **Only a `knowledge` documentation carries a move back to Git.** On `mkdocs` or `rst`,
  `publish_required` is **false** and the `next_step` says the move stays in Odoo,
  permanently: those formats have no descriptor tree a publication can write a move into.
  Read the pair rather than assuming (`pending-work.md` §2).

---

## docs_trash

Send a document to the Knowledge trash — a **soft, reversible** delete (there is
no hard delete via the MCP; it can be restored within the retention window). Trashing a
document also trashes its writable descendants. A trashed document is de-indexed, so it
stops appearing in `docs_search` / `docs_answer` immediately.

Input:
```json
{ "article_id": 3886 }
```
Output (a **native** document — nothing else to do):
```json
{ "article_id": 3886, "trashed": true }
```
Output (a **`knowledge`** documentation — the file is still in the repository):
```json
{ "article_id": 3886, "trashed": true, "repo_id": 4, "publish_required": true,
  "next_step": "\"Old install guide\" was removed from the \"Dyncloud docs\" documentation in Odoo. The file(s) are still in the repository: the removal is held PENDING publication, and reaches the repository with the documentation's next publication — which is scheduled, and which whoever administers the repository can also launch." }
```
Output (a **`mkdocs`/`rst`** documentation — the file is never leaving):
```json
{ "article_id": 3886, "trashed": true, "repo_id": 4, "publish_required": false,
  "next_step": "… removed from the documentation in Odoo only. The file(s) are still in the repository and stay there: this repository publishes from 'mkdocs' files … Remove the file(s) in Git directly if they should leave the repository too." }
```
Errors: missing `article_id`, not found / no access, mirrored-document rejection.

**`trashed: true` is not "deleted from the documentation".** On a repository it means the
page left Knowledge while its `.md` stayed exactly where it was — the reader on the website
still finds it. On a `knowledge` documentation the removal is pending; on any other format
**nothing will ever remove it** and saying "it will go with the next publication" is a
promise the repository cannot keep. Read `publish_required` and `next_step` before you report
anything.

---

## docs_request_upload

Issue a one-time, short-lived upload URL so a **large** Markdown body goes straight to
the server out-of-band, without its bytes crossing the model channel. Use before
`docs_create`/`docs_update`/`docs_create_tree` when the Markdown is big (> ~48 KB); for
small content just pass `markdown` inline.

Input:
```json
{}
```
Output:
```json
{ "upload_url": "https://<host>/mcp/<db>/<slug>/upload/<token>",
  "upload_id": 264, "expires_minutes": 15,
  "instructions": "POST the .md file to upload_url (multipart field 'file'); the response returns {attachment_id}. Pass that id as content_upload_id." }
```

Full flow:
```bash
# 1) docs_request_upload  ->  {upload_url, ...}
# 2) POST the .md (multipart field 'file'):
curl -s -X POST "<upload_url>" -F "file=@big.md;type=text/markdown"
#    -> {"ok": true, "attachment_id": 369846, "bytes": 12345}
# 3) docs_create / docs_update / docs_create_tree with  "content_upload_id": 369846
```
The URL is presigned and expires (~15 min). The `attachment_id` is single-use for that
create/update — but a call that is **refused** does not burn it: the whole write unwinds
together, so you can fix the argument and retry with the same id.

For a **file** — an image, a diagram, a PDF — the tool is `docs_asset_upload` below, not
this one: it takes a real file name and ties the file to a document. This one only ever
carries the Markdown body of a document.

---

## docs_asset_upload

Add a **file** to a document — an image, a diagram, a PDF, a spreadsheet — in **two
calls**, because the bytes cannot come through this channel any more than a 20 MB PDF
could. The first call issues a one-time upload URL for the file name and the document; the
file is POSTed straight to it; the second call passes the id back and **that** is when the
file becomes a file of the document.

**Call 1 — ask for the URL.** `article_id` (or `path`) and `filename` are both required.

Input:
```json
{ "article_id": 3852, "filename": "network-diagram.svg" }
```
Output:
```json
{ "attached": false,
  "upload_url": "https://<host>/mcp/<db>/<slug>/upload/<token>",
  "upload_id": 271, "filename": "network-diagram.svg", "max_bytes": 26214400,
  "expires_minutes": 15, "article_id": 3852, "document_path": "docs/network/envoy.md",
  "instructions": "POST the file to upload_url (multipart field 'file'); the response returns {attachment_id}. Then call docs_asset_upload again with that attachment_id and the same article_id to attach it to the document." }
```
**`attached: false` here is not a failure** — it is the honest answer to "is the file on
the document yet", and it stays false until call 2. Read `filename` back: the name is
sanitised (accents folded, anything outside `[A-Za-z0-9._-]` becomes a hyphen, any folder
in it dropped), and the sanitised name is what the link in the Markdown will have to say.

**The POST** (whoever does it — see `workflows/files.md` for each environment):
```bash
curl -s -F "file=@network-diagram.svg" "<upload_url>"
#  -> {"ok": true, "attachment_id": 369846, "bytes": 18422}
```

**Call 2 — attach it.** Same `article_id`, plus that `attachment_id`.

Input:
```json
{ "article_id": 3852, "attachment_id": 369846 }
```
Output (a document of a **managed** repository):
```json
{ "attached": true, "article_id": 3852, "document_path": "docs/network/envoy.md",
  "title": "Envoy Gateway",
  "variant": { "lang": "es", "version": "main", "is_default": true },
  "file": { "attachment_id": 369846, "name": "network-diagram.svg",
            "mimetype": "image/svg+xml", "size_bytes": 18422,
            "in_repository": false, "path": "", "pending_publication": true },
  "message": "The file is attached to the document, next to the page it belongs to: the file is held PENDING publication, and reaches the repository with the documentation's next publication … For it to be VISIBLE in the documentation, the page must reference it: write \"![](network-diagram.svg)\" (or a link) in the Markdown with docs_update." }
```
Output (a **native** document — nothing to publish):
```json
{ "attached": true, "article_id": 3884, "document_path": "Meeting notes",
  "title": "Meeting notes",
  "file": { "attachment_id": 369901, "name": "minutes.pdf", "mimetype": "application/pdf",
            "size_bytes": 240118, "in_repository": false, "path": "",
            "pending_publication": false },
  "message": "This document is not kept in a Git repository, so there is nothing to publish: the file is attached to the document in Odoo and is served from there." }
```
The `message` is composed **for the server you are on** — it names a tool only if you have
it — so read it out rather than substituting a step of your own. `pending_publication` in
`file` is the flag, not your memory of which kind of document it was.

**`variant` is the row the file actually landed on**, which is not always the one you
asked for: a file attached while reading a language the document does not carry lands on
its default row. Read it back.

**Two rules that decide whether the file is any use.**

- **The upload does not make it visible.** A file attached and never referenced is a file
  in the repository nobody sees. Make the page point at it with `docs_update` —
  `![Network diagram](network-diagram.svg)` for an image, `[Manual](manual.pdf)` for a document.
  Publication moves the file; it does not write that link.
- **A relative link by NAME, never the attachment id.** The file is committed next to the
  page that links it, so `manual.pdf` reads the same in Odoo, in GitLab's file viewer and
  in any Markdown editor. An `/web/content/369846` written into the prose is a link that
  works in exactly one of those places.

**What is refused, and in what words.** Two rules only — a documentation carries whatever
a documentation carries, so there is no allow-list of types:

- **25 MB** (`max_bytes: 26214400`): `The file "x.pdf" cannot be attached. The file is
  31.4 MB; the limit for a documentation file is 25 MB.` A blob in a Git history is
  permanent and every clone pays for it forever.
- **Executables**, by extension *and* by first bytes — `payload.exe` renamed `manual.pdf`
  is one keystroke. Refused on the name alone in **call 1**
  (`"install.sh" cannot be published to a documentation repository (executable files never
  are).`), and on the bytes in call 2.
- An empty file: `The file "x.pdf" cannot be attached. The file is empty.`

**An SVG that carries anything runnable is stored and published — but not labelled as an
image.** Script tags, `on…=` handlers, `javascript:`, `<foreignObject>`/`<iframe>`,
declared entities: any of those and the file keeps a neutral mimetype, which means **it
will not render inline** anywhere. Nothing is lost — it is stored and downloadable — but
if somebody uploads a diagram and it does not show, this is the reason and it is worth
saying: an SVG exported by some drawing tools embeds handlers without asking.

**Errors of the second call.** `{"attached": false, "error": "'content_upload_id' is not
an upload of this server: pass the 'attachment_id' returned by POSTing your .md to a
docs_request_upload URL."}` — the shared ownership check, so its wording names
`content_upload_id` even in the file flow. It means the id is not one **this server**
recorded for **you**: a URL that expired, an upload that never happened, or an id from
somewhere else. Only an attachment born of one of these upload tokens is accepted, and
that is deliberate — otherwise a wrong integer could move an invoice's XML into a client's
documentation, where the next publication would commit it. Redo call 1 rather than
retrying with the same id.

---

## Pasting a whole `.md` file (its YAML header)

Somebody opens the `.md` in their editor, selects all and pastes it — header included.
That is safe, and it behaves in a way you have to be able to explain. It applies to
`docs_create`, `docs_create_tree` (per node) and `docs_update` alike. **It is no longer the
only door**: `front_matter` and `metadata` reach the same layers, and are clearer.

- **The header is split off, never stored as body.** Content that opens with a `---` YAML
  block has it parsed out and **merged into the document's own header**. Stored as
  body it would come back doubled: the file committed to the client's repository would grow
  a second `---` block on top of the one it already had. A `---` that is a horizontal rule,
  or a header too broken to parse, is left untouched as ordinary text.
- **A declared field inside a pasted header IS written** as a typed field, under the same
  rules as `metadata` — it is not rejected, and it is not merely a header key. So a paste
  can change `owner` without anybody typing `metadata`: read `metadata` in the answer to see
  what moved. The one place a typed field IS refused is the header of a **translation**
  (`metadata_warnings` says so): typed fields belong to the document, not to one of its
  languages.
- **`front_matter` lists the keys that were absorbed** (`["description", "title"]`), so
  you can see the header did not silently become prose. Merging, not replacing: keys you
  did not send stay as they were.
- **`front_matter_warnings` lists what did NOT make it.** A value the schema cannot read
  (`weight: alto`, an unparseable date) is dropped rather than written as a value nobody
  typed — and it is absent from `front_matter` precisely because it was dropped. **Pass
  these on to the user in their own words**: they are the one thing the answer's
  `updated: true` would otherwise hide.
- **A key the documentation does not manage fails the call** on a Git-backed document (see
  "Three doors" at the top). A pasted file from a repository that declares `weight` is fine;
  the same header pasted onto a documentation that does not declare it is refused, naming the
  key.
- **A `title` in the header renames the document** when you pass no explicit `title`
  argument. It has to: that header is what gets committed, and the next import names the
  page from it. Pass `title` explicitly when you want the document to keep its current
  name — an explicit `title` wins in **both** places, header included. Either way, **say the
  new name out loud** if it changed; a page that renames itself costs trust.

## Authoring so the document gets indexed (RAG)

If the system has RAG search configured, a document you create or edit should be picked
up and (re)indexed **automatically** — *provided you author it in the state the indexer
expects*. Whether RAG is on and how it's configured is a Knowledge-side setting you don't
control; what you DO control is producing a document that would be indexed correctly if
it is. So split it:

**What you control (do these):**

- **Author in Markdown, with real content.** The write tools store your `markdown` as
  the document's canonical source and type the document as *Markdown*. That Markdown
  typing is exactly what a standard markdown-scoped eligibility rule matches — so always
  pass real Markdown (inline `markdown`, or `content_upload_id` for large bodies). Don't
  create an empty document and expect it to be searchable: no content → no chunks.
- **Structure it with headings.** The indexer splits a document into chunks by its
  heading structure and uses the heading path as each chunk's retrieval context. Clear
  `#` / `##` / `###` sections, one idea per section, beat a wall of text — better chunks,
  better retrieval, cleaner citations. Put the key fact near its heading.
- **Give every language its own `description`.** It is what the index reads for that
  language; a translation without one is a page nobody finds in the language it was
  written for.
- **Edit content through `docs_update`.** Replacing the Markdown re-marks the document
  for re-indexing, so an edit re-indexes automatically (same path as a create). Editing
  only the title does not change the indexed content.
- **Writable documents only.** Native documents, and imported ones on a **managed**
  repository. A **mirror** is read-only and indexed from its source.

**What the system controls (you cannot force):** whether RAG is configured at all, the
eligibility rules, and the schedule. Indexing is **asynchronous** — it runs on the system's
jobs, not on your write — so a fresh document is usually searchable a short while later, not
instantly. There is no MCP tool to trigger it.

**Verify, don't thrash:** after a create/edit, check `docs_metadata` → `indexed` /
`chunk_count`, then confirm with a `docs_search` for a distinctive phrase. If it stays
`indexed: false` after the system has had time to run, that is an eligibility/configuration
matter on the Knowledge side — say so and let the user check it; don't re-create or re-edit
in a loop.

## After writing (housekeeping)

- Confirm placement with `docs_metadata` (right parent, `can_write`, `is_synced`), and the
  variant with `docs_metadata {article_id, lang}` — the echo of a write is not proof.
- To reorganize afterwards: `docs_move`. To retire: `docs_trash` (reversible, de-indexes
  at once).
- Say where the work stands: saved in Odoo, and — on a repository — pending publication.
  `pending-work.md` has the sentence.
