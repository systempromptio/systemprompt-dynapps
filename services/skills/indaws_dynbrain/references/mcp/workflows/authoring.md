# Workflows — create, edit, reorganize, retire

Everything that writes. The one check that opens all of these is the document's **origin**
(`docs_metadata` → `repo` / `can_write`): it decides whether you may write at all, and what
is true to say when the call returns. The table is in `../../../SKILL.md`; tool details in
`../write-tools.md`; the index of every workflow is `README.md`.

**No recipe here ends in a publication step.** What happens to a change afterwards, and who
carries it, is `pending.md` and `../pending-work.md`.

---

## 1. Author a whole section

Classify the type, confirm you can write, learn the fields, lay the skeleton down in one call.

```text
# 1. Read the layout from Knowledge (anchors in ../../knowledge-as-source.md); stop if none resolves
docs_metadata { article_id: 3879 }             # parent writable?  can_write:true
docs_metadata_schema { article_id: 3879 }      # the fields THAT PARENT declares, and their `expects`
docs_create_tree {
  parent_id: 3879,
  nodes: [ { title: "Administration guide", markdown: "# Administration guide",
             front_matter: { description: "Everything an administrator has to do" },
             children: [ { title: "Install & requirements", markdown: "# …\n- PostgreSQL 15\n- 4 GB RAM\n" },
                         { title: "Backups", markdown: "# Backups\n- Every 6h, kept 30 days.\n",
                           metadata: { status: "draft" } } ] } ]
}
  → returns the created ids
docs_metadata { article_ids: [3881, 3882, 3884] }   # confirm placement + what actually landed
```

**The schema call is not optional and it is not ceremony.** Fields are declared per section,
so asking the repository loses whatever that section adds — and a `metadata` key the
descriptor does not declare **fails the whole call**, rolling back the entire subtree. One
bad key, no section. Add a single page later with `docs_create { parent_id, title, markdown,
front_matter, metadata }`; for a large body, swap inline `markdown` for the upload flow (§6).

**Read `in_repository` / `pending_publication` in the answer before you report back.** A
native parent means the section is live as soon as it is created. A parent that belongs to a
managed documentation means every page you just created is inside that repository and pending
publication — say that, don't call it published (`pending.md` §1).

**So it gets indexed:** author real, heading-structured Markdown, and give every language its
own `description`. Indexing is automatic but **asynchronous** — re-check `docs_metadata` →
`indexed` / `chunk_count` a bit later, then `docs_search` a distinctive phrase to confirm.
You can't trigger indexing from the MCP (`troubleshooting.md` §3). Full rules in
`../write-tools.md` → "Authoring so the document gets indexed (RAG)".

---

## 2. Edit an existing document

Always check the origin first with `docs_metadata` → `repo` / `can_write` / `is_synced`.

```text
docs_metadata { article_id: 3882 }             # can_write:true → OK to edit
docs_update { article_id: 3882, markdown: "# Install & requirements\n- PostgreSQL 15\n- 4 GB RAM\n- Redis 7 for cache\n" }
  → { updated: true, path: "docs/install.md", pending_publication: true }
```
`can_write:false` with `is_synced:true` → the document is imported **and** its repository
is `mirror` (read-only): stop and tell the user to edit it at its Git source. If
`can_write:true` **with** `is_synced:true`, the repo is `managed` — you may edit it, the
change is saved in Odoo and it is pending publication. On a native document
`pending_publication` is simply absent and the edit is the whole job.

A second **language** is `docs_update {article_id, lang}` on the SAME document, never a
second `docs_create` — see `../write-tools.md`, where that mistake has its own section. And
`docs_create {lang}` inside a repository is not the shortcut it looks like: it creates the
document in the repository's base row, not a translation.

---

## 3. Reorganize: move a document

`docs_move` re-parents a document. **It is not limited to native documents**: a document of
a `managed` repository moves fine, and reorganizing a Git documentation from Odoo is a normal
task. Both ends must be writable — a `mirror` is refused, as always.

```text
docs_metadata { article_id: 3886 }                                  # origin of the document
docs_metadata { article_id: 3879 }                                  # and of the new parent
docs_move { article_id: 3886, new_parent_id: 3879, position: "last" }
  → { moved: true, repo_id: 4, publish_required: true, next_step: "…" }
```

Two things about a move inside a repository that surprise people:

- **The file is not renamed.** The move changes where the page sits in the documentation's
  descriptor (`knowledge.yml`); its folder and file name stay exactly as they were. Worth
  saying out loud when the user expects to see the path change — every reader of the
  documentation sees the new place regardless.
- **Whether it ever reaches Git depends on the FORMAT**, and the answer says so.
  `publish_required: true` on a `knowledge` documentation → pending, and the next publication
  carries it. `publish_required: false` on `mkdocs`/`rst` → the move lives in Odoo and
  **nothing will ever carry it to Git** (`pending.md` §2). Read the boolean; do not assume.

Cannot move a document under itself or its own descendant; `position` optionally places it
first or last among the new siblings.

---

## 4. Retire a page (and what the format decides)

The most silently broken task there is: `docs_trash` answers `trashed: true`, the page
disappears from Knowledge and stops appearing in search at once, and the `.md` **stays in the
client's repository** — their readers keep finding it. What happens next is not the same
everywhere, and the answer tells you which case you are in.

```text
docs_metadata { article_id: 3886 }        # repo:"Dyncloud docs", can_write:true → MANAGED
docs_trash    { article_id: 3886 }
```

**A `knowledge` documentation** — the removal is real, and waiting:
```text
  → { trashed:true, repo_id:4, publish_required:true,
      next_step:"… The file(s) are still in the repository: the removal is held PENDING
                 publication, and reaches the repository with the documentation's next
                 publication — which is scheduled …" }
```
> "It is out of the documentation in Odoo now, so nobody finds it there any more. The file
> leaves the repository with the next publication."

**A `mkdocs` or `rst` documentation** — the removal never leaves Odoo:
```text
  → { trashed:true, repo_id:4, publish_required:false,
      next_step:"… removed in Odoo only. The file(s) are still in the repository and stay
                 there … Remove the file(s) in Git directly if they should leave the
                 repository too." }
```
> "It is out of the documentation in Odoo. The file stays in the repository, and nothing here
> will remove it — somebody has to delete it in Git."

That second case is the expensive one: the page vanishes from search while its file sits in
the client's repository forever, and the next import can even bring the page back. Reading
`publish_required` is what tells the two apart.

**On a native document there is no second act.** No `repo` in `docs_metadata`, no
`publish_required` in the answer: `trashed: true` is the whole story and "done, it is retired"
is the right thing to say.

Trashing is reversible (Knowledge trash, retention window) — but **there is no `restore`
tool**: bringing a document back is an ORM job, not one of these.

---

## 5. Set the header and the typed fields (without resending the body)

Three arguments, three layers, and the order that keeps them from failing:

```text
docs_metadata_schema { article_id: 3882 }
  → status (selection: draft|approved), owner (many2one, expects res.users.login),
    reviewed_on (date)
docs_update { article_id: 3882,
              front_matter: { description: "What the platform needs to run", icon: "🔧" },
              metadata: { status: "approved", owner: "j.smith", reviewed_on: "2026-08-28" } }
  → { updated: true, front_matter: ["description","icon"], front_matter_warnings: [],
      metadata: ["owner","reviewed_on","status"], metadata_warnings: [] }
```

Four things to carry:

- **The schema call comes first** because a `metadata` key the descriptor does not declare
  **fails the whole call** — the good keys do not land either. And no tool can add a field:
  it goes into the `knowledge.yml` in Git.
- **The header set is closed on a Git document.** `title`, `icon`, `description`, plus the
  declared typed fields. A `weight` or an `audience` the documentation does not declare is
  refused, whatever any example shows.
- **`reviewed_on` is not checked.** A date and a free-text field are stored and committed
  exactly as sent — write `2026-08-28`, never "yesterday". Nothing will refuse it and Odoo's
  form will not even paint it.
- **Read both `*_warnings`.** Green answers hide refused keys, and they are the only trace.

A pasted `.md` still works and goes to the same places, with one thing to voice: **a `title:`
in the header renames the document** when you pass no explicit `title` argument. Say the new
name out loud. And a declared field inside that header **does** get written as a typed field
— read `metadata` in the answer to see what the paste moved.

---

## 6. Move large content out-of-band

Big documents never go through the model channel — read and write them via presigned URLs.

Read a large document:
```text
docs_get { article_id: 4120 }
  → returns markdown_url (size_bytes 75625), no inline markdown
curl -s "<markdown_url>"                        # fetch out-of-band; read selectively
```
Write a large document:
```text
docs_request_upload {}                          # → {upload_url, ...}
# curl -s -X POST "<upload_url>" -F "file=@big.md;type=text/markdown"  → {attachment_id: 369846}
docs_create { parent_id: 3879, title: "Full install guide", content_upload_id: 369846 }
```
Grab a whole documentation at once:
```text
docs_export { space: 5, format: "full" }        # or "index" for a TOC
  → download_url ; curl -s "<download_url>"
```
Why: keeps your context small; the URLs are short-lived and served with your own
permissions. If the create is **refused** — a bad `metadata` key, say — the upload id is not
burned: fix the argument and call again with the same one.

**These two tools carry Markdown only.** A real file — an image, a diagram, a PDF, a
spreadsheet — is `docs_asset_upload` and has its own recipe, including how the bytes move in
each environment: `files.md`.

---

## 7. Link one document to another

A cross-reference between pages is written **exactly like a file link: a relative path to
the other page's `.md`**. That single spelling reads the same in Odoo, in the platform's
file viewer and in any Markdown editor, and it is the only one the documentation can
resolve at both ends.

```text
# from docs/guias/instalacion.md
[copias de seguridad](backups.md)          # a sibling page
[volver al índice](../index.md)            # up a level
```

The renderer turns that path into the document in Odoo; the file keeps the relative path in
Git. **Never paste the Odoo URL** (`/odoo/knowledge/article/<id>`) into the prose: it looks
right on screen, and in the repository it is a dead reference to a database nobody reading
the file has.

**Across documentations.** When several documentations share one repository, a link that
leaves your own docs folder is resolved from the **repository root**, so it works too:

```text
# from <docs_dir>/<section>/<file>.md
[el contrato del chart](../../../platform/docs/01-contratos/chart-values.md)
```

Two behaviours nobody tells you about, and both matter:

- **Moving or renaming a page rewrites the links that point at it.** Automatically, in the
  same pass, in every document that referenced it. You do not hunt them down, and you should
  not "fix" them by hand beforehand — you would be fixing what the pass is about to fix, and
  your edit is what would end up wrong.
- **A link this change would break stops the WHOLE publication**, and nothing is committed.
  That is the system refusing to publish a documentation it has just broken, and it is the
  right answer — the alternative is a reader finding the hole. Nothing is lost and the work
  stays pending; what has to change is the content. **You will probably not see the message**
  (it is worded by a publication a caller runs, and you may not have that tool), so when
  something you retired or moved stays pending for far longer than a publication cycle, this
  is the first thing to suspect: `troubleshooting.md` §2. A link that was **already** broken
  before your change is reported and blocks nothing.
