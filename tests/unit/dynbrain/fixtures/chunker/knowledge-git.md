# Dynapps Knowledge

Imports **mkdocs-style Git documentation repositories** (GitHub / GitLab) into Odoo's
Knowledge app as living `knowledge.article` records, and exposes them to LLMs through a
"magic" `/llms.txt` URL.

This is the **core** module of the Dynapps Knowledge suite. It imports the documentation
_and_ hosts the shared AI provider layer that the rest of the suite uses for embeddings,
reranking and synthesis:

| Module                                                    | Role                                                         |
| --------------------------------------------------------- | ------------------------------------------------------------ |
| [dynapps_knowledge_base](../dynapps_knowledge_base)       | Canonical Markdown + neutral indexing interface (foundation) |
| **dynapps_knowledge** (this)                              | Git import → articles, `/llms.txt`, shared AI provider layer |
| [dynapps_knowledge_rag](../dynapps_knowledge_rag)         | Semantic RAG search (pgvector)                               |
| [dynapps_knowledge_rst](../dynapps_knowledge_rst)         | reStructuredText / Sphinx import                             |
| [dynapps_ai_mcp_knowledge](../dynapps_ai_mcp_knowledge)   | MCP tools for assistants                                     |
| [dynapps_knowledge_website](../dynapps_knowledge_website) | Public publishing                                            |
| [dynapps_knowledge_mermaid](../dynapps_knowledge_mermaid) | Mermaid diagrams in reader + editor                          |

## What it does

- **Reads Git repos** structured like [mkdocs](https://www.mkdocs.org/): honours
  `mkdocs.yml` `nav:` or falls back to the folder tree, front matter, `index.md` home
  pages.
- **Versions**: single ref, or one space per branch / tag.
- **i18n**: language variants by suffix (`page.es.md`) or folder (`es/page.md`), with a
  `default_lang` and language fallback.
- **Markdown → HTML** for reading in Odoo, rewriting `.md` links to article links and
  images to `ir.attachment`.
- **Front matter** validation (description / categories / version / language), with a
  deterministic `description` fallback (first prose paragraph).
- **Read-only articles**: imported articles are guarded against manual edits (unless the
  write-back module is installed).
- **Idempotent sync** keyed by `(space, source_path)`, skipping unchanged commits.
- **Automation**: a cron and GitHub / GitLab **webhooks** keep articles in sync.
- **Security**: repository Git tokens are encrypted at rest (Fernet).
- **`/llms.txt`**: `/llms.txt`, `/llms-full.txt` and a `.md` view per article, following
  the [llms.txt](https://llmstxt.org/) spec, with caching. The shared cache is served
  only to an internal user who may read every article of the space, and only while it
  holds exactly the articles the space has now, as last written (an edit by hand between
  two syncs invalidates it); anyone else gets a render of what they may read. The cache
  fields are readable by the system group only.
- **Repository buttons** (the Git ones plus check links, check fidelity, generate
  structure, preview import, restore from Git) are refused to anyone but a Repository
  Manager before they touch Git, the chatter or the repository, over RPC too.
- **Write-back stays inside the docs folder**: Odoo writes documents and files inside
  the docs folder only, plus the descriptor (its `tree:`, or a generated one where the
  branch has none). A route, a recorded path or a file name with `..`, a leading `/` or
  a backslash is refused on the record for every writer, and every file the write-back
  writes, moves or stages is checked again (symlinks resolved) before it is written. A
  path outside the docs folder stops the publication: nothing is pushed. Where a row or
  a file publishes (`git_path`, `dynapps_repo_path`) is recorded only by the import and
  the publication (`asset_rules.path_writer()`) or the superuser, never on a context
  flag.

## Key models

| Model                         | Purpose                                                                                                                         |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `dynapps.knowledge.repo`      | Repository config + sync orchestrator                                                                                           |
| `dynapps.knowledge.space`     | One documentation space (per version / language)                                                                                |
| `dynapps.knowledge.version`   | A branch / tag version                                                                                                          |
| `knowledge.article` (inherit) | Adds `dynapps_repo_id`, `dynapps_space_id`, `dynapps_lang`, `dynapps_source_path`, `dynapps_markdown`, `dynapps_description`, … |

## Shared AI provider layer

This module also carries the suite's **AI provider registry** (`tools/ai/`), a single
pluggable layer for three capabilities — `chat`, `embed`, `rerank` — consumed by the RAG
engine and the MCP module:

| Provider                                                            | Capabilities  | Notable defaults                                                |
| ------------------------------------------------------------------- | ------------- | --------------------------------------------------------------- |
| **OpenAI** (and OpenAI-compatible: Azure, OpenRouter, Ollama, vLLM) | chat, embed   | `text-embedding-3-large`, `gpt-4o-mini` (cheap: `gpt-4.1-nano`) |
| **Anthropic (Claude)**                                              | chat          | `claude-opus-4-8` (cheap: `claude-haiku-5-5`)                   |
| **Cohere**                                                          | embed, rerank | `embed-v4.0`, `rerank-v3.5`                                     |
| **Voyage**                                                          | rerank        | `rerank-2.5`                                                    |
| **ZeroEntropy**                                                     | rerank        | `zerank-2`                                                      |

A provider is a class: declaring it registers it (`Provider.__init_subclass__`), so
adding a vendor is one file, no `if provider ==` chain and no table to keep in sync.
Each capability can be routed to a different vendor and each has its own `base_url`
override, which is what makes the OpenAI-compatible family reach Azure / Ollama / vLLM
as **configuration, not code**. `dynapps.ai.config` resolves
`(provider, key, model, base_url)` per capability, with legacy-parameter fallback.

The reranking side (`zerank-2`, Cohere, Voyage) and the embedding/synopsis defaults
serve the RAG engine, whose retrieval pipeline is adapted from the open-source
[gbrain](https://github.com/garrytan/gbrain) (MIT) — see that module's README.

## Configuration

Knowledge ▸ Configuration ▸ **Git Documentation** (developer mode) → **Repositories**.
Create a repository, set the clone URL and auth, then **Synchronize**. AI providers/keys
live under the same Settings page (one key per capability, or a shared key per vendor).

## Known issue — diagrams written as inline `<svg>`

**Write a diagram as a `.svg` FILE next to the document and link it.** An inline `<svg>`
inside the Markdown works, but it takes a detour that has cost us three separate
defects, and one of them is still open.

Why the detour exists: the rendered body is a sanitised `fields.Html`, and the sanitiser
keeps the `<svg>` wrapper while deleting the drawing inside it — measured on article
3832, a 15 KB diagram came out as **99 characters, no `viewBox`, not one child
element**. So the importer extracts the inline diagram into an `ir.attachment` and
points an `<img>` at it, which survives sanitisation and renders everywhere.

The extraction receives markup that has been through an **HTML** parser, and an SVG file
is read as **XML**. Two consequences, both fixed, both invisible to anything short of a
browser (the attachment is created, the mimetype is right, the route answers 200 and
`curl` prints a perfectly good drawing):

- no `xmlns` → the browser does not parse it at all (`naturalWidth` 0);
- attribute names lowercased → `viewBox` became `viewbox`, so the drawing declared no
  view box and rendered at the default 300×150.

**Still open:** the extraction only happens on IMPORT. A re-render — which is what a
save in Odoo does — resolves diagrams from the row's cached asset map by the sha1 of the
markup, and never creates an attachment (deliberately: opening an editor must not
manufacture files). So an inline `<svg>` **added or edited inside Odoo** finds no entry,
is not converted, and the sanitiser empties it. The page shows a hole until the change
reaches Git and comes back through an import.

Fixing that by creating attachments during a re-render was considered and rejected: a
save would then manufacture files, and every edit of a diagram would leave another
orphan behind. So the rule at the top of this section is the answer, and **Check links**
names the documents that do not follow it (path and count, in the synchronization log)
so that standing on this is at least visible. See `tools/svg_tools.py`.

## Linking to another documentation

Two shapes, and which one to write depends on a fact about the repositories, not about
the documentations.

**Same Git repository** (several documentations under different docs folders, the way
Dyncloud publishes _Trust center_, _Base_ and _Platform_ out of one repo): write the
ordinary relative path — `../../base/docs/instalacion.md`. It is valid in the checkout,
on GitLab and in any editor, and Odoo resolves it too.

**Different repositories**: there is no relative path that reaches, so name the document
instead:

```markdown
Ver la [guía de instalación](/docs/go/dyncloud/plataforma/instalacion).
```

`/docs/go/<slug>/<path>` — the documentation's **Link slug** (Knowledge ▸ Configuration
▸ Repositories) and the document's path inside its docs folder, with the `.md` optional.
The link names the DOCUMENT and the destination is decided per reader: an internal user
lands in the backend, a visitor on the published page of whichever documentation exposes
it to them. A document that moves takes its links with it, and a publication that would
retire a document something still names is refused rather than pushed.

Accepted limitation: it is a path on the site, so it does not resolve on GitLab. Writing
the full domain would — and would nail one Odoo instance into the customer's repository.

## Ordering a nested documentation

A repository can be nested under an article of another one (**Container article**), and
then its root sits beside the roots of the other documentations hanging there. That root
is the one node whose place its own `tree:` cannot state — it is not in that list — so
it is declared at the top of the descriptor:

```yaml
version: 1
weight: 30 # where this documentation sits among its siblings
tree:
  - page: index.md
```

Same number a page writes in its front matter, and it lives here for the same reason the
tree does: order is shape, it is reviewed in a pull request, and a documentation that is
a bare container has no `index.md` to carry it.

**Absent, nothing is written.** Roots ordered by hand before the key existed keep the
order they were given; a default of `0` would collapse a container's children onto one
number on the next import. And a reorder made in Odoo is not run over while it waits to
be published, like any other change of shape.

It travels **both ways**, like the tree itself: reordering the root in Odoo writes the
key back to the descriptor and publishes it. The rest of the tree gets that for free —
here order _is_ list order, so a page moved in Odoo simply comes out elsewhere in
`tree:` — while the root needed a key of its own to have anywhere to say it. A
documentation that is **not** nested never starts declaring one on its own: it has no
siblings to be ordered against, and writing a key nobody asked for is an edit nobody
made. Once the key is in the file, it is kept in step whatever the repository is.

## Restricting who reads a documentation

`access:` sets the **audience** of the documentation's root (module
`dynapps_knowledge_access`): only the internal users who match one of the lists reach
the tree. It needs `version: 2`, so that an older build refuses the file instead of
ignoring the key and importing the tree open to everyone.

```yaml
version: 2
access:
  companies: [Dynapps Spain, base.main_company] # exact name or xmlid
  groups: [hr.group_hr_user] # xmlid
  users: [maria@dynapps.eu] # login of an internal user
tree:
  - page: index.md
```

- **Repository managers always pass**, like the administrator: the manager group implies
  `group_every_space`. They read and write a restricted tree per Knowledge's own rules,
  whatever its audience, so they cannot lock themselves out. The link is written outside
  the groups file's `noupdate` block, so every `-u dynapps_knowledge_git` restores it:
  an administrator cannot remove it for good from the UI (by design), and a deploy runs
  the restart and `-u dynapps_knowledge_git` together.
- **Declared, Git decides.** Every import writes it over what Odoo holds, and in Odoo
  the audience is read-only ("edit it at the source"), for repository managers too.
- **Absent, Odoo decides.** The import never touches the audience. Removing the key
  later leaves the audience as it was and hands it to the Odoo UI: deleting a line never
  opens a restricted tree.
- **Refused, nothing imported:** an `access:` that names nobody (name a group such as
  `base.group_system` for "administrators only"); a value that does not resolve, named
  in the error; a portal login; a documentation nested under a **Container article**,
  because an audience belongs to the root of a Knowledge tree and a shared container
  would mix them.
- The write-back rewrites `tree:` only; `access:` and `version:` stay as written.
- **Who decides who reads the tree is who may push to the repository.** With `access:`
  declared, anyone who can change `knowledge.yml` on the imported branch decides the
  audience, through a merge request that may merge by itself. Protect that branch
  accordingly. Odoo's own write-back cannot change `access:`: it writes inside the docs
  folder only, plus the descriptor's `tree:` (see "Write-back stays inside the docs
  folder" above).
- "Audience from knowledge.yml" (`dynapps_access_declared`, on the repository) is set by
  the import only; a create or write from anywhere else is refused, over RPC too.
- **The import is trusted through `sync_writer()`, never a context key.** The guards on
  managed content (the repository and archive boundaries, the property definition, the
  Git snapshot, the delivered blobs and checksums, the Markdown body) and the "pending"
  marking step aside only inside that block, which server code enters around its own
  write. A `dynapps_syncing` sent with `call_kw` is ignored (it counts only for uid 1:
  crons, data files). Where a document belongs (its space, and the repository, version,
  language and provider stored from it) is written by the sync, `sudo()` or uid 1 only,
  so an editor cannot detach a document from its repository. A space's version and
  repository and a version's repository are re-pointed by the sync only, a declared root
  stays at the top of the tree, and a document created outside the sync sits in its
  parent's space, in a documentation that accepts new documents.
- **A content row stays a file of its own document (#8).** Its document, version,
  language and `base_blob` are written by the sync only, a row created by hand takes its
  document's repository's version, a plain move stops at the NEAREST documentation, and
  duplicating a documentation's document onto a route it has is refused.
- **What the audience lock protects against.** An audience declared in `knowledge.yml`
  is locked against editors and against accidental changes in the UI, on create as on
  write. It is not a boundary against a repository manager: a manager configures the
  repository itself (its clone URL, its root article) and can re-point it, so give that
  group only to people you would trust with the audience.
- A version other than the default one only fills the tree its default version builds:
  when the default version's `access:` is refused, or its descriptor is refused whole
  (on a first import: any failure of the default version), the other versions are not
  imported in that pass either, instead of importing their documents open. The failure
  of the default version is reported; the tree keeps what the last successful import
  gave it.

## Metadata: two layers, and when to use each

Documentation carries facts about itself — who signs it, whether it is doctrine or a
draft, whether an agent may edit it. There are two ways to record that, and they coexist
on purpose.

**The open layer.** Anything written in the front matter is kept whole in
`dynapps_frontmatter`, declared or not. Nothing to configure, nothing to declare: write
a key today and it is stored today. Readable through the ORM and through
`docs_metadata`, which returns it as `front_matter`. What it is not, is filterable: it
is a JSON blob, so "every document with no owner" means reading them all.

**The typed layer.** A field declared in `properties:` becomes a real Odoo property:
typed, painted by the Knowledge panel, filterable with an ordinary domain and groupable
in `read_group`. Declare a field when you want to _ask questions_ about it.

Promoting a field from one layer to the other is four lines in the descriptor — no
migration, no code.

### Declaring the fields

The types live in `knowledge.yml`, which is where the shape of the tree already lives
and where changes are reviewed in a pull request. The front matter of a document carries
**values only**:

```yaml
properties:
  - name: truth_source
    type: selection
    string: Fuente de verdad
    selection: [[decided, Doctrina], [draft, Borrador], [external, Externa]]
  - name: owner
    type: many2one
    comodel: res.users
    string: Dueño
    domain: [["share", "=", false]] # Odoo: what the panel lets you pick
    lookup: [["login", "=", "$value"]] # ours: text -> record
    value_field: login # ours: record -> text (the SAME field)
  - name: ai_writable
    type: boolean
  - name: reviewed_on
    type: date

tree:
  - page: index.md
  - section: como-trabajamos
    index: como-trabajamos/index.md
    properties: # only for the documents of THIS block
      - name: fase
        type: selection
        selection: [[propuesta, Propuesta], [cierre, Cierre]]
    children:
      - page: como-trabajamos/como-presupuestamos.md
```

```yaml
---
title: "Cómo presupuestamos"
truth_source: decided
owner: julio.pons@dynapps.es
ai_writable: false
reviewed_on: 2026-08-14
fase: propuesta
---
```

`lookup` and `value_field` are the two directions of one bridge: a relation is a
**record** in Odoo and a **text** in the file, so something has to translate each way —
`lookup` on the way in, `value_field` on the way back. Both are ours and both are
stripped before the definition reaches Odoo.

Neither can be Odoo's `domain`, which restricts what a human may pick in the widget and
is validated against the model: a template with `$value` in it would fail that
validation and be **deleted silently**. And `value_field` is a field NAME, never an
expression — the descriptor comes from a Git repository, which is untrusted input.

**Write back the same field you search by.** The descriptor refuses a `value_field` that
its `lookup` does not consult, and the refusal is worth the sentence it costs: searching
by `login` while writing back `email` breaks nothing on import. It breaks one
synchronization after somebody changes an owner — the import looks for an email among
logins, finds nothing, and **empties a field nobody touched**. A relation also needs
both keys: without `lookup` the text cannot become a record, without `value_field` an
edit made in Odoo can never go home.

A lookup that matches nothing — or more than one thing — leaves the property **empty**
and says so in the synchronization log. An owner filed under the wrong colleague is
worse than an owner nobody set.

### Where each definition lands

Odoo reads a property definition from the **immediate parent** and inherits nothing, so
the import writes it on every node that groups others:

| Node                | Receives                     | Seen by                                |
| ------------------- | ---------------------------- | -------------------------------------- |
| Root of the space   | the documentation-wide block | its first-level children               |
| `como-trabajamos`   | that block **plus `fase`**   | the documents inside `como-trabajamos` |
| any other container | the documentation-wide block | its own children                       |

**A property is a property of the DOCUMENT, not of a translation.** Values come from the
base-language file; an `owner` in the `.en.md` is ignored, because a document is one
article and its languages are rows of it.

### Which half Odoo may change

|                                     | Source of truth     | From the Odoo UI                      |
| ----------------------------------- | ------------------- | ------------------------------------- |
| The definition (which fields exist) | the `knowledge.yml` | **refused** on a managed tree         |
| The value (who signs this document) | the document        | **editable**, and it goes back to Git |

The split is not a preference, it is what makes the panel usable. Odoo's own mechanism
does not split it: editing a property from the panel rewrites the definition on the
**parent** (`fields_properties.py:322-327`), so one person adding a field to their
document would silently restructure every sibling — and the next sync would undo it
anyway. So the definition is refused here, with a message naming the file and the
repository to open instead of a bare "not allowed".

### The way back

Changing a value in Odoo writes it into the front matter of the base row and flags that
row, which is what the write-back wakes on. The result is an ordinary commit with the
right author: changing who signs a document travels exactly like changing a paragraph.

When Git and Odoo disagree about a value, the precedence is the one the text already
uses, read off the same flag so the two cannot drift apart:

1. the values match → nothing moves (this is how a published edit coming home closes its
   own cycle);
2. the base row has an **unpublished edit** → the import does not touch its properties;
3. otherwise → Git updates the value.

### Asking

```python
# every document nobody signs
[("article_properties.owner", "=", False)]
# doctrine not re-signed in a year
[("article_properties.truth_source", "=", "decided"),
 ("article_properties.reviewed_on", "<", "2025-08-14")]
```

From the MCP, `docs_search` and `docs_list` take a `where` mapping
(`{"doc_type": "gotcha"}`). It is a map and never a raw domain — an assistant handed a
domain can reach fields it was never meant to read — and an undeclared field is refused
with the list of the ones that exist.

## The emoji of a document

Knowledge shows an emoji beside every document. It is declared in the front matter,
beside the title, and it travels in both directions like any metadata value:

```yaml
---
title: "40 · Desarrollo"
icon: "🛠️"
---
```

**Do not put the emoji in the `title`.** That is the whole reason this field is
supported: the title reaches the embedding (`_dynapps_index_context`), so an emoji
written into it becomes noise in the vector of _every_ chunk of the document — a heading
like `📂 CONSULTORIA 🚀🚀` ships two of them with each fragment. `icon` never gets
there, and the tree still shows it.

Three rules worth knowing:

- **It belongs to the document, not to the translation.** `icon` is a column of the
  article, and the article is one record for every language: the base language decides,
  and an `icon:` in a `.en.md` is ignored rather than fought over.
- **A document with no `icon:` leaves the field alone.** Absence is silence, not an
  instruction — otherwise every emoji set by hand would be wiped the first time this
  ran. Removing one is writing `icon: ""`.
- **Changing it in Odoo travels back to the file**, with the same precedence as any
  other value: an unpublished change is not reverted by an import, and a published one
  coming home through the webhook moves nothing.

## When a publication gets stuck

`dynapps_git` warns the repository's maintainers whenever the circuit fails — see its
own README for who is warned, and why exactly once. What this module adds is the part
only it knows: **which documents are waiting inside the stuck proposal**. The activity
lists them by title (up to ten, then a count), because "a merge request has a conflict"
is a task the reader must first investigate and "the two pages you rewrote on Tuesday
are stuck" is one they can start.

The maintainers, the chatter and the "since when" fields are on the repository's
_Synchronization_ tab. The list degrades rather than fails: if the documents cannot be
resolved the warning still goes out without them.

## Pushes on a repository that imports by hand

A documentation repository is `managed` — the consultants write in Odoo and Git keeps
the copy — so it defaults to `sync_mode = Manual only` (see `dynapps_git`'s README). A
validated push then **queues nothing**: the event is still remembered, including the
file list the platform computed for free, so the synchronization somebody eventually
asks for is still cheap. The decision is asked of the model, never of the webhook
controller, which is public and has no business knowing it.

What this does **not** change is publishing: a write-back still reads the repository
first, in every mode, because publishing onto a base Odoo has not read replays Odoo's
bytes over whatever reached Git and erases it in silence. If you ever need to see that
promise asserted on real content, it is `tests/test_sync_mode.py`.

## When the documentation is published, and when it merges

This module ships its **own** write-back cron, because `dynapps.knowledge.repo` inherits
`dynapps.git.repo` by prototype and therefore has its own table: the generic job runs on
a table that holds no documentation repository. It is daily with its `nextcall` at
**02:00 UTC** — nights are when nobody is editing, so the tree it reads holds still
while it reads it — and, like every job here except the AI descriptions, it is installed
**disabled**.

The merge request poll is not duplicated: proposals are one plain model with one table,
so `dynapps_git`'s job already follows the ones this one opens. It runs every ten
minutes, which is why the two no longer need to be spaced hours apart: whatever the
02:00 pass opens is picked up minutes later, merged if it is small and clean, and left
open with an activity if it is large. What "large" means, and what happens to a proposal
that deletes documents, is in [`dynapps_git`'s README](../dynapps_git/README.md).

## Further reading

- [`docs/mkdocs-import.md`](docs/mkdocs-import.md) — structure, i18n, versions.
- [`docs/llms-txt.md`](docs/llms-txt.md) — the `/llms.txt` endpoints.

## Technologies referenced

mkdocs · [llms.txt](https://llmstxt.org/) · PyYAML · Python-Markdown · Fernet
(cryptography) · Git (GitHub / GitLab webhooks) · OpenAI · Anthropic · Cohere · Voyage ·
[ZeroEntropy](https://www.zeroentropy.dev/) · retrieval design adapted from
[gbrain](https://github.com/garrytan/gbrain) (MIT, © Garry Tan).
