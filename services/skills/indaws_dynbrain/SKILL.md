---
name: indaws-dynbrain
version: 2.3.0
effort: medium
model: opus
description: >-
  Dynapps documentation brain: how to READ the house doctrine out of Odoo Knowledge (where a document
  goes, how it is written, what its metadata means) and how to DRIVE a documentation MCP — search,
  retrieve, navigate, author in both languages, and say truthfully what happens to what you wrote.
  Use it when the user wants to write, structure, organize,
  search, retrieve or answer questions from Dynapps/Odoo Knowledge documentation, or works with
  docs_search / docs_answer / docs_get / docs_create / docs_outline tools — even if they never say
  "dynbrain". The server's name is the user's to choose: identify it by its docs_* tools, never assume one.
---

# Dynbrain — Dynapps documentation brain

Two jobs: **authoring** (drafts follow the house layout, which is **not written here** — it is
documentation and it lives in Knowledge; the anchors and the order of calls are
`references/knowledge-as-source.md`) and **the documentation MCP** (how to drive it well —
per-tool examples, conventions and chained recipes under `references/mcp/`, indexed by its
`README.md`). Traps live in
`references/gotchas.md`: **read that one before reporting any result back.** This page is
the orientation and the rules that always apply.

## On activation, before anything else

Two checks, mandatory, every activation — both are cheap, and neither reads a document body:

1. **Identify the connected documentation MCP.** Never assume a name. Follow "Which MCP am I
   using?" below to find the server exposing the `docs_*` family; more than one candidate, none,
   or any ambiguity → **ask the user** before calling anything else.
2. **Locate the governance doctrine — do not read it yet.** One typed call finds where it lives
   and roughly how it is shaped, so you know where to go back to later:

   ```jsonc
   docs_search { "query": "how this documentation is organised, where a document goes",
                 "where": { "doc_type": "governance" } }
   ```

   Keep only what the hits themselves carry — their `id`s, how many came back, how they are
   titled or sectioned. **Do not call `docs_get` on any of them here**: this is a locate-only
   call, not the "read the doctrine" step — that one stays gated by
   `references/knowledge-as-source.md` → "When to go and get it", and applies only once you are
   actually about to author. An empty or refused result is itself the finding for this
   installation: carry it forward, don't retry with a bare search.

## Language

Write everything **in English** inside this skill and its references, **example sentences included**
— that keeps it portable, and an example teaches the *shape* of a sentence (what it names, what it
distinguishes, what it asks), which survives translation. When you **talk to the user, reply in the
language they wrote in**, every turn, so you will be saying those examples in their language.
Documentation goes in the language of its target Knowledge space and audience; if unclear, ask.

## Which MCP am I using? (never hardcode a name)

The tools come from **an MCP server the user configured, whose name is theirs to choose**,
so they appear as `mcp__<their-server-name>__docs_search`. Identify the server by its **tool
family** — the one exposing `docs_*` (`docs_search`, `docs_get`, `docs_create`, …; the full
list, read and write, indexes `references/mcp/README.md`) — never by a name you wrote down.

Exactly one such server → use it; the user names one → keep it for the session; more than
one, none loaded, or any ambiguity → **ask before calling anything**, because the wrong
server searches the wrong corpus. The access contract (preflight, and what to do when the MCP
is absent) is `references/shared/docs-access.md`. Every tool runs **as the calling
user**, so empty results can mean "no access", not "nothing there".

**Your tool list is the contract, not this page.** A server exposes the families it was
granted, so two servers of the same product do not offer the same tools. Read what you
actually have before promising a step: `references/mcp/README.md` → "Families, and why a
tool you read about may not be there".

## Where does this document come from? (check before writing; it decides what comes after)

Not every document is on its way to Git. **Three** origins exist, and they are what makes
creating, editing or retiring a document a finished job or a half-done one.
`docs_metadata {article_id}` answers it in one call — read `repo`, `is_synced`, `can_write`:

| What you see | Origin | What it means after a write |
| --- | --- | --- |
| **no `repo`** (empty) | **native** — lives only in Knowledge | **Nothing to publish.** Saving *is* the change. No commit, no proposal, no URL. |
| `repo` + **`can_write: false`** | **mirror** — imported, read-only | You cannot edit it; the tools refuse. It is edited at its Git source. |
| `repo` + **`can_write: true`** | **managed** — imported and writable | The change is live in Odoo now and **held pending publication**: it reaches the repository with the documentation's next publication, which is scheduled and which whoever administers the repository can also launch. |

When you create a page or attach a file, the answer says it too (`in_repository`,
`pending_publication`, and a `next_step` written for the server you are on). Both mistakes
cost: announce a publication step on a **native** document and the user waits for a URL that
never exists; call a **managed** change "in the repository" while it is still pending and they
believe the client has the new text — the repository does not, and nobody finds out until the
next reader looks. So check the origin **before** you write: it governs creating, editing,
retiring and attaching a file alike.

## Retrieval: ask well, scope on purpose

Retrieval is hybrid (embeddings + keyword), reranked and **cross-lingual**, so lean on
meaning: ask a full natural-language question, not a keyword bag ("How often are backups taken
and how long are they kept?" beats "backup frequency retention"). Pick the tool for the
need — `docs_search` to find, `docs_answer` for a cited answer, `docs_get` to read one
document, `docs_outline` to navigate. Scope with a `space` from `docs_list_spaces`, never an
invented id, and leave `lang` off: a Spanish question finding English documents is the point.
One reformulation usually fixes a bad result. Examples: `references/mcp/read-tools.md`.

## Authoring

**The house layout is not in this skill: it is documentation, and it lives in Knowledge.**
It is not addressed by name, path or id either — those are per-installation — but **found by
what it is**, in one call:

```jsonc
docs_search { "query": "how this documentation is organised, where a document goes",
              "where": { "doc_type": "governance" } }
```

`where` is a typed filter, so nothing outside that class can come back and relevance only
orders the results. Read the hits by the `id` the search returned. **If two of those documents
disagree, that is a finding, not a fork**: say so, name both, follow the stricter one meanwhile,
and let a person settle it — choosing in silence costs everything written under the choice. The full flow, the
narrower filters and what to do when it comes back empty — which is the normal case on a
client's Odoo — are in `references/knowledge-as-source.md`. **When nothing is published, say
so and agree the shape with the user; never fall back to a bare search and treat what it
returns as the house standard.**

Two calls decide the write and neither can be guessed:

- `docs_metadata {article_id: <the parent>}` — native, mirror or managed, and which block you
  are in.
- `docs_metadata_schema {article_id: <the parent>}` — the fields **that apply there**, and an
  `expects` per field saying exactly what to write into it. Anchored on the PARENT, never on
  the repository: fields are declared per section, so a repository-level answer loses whatever
  that section adds and a valid value gets dropped. **Call it before you write metadata**, not
  after a warning.

Then author with `docs_create` / `docs_create_tree`, in **real, heading-structured Markdown** —
what lets a RAG-configured system chunk and index it (indexing is asynchronous and the system's
job: verify with `docs_metadata`, never trigger it). Under a Git documentation the page is
created **inside the repository** — report the `path` that comes back — and held pending.

**The Markdown itself is a closed grammar**, and it is not the one you know from GitHub or
mkdocs: an attribute list, a footnote or a task list degrades, and a bare `<tag>` in prose
makes the save fail outright. What survives, what does not and what gets rewritten without
asking — all of it measured — is `references/markdown.md`. Read it before writing a page
that uses anything beyond headings, lists, links and emphasis.

**Three arguments, three layers, and you do not hand-write YAML to reach any of them**
(`references/mcp/write-tools.md`):

| What | Argument | On `docs_create`, `docs_create_tree` (per node) and `docs_update` |
| --- | --- | --- |
| The title | `title` | With `lang` it names that translation; without it, the document |
| The documentation's typed fields | `metadata` | Whatever `docs_metadata_schema` lists — **without `lang`** |
| The other header keys | `front_matter` | `description`, `icon` — and only what the documentation manages |

A pasted `.md` with its header still works and lands in the same places; an explicit argument
wins over the same key inside a pasted header.

**Two refusals that fail the WHOLE call rather than warning you** — on a document that belongs
to a repository, and this is the change most worth carrying:

- A **`metadata` key the documentation does not declare**. Nothing is created, nothing is
  written. No tool can declare a field: it is added to the repository's `knowledge.yml`, in
  Git, and the error says so.
- A **header key outside what the documentation manages**. What Odoo may write into a
  Git-backed header is `title`, `icon`, `description` **plus the declared typed fields** —
  nothing else. A `weight:` or an `audience:` the descriptor never declared is refused, because
  accepting it would store a key in Odoo that the published file will never carry.

On a **native** document neither applies: there is no descriptor to answer to, so an unknown
key is a warning and the header stays permissive.

Afterwards read `front_matter_warnings` and `metadata_warnings`: **those** are the keys that
were refused while the answer still said `updated: true`. And know what is **not** checked —
a relation, a selection, a tag and a number are resolved and reported, but **a date and a
free-text field are stored exactly as sent and committed to the customer's file that way**.
So write `2026-08-28`, never "yesterday": nothing will stop you, and nothing will show it in
Odoo's own form either.

### Every document exists in Spanish and in English. Always, unasked.

Spanish is the default and the one that governs; English is not optional and not "for later".
**Nobody has to ask for it** — a document with one language is unfinished, and saying so is
part of the job, not a favour.

```jsonc
docs_create { "parent_id": <parent>, "title": "…", "markdown": "<the Spanish body>",
              "front_matter": { "description": "…" } }
docs_update { "article_id": <the new id>, "lang": "en", "markdown": "<the English body>" }
docs_update { "article_id": <the new id>, "lang": "en", "front_matter": { "description": "…" } }
docs_metadata { "article_id": <the new id>, "lang": "en" }   // check the variant, not the echo
```

Three things make that sequence what it is:

- **The second language is a VARIANT**, `docs_update {article_id, lang}` on the same document.
  Two `docs_create` give two unrelated documents, both calls succeed, and nothing warns you.
  A `docs_create {lang}` inside a repository does **not** create a translation either: it
  creates the document, in the repository's own base row, and says so.
- **The title and the description belong to the FILE; the typed fields belong to the DOCUMENT.**
  Two files, two headers — so a translation carries its own — but one article, so `doc_type`,
  `territory` or `owner` are written **without `lang`**, and sending one inside a translation is
  refused and reported in `metadata_warnings`.
- **A translation with no `description` is not broken.** There is no inheritance on purpose: the
  description is what the index reads, and lending it the original's would make the page look
  described while being unfindable in its own language. Write it, do not report it as a fault.

The English has to be actual English, not the Spanish text under another label: it feeds the
English index, and a mirror degrades retrieval in both languages. If a documentation has no
second language configured the MCP refuses the variant — report that, do not leave it half done.

**Where this skill and a tool description disagree, the tool wins.** It ships with the server;
this page does not.

## You write. Something else publishes.

A change to a `managed` document is **saved in Odoo the moment the tool returns** and **held
pending** until the documentation's next publication carries it to Git. That publication is
scheduled, and whoever administers the repository can also launch it. **It is not your step**,
and on most servers — including the one this skill was written against — you do not even have
the tools for it: `docs_propose_changes` and its three companions are a separate family that a
server exposes or does not.

Four rules, and they hold whether or not you have those tools:

- **Never offer to publish, merge or synchronise.** Do those only when the user names them,
  and only with tools you can actually see in your own list. An offer reads as a
  recommendation, and all three write to a system that is not yours.
- **"Saved" and "in the repository" are different sentences, and both are true at different
  moments.** Say: *"It is saved in Odoo. The repository gets its copy with the next scheduled
  publication — nothing for you to do."* Never tell somebody their work "does not count yet",
  "is not saved" or "could be lost" because it is not in Git. It is false, and it makes people
  re-type what they already have.
- **Read the answer before you narrate it.** `pending_publication`, `publish_required` and the
  `next_step` sentence are written for the server you are on; they, not your memory, say what
  is left. `trashed: true` on a Git documentation is a page removed in Odoo whose file is
  still in the repository.
- **Anything an assistant wrote needs a person before it merges.** A change proposal carrying
  work produced through this MCP is deliberately held (`waiting_because: "awaiting_review"`)
  and no amount of retrying moves it: a human approves it on the platform. Say that instead of
  "it will go out tonight".

What the mechanism actually does, what a repository format changes about it, and the four
tools for the servers that do have them: `references/mcp/pending-work.md`.

## Synchronising is the other direction, and it is nobody's here

`docs_sync_status` answers "is Odoo still in step with Git, and what would arrive". Reading a
repository INTO Odoo is a button on the repository's form, for whoever administers
repositories (`can_synchronize` says whether your user is one). **No tool of this MCP does
it**, and on a documentation Odoo owns an import **replaces what Odoo holds**. So never
synchronise on your own initiative, never without first saying what would change
(`incoming` names the files), and when the user is not allowed to, say who is and stop there.
Hunting for a way around it is the wrong instinct.

## Typical flows

Sequences live in `references/mcp/workflows/` (its `README.md` says in one line what each recipe is for). Six worth carrying in your head:

- **"Find/answer something"** → (`docs_list_spaces` to scope) → `docs_answer` for a cited
  answer, or `docs_search` + `docs_get` to read the source.
- **"Fix / update this imported page"** → `docs_metadata` (**origin**) → `docs_update` →
  read `pending_publication` → say it is saved and pending. That is the whole task.
- **"Reorganize this documentation"** → `docs_move` (**it works on Git documentations too**;
  the file is not renamed) → read `publish_required`: on a `knowledge` documentation the move
  is pending, on a `mkdocs`/`rst` one **nothing will ever carry it to Git** and the answer
  says so.
- **"Retire this page"** → `docs_trash` → **native: done, say so.** Managed: the answer
  carries `publish_required` and a `next_step`; read them out rather than assuming.
- **"Add this diagram / PDF to the page"** → `docs_asset_upload` twice, with the bytes
  POSTed to the URL in between (no tool of yours carries them) → `docs_update` to link it by
  its **relative name**. A diagram is a **`.svg` FILE**, never an `<svg>` pasted
  into the Markdown — that one empties out the next time the page is edited in Odoo.
  Files: `references/mcp/workflows/files.md`.
- **"Link this page to that one"** → a **relative path to the other page's `.md`**, never an
  Odoo URL. A move rewrites the links pointing at the moved page for you; a link the change
  would break **stops the publication** and nothing is committed.
  Links: `references/mcp/workflows/authoring.md` §7.

## Typed metadata: asking questions of the corpus, not of the prose

A documentation declares **typed fields** in its `knowledge.yml` — a status, an owner, a
review date — and each document carries the values in its header. That is what lets you ask
things retrieval cannot answer: *which documents nobody owns*, *what is still a draft*,
*where an agent may write*. Three tools in this order: **`docs_metadata_schema`** (what
exists and what to write into it, anchored on the parent), **`docs_update {metadata}`** (set
it), **`where`** on `docs_search` / `docs_list` (filter by it — `docs_search` answers "what is
relevant", `docs_list` answers "what is left to sign").

One trap worth carrying even before you open anything: **you write a relation as text and
filter it by id.** Both ends are correct and they are not interchangeable. The rest —
`expects` per type, what is checked and what is not, and why an undeclared name fails the
whole call — is `references/mcp/read-tools.md` → `docs_metadata_schema` and
`references/mcp/write-tools.md`; the traps are in `references/gotchas.md`.

These tools say what a field **is**. What each one **means**, and which value to choose, is
doctrine and lives in Knowledge — `references/knowledge-as-source.md` says where.

## Canonicity: is this document signed?

Retrieval says what we have written. It does not say what anybody has **verified** — that is a
separate layer, `dynbrain-canon`, and it lives in that same typed metadata under the
`canon_` prefix — so the tools above are how you read and set it.

Two facts worth carrying while you read or author:

- **The absence of a canonical state means nobody has signed the document.** Most of the corpus is
  in that position, so do not present a retrieved passage as doctrine. "This is what our
  documentation says" is honest; "this is our policy" is not, unless `canon_state` says `verified`.
- **Where you put a document is an instruction to the weekly sweep.** Each block of a territory
  carries a different regime — one is signed doctrine, another is maintained by its source, another
  is a record of the past that never contradicts anything — so filing a decision in the wrong one
  means propagation never sees it. Which block is which is doctrine, and it is read from the corpus
  (`references/knowledge-as-source.md`), never remembered from here.

The model, the six states and the deadlines are one page: `dynbrain-canon/README.md`.

## Before writing doctrine, check the running system

Documents, skills and memory tell you what we **believe**. Only Odoo tells you what is
**happening**. Curate from all four, and treat the system as the tie-breaker — it is the
one source nobody edits to make a point. What that caught on the first real article — a
template that did not exist, a fourth one nobody remembered, a "retired" one still in use,
and a price config contradicting itself — is in `references/knowledge-as-source.md`.

Three cheap checks before signing anything:

| Claim in the draft | Check |
| --- | --- |
| Names a record — template, product, stage, activity type | `orm_search_read` it. If the name does not resolve, the doctrine is unusable |
| Says something is retired, standard or "what we use" | `orm_aggregate` by month. Adoption is a number, not an opinion |
| Quotes a price, rate or factor | Read it from the skill's own config, then look for a second value of it |

**Report the gap, do not paper over it.** "This is our policy and 25% of quotes still do the
old thing" is useful; "this is our policy" alone is how a document stops matching reality
without anybody noticing.

And prefer a **table of values over a screenshot**. A capture of a spreadsheet cannot be
read by retrieval and cannot be diffed, so when a price changes the image lies and nothing
detects it. Screenshot the *layout* if it helps somebody find a cell; never the numbers.

## Gotchas

Full list in `references/gotchas.md`; consult it **before** reporting a result. The costliest:
a second language is `docs_update {article_id, lang}` on the same document, never a second
`docs_create`; a `metadata` field the descriptor does not declare **fails the whole call**, and
so does a header key outside `title`/`icon`/`description`-plus-declared-fields; a date and a
free-text value are **never checked** and travel to the customer's file as typed;
`trashed: true` leaves the file in the repository, and on a `mkdocs`/`rst` documentation it
leaves it there **forever**; "nothing pending" also covers work already handed to a proposal
and not yet merged; an uploaded file nobody links is invisible; `metadata_warnings` is a field
that did **not** get written, however green the answer looks; and **a translation owns its
title and description but not the typed fields** — those go without `lang`, and a translation
with no description of its own is the gap to fill, never a defect to report.

## Delegates to

`dynbrain-canon` — the canonical state: which documents are verified, who signs them, when that
expires, and what may be indexed.
`dynbrain-weekly-queue` — the Monday sweep that turns a published novelty into reviews for the
owners of the affected documents.
`developer-skills:git-manager` — only on a server that exposes the publish family: the commit
convention in `developer:skills/git-manager/references/commit-standards.md`.

Nothing else: this skill owns its own retrieval rules.

## Consumed by

The **devops suite**, through one contract: its `references/shared/docs-access.md` makes this
skill the source of truth for reading documentation, and its skills inherit that file rather
than depending on this one each. Not a roster on purpose (a copy here rots, they live in
another repository): `grep -rl dynbrain ../ai-claude-devops/skills/*/`.
