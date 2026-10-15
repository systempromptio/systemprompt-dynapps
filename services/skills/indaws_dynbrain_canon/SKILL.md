---
name: indaws-dynbrain-canon
version: 1.1.3
effort: medium
model: opus
license: Interno Dynapps
description: >-
  Canonicity for Dyn Brain: which documents are verified, who owns them, when they must be
  re-signed, and which ones leave the RAG index. Owns the Dyn Brain article tree, the property
  schema behind the canonical state, the seeding of an existing subtree, and the daily deadline
  and escalation pass. Use it when the user wants to set up or inspect Dyn Brain's canonical
  layer, mark documents as verified, check what is expiring, seed a subtree into a territory, or
  understand why a document did or did not leave the index. NOT the weekly sweep that propagates
  a novelty — that is dynbrain-weekly-queue. NOT retrieval or authoring — that is dynbrain.
---

# Dyn Brain — canonicity

A document that nobody signed is not documentation, it is a rumour with formatting. This skill
owns the layer that answers three questions about every managed article: **is it verified, who
signs it, and when does that expire.** Everything else in Dyn Brain reads those answers.

Five jobs, and nothing else: the **tree** (`references/tree.md`), the **canonical model**
(`references/canon-model.md`), the **daily pass** over deadlines (`references/deadlines.md`),
**seeding** an existing subtree (`references/seeding.md`), and what may be **indexed**
(`references/rag-eligibility.md`).

**Writing a curated article is `references/authoring.md`** — the rich Markdown syntax, the
state as a review gate, the three silent property traps, and the shape that held up. Read it
before authoring anything: every item in it cost something to learn on the first article.

## Language

Write everything **in English** inside this skill and its references, example sentences
included — same rule as `dynbrain`, and the convention of this group repository. When you
**talk to the user, reply in the language they wrote in**.

## Which MCP am I using? (never hardcode a name)

Two different servers, both needed, and they are not interchangeable:

- The **generic ORM proxy** — `orm_search_read`, `orm_write`, `orm_create`, `orm_aggregate`,
  `orm_post_message`. Everything in this skill goes through it: `article_properties`,
  `mail.activity`, `dynapps.knowledge.rag.rule`. Which server answers, and the fallback when
  none does, is the dual contract in `references/shared/odoo-connection.md` — this skill
  does not open its own connection.
- The **documentation MCP**, the one exposing `docs_*`. Used for reading and authoring
  documents, per `dynbrain`.

Identify each by its tool family, never by a name you wrote down. Either can be **entirely
absent** from a session, particularly a scheduled one. If the one you need is missing, say so
and stop — never guess at a result.

## Gotchas — the three hard facts about properties

Verified against production. All three fail **silently**: the write returns success and stores
nothing.

1. An article carries `article_properties` only if its **direct parent** declares
   `article_properties_definition`. It does **not** cascade to grandchildren.
2. A **root** (`parent_id = False`) can never carry them, whatever it declares.
3. Writing `article_properties_definition` **replaces the whole list**.

So: provision the definition on **every** node of the tree, not only the ones that have
children today — `nodes_to_provision()`, not `parents_needing_definition()`. Merge with
`merge_definition()`. And **always re-read** with `assert_written()`. An `ok: true` from the ORM
means nothing here.

## The one guard that matters

The RAG exclusion rule matches `canon_managed = True AND canon_state NOT IN (verified,
expiring)`. So **marking an article managed without verifying it in the same write retires it
from the index.** Activating canonicity on 500 articles to start curating them would retire 500.

Always build the write with `activate_canonicity()` from `scripts/canon_state.py`. Never assemble
the dict by hand — by hand is how the guard gets skipped, and nothing downstream will notice.

`supersede()` is the deliberate exception: retiring a source that a curated article replaced is
exactly a managed-and-not-eligible write, and that is the point.

## Never

- **Never write to an article body.** This skill only ever writes its own `canon_*` properties.
  Propose; a person applies.
- **Never archive or delete.** Nothing here is irreversible; every state returns to `verified`
  with one action.
- **Never touch `is_published` / `website_published`.** Verified is not published, and publishing
  is not this skill's decision.
- **Never use `write_date` as a freshness signal.** A bulk reprocess left 65 distinct timestamps
  across 2.880 articles. Use `canon_verified_on`.
- **Never use `stage_id`** for canonical state: stages are scoped to the containing article.
- **Never put prescriptive content in a root.** A root's body is never swept, so doctrine there
  is invisible to propagation.

## Typical workflows

- **"Write a curated article"** → `references/authoring.md`. Create it managed and `draft` so
  the exclusion rule keeps it out of the index until its owner signs it.
- **"Set up the tree"** → `references/tree.md` → create with `docs_create_tree` → provision the
  definition on every node → activate canonicity on the 22 non-root nodes → re-read to confirm.
- **"Seed this subtree into a territory"** → `references/seeding.md`. **Dry run is the default**:
  report total, native and synced, write nothing. Real execution needs explicit human
  confirmation, and never changes a `parent_id`.
- **"What is expiring?"** → `references/deadlines.md` → `deadline_pass.py` returns the transitions
  that fall due today, with their reason. It does not apply them.
- **"Why did this leave the index?"** → `references/rag-eligibility.md`. Read its `canon_state`
  first: managed and not in {verified, expiring} means excluded by rule `sequence: 5`. Then check
  `dynapps_rag_mode` — a per-article `always`/`never` **beats every rule**.

## Scripts

| Script | What it owns |
| --- | --- |
| `canon_state.py` | States, transitions, deadlines, the annual floor, activation and supersession |
| `provision_definition.py` | The property definition, merging, the re-read guard |
| `seed_report.py` | The dry-run report and the writes a seeding would make |
| `deadline_pass.py` | Which transitions fall due today, and why |

All pure functions: they decide what to send, the caller sends it. That is what makes them
testable, and the tests are in `tests/test_*.py` at the repository root.

## Delegates to

`dynbrain` — reading and authoring documentation, and the `docs_*` access contract.
`dynbrain-weekly-queue` — the weekly sweep that turns a published novelty into reviews.

## Consumed by

`dynbrain` — before writing or indexing anything, to know whether a document is signed.
`dynbrain-weekly-queue` — to decide who owns a review and when it falls due.
`dynbrain-decision-harvest` — to publish a novelty in a state the exclusion rule accepts.

Nobody outside the Dyn Brain group: the canonical state is this group's private vocabulary.
