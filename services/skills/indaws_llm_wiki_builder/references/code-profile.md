# The `code` profile — a wiki over a source tree

For building a second brain over a codebase (Odoo CE/EE/OCA, or a client's custom modules) that
gets **structure right and never guesses**. The unit of value is: you ask "who extends
`sale.order` and what does `sale_stock` add", and the answer is exact, cited, and checkable in one
jump to `file:line @sha`.

- [Why this profile is different](#why-this-profile-is-different)
- [Two layers, one hard rule](#two-layers-one-hard-rule)
- [Layout](#layout)
- [Workflow](#workflow)
- [Citations](#citations)
- [What static extraction cannot know](#what-static-extraction-cannot-know)
- [Base version and deltas](#base-version-and-deltas)
- [Page taxonomy](#page-taxonomy)
- [Drift](#drift)
- [Anti-patterns](#anti-patterns)
- [Piloting it before trusting it](#piloting-it-before-trusting-it)

## Why this profile is different

Every other profile ingests documents: text in, prose synthesis out. Here the source is code, and
that breaks three assumptions:

1. **The source cannot be copied into `raw/`.** Odoo 19 CE is 1.3 GB and already a git repo. So
   `raw/` stays empty for code and sources are referenced by **repo + commit SHA + path**.
2. **"One page per source" has no meaning.** 8.500 files is not 8.500 sources. The unit is a
   **module** for inventory and a **subsystem or mechanism** for synthesis.
3. **"Without mistakes" is not achievable by writing.** An LLM writing `fields.Many2one`
   signatures or `_inherit` chains from memory produces plausible, subtly wrong prose — and wrong
   with confidence is worse than absent. Structure has to be *extracted*, not authored.

## Two layers, one hard rule

**Layer A — inventory (generated, never authored).** `scripts/code_extract.py` parses the tree
with `ast` (Python), ElementTree (XML) and regex (JS registrations) and writes: models with
`_name`/`_inherit`/`_inherits`, every field with its type, comodel, `compute`/`related`/`store`,
every method with its real signature, decorators and a normalised `body_sha1`, `@http.route`
routes, XML ids, `registry.add` keys, `patch()` targets, and each module's manifest — all pinned to
the repo SHA **and stamped with its Odoo major**, into a queryable SQLite plus JSONL. Nothing is
imported or executed, so there is no runtime, no database, no risk. Query it with
`scripts/code_query.py`; a structural question is a query, not a 1.3 GB grep.

One inventory holds **several majors side by side**, which is what makes the version difference a
query instead of a hand-kept cheat sheet that silently expires (see
[Base version and deltas](#base-version-and-deltas)).

**Layer B — the wiki (this is where you write).** The *why*: mechanisms, cross-module flows, "how
to do X correctly here", traps, design intent, decisions. Prose, cited into layer A.

> **The rule that makes this work: you never retype structure.** A signature, a field list, an
> inheritance chain in a layer-B page must come from a layer-A query and carry its citation. If
> you catch yourself writing a signature from memory, stop and run `code_query.py`. This single
> rule is what separates a second brain from a confident liar.

## Layout

```
<wiki-root>/
├── CLAUDE.md
├── index.md · log.md · open-questions.md
├── inventory/                # layer A — GENERATED, never hand-edited
│   ├── inventory.sqlite      # every extracted major, query with code_query.py
│   ├── jsonl/<version>/*.jsonl   # same facts, greppable and diffable
│   ├── meta.json             # base_version, versions, refs (per version+layer+repo)
│   └── modules/<module>.md   # generated page per module (only for what you document)
└── wiki/                     # layer B — written by you
    ├── overview.md           # the map: how the subsystems relate
    ├── subsystems/           # sale-to-invoice, stock valuation, mail threads…
    ├── models/               # one page per model that carries real weight
    ├── mechanisms/           # compute/depends, ORM cache, cron, MRO of _inherit…
    ├── patterns/             # "how to extend X well" + the trap next to it
    ├── deltas/               # one page per CHANGE between majors that has a story
    ├── migrations/           # per version jump: an aggregation over deltas
    └── syntheses/            # filed-back answers, comparisons
```

Everything outside `deltas/` and `migrations/` describes the **base version** and does not repeat it
in every sentence.

`raw/` stays empty (or holds pasted docs, release notes, tracebacks). The code is not copied; it
is cited.

## Workflow

**1. Build the inventory** (minutes, zero tokens). `$CE19`/`$EE19` son las rutas de *tu* fuente:
resuélvelas con `references/shared/odoo-source.md` — **no asumas `/opt/odoo`**, que es la
convención de un entorno concreto y no la de todos.

```bash
# the base version — the one the prose will describe
python3 scripts/code_extract.py --odoo-version 19.0 --base \
  --addons-path $CE19/addons \
  --addons-path $CE19/odoo/addons \
  --addons-path $EE19=ee \
  --out <wiki-root>/inventory

# each older major into the SAME inventory, so deltas become queryable
python3 scripts/code_extract.py --odoo-version 17.0 \
  --addons-path $CE17/addons \
  --addons-path $CE17/odoo/addons \
  --addons-path $EE17=ee \
  --out <wiki-root>/inventory
```

Measured on this machine (Odoo 19 CE+EE, extractor 1.1.0): 1.428 modules · 15.016 py + 10.882 xml +
8.671 js files → **36 s**, 36 MB SQLite + 52 MB JSONL **per major**. 0 Python `SyntaxError`; the 20
recorded notes are 9 XML files that are not standalone documents and 11 deliberate refusals
(`_name = f'export.{name}'` in test modules — the extractor never guesses a model name). Add
`--addons-path $OCA19=oca` and your custom repo with `=custom`.

Re-running a version **replaces exactly that version's rows** (delete-then-insert). Do the extra
majors early even if you will not write deltas for months: while you write the base pages,
`lifespan` is a guardrail that tells you whether "in 19 it works like this" is a novelty or has been
true since 14 — and that changes the sentence.

The paths above are **examples from one machine**. Resolve the real ones per project (in this
ecosystem: `references/shared/odoo-source.md`) instead of assuming `/opt/odoo` — a wrong
tree silently produces an inventory for the wrong version, which is the worst possible failure.

**1b. The framework itself needs `--core-path`.** `odoo/` is not an addons tree: it has no manifests,
so `discover()` finds nothing, and `BaseModel`/`Field`/`Environment` don't subclass `Model`, so the
addons parser would skip them even if it looked. `--core-path $CE19/odoo` records **every**
class and function as `kind='pyclass'` with a dotted path in `model`
(`odoo.orm.models.BaseModel`), one pseudo-module per subpackage (`odoo.orm`, `odoo.tools`), skipping
`addons/`. Every existing query works on it unchanged — Odoo model names never collide with dotted
`odoo.*` paths. Measured: 19.0 core = 13 pseudo-modules, 306 classes, 2.969 functions in ~1,5 s.

Three traps this mode exists to survive, all found by verifying instead of assuming:

- **`class BaseModel(metaclass=MetaModel)` has no bases.** Recording only `node.bases` printed an
  empty inheritance for the most important class in the ORM. Keywords are now recorded too
  (`metaclass=MetaModel`).
- **`AbstractModel = BaseModel` is an alias, not a class.** Without recording module-level aliases
  (`kind='pyalias'`) the inventory answered "AbstractModel does not exist" — a confident negative
  about a name people type every day.
- **A class that MOVES between versions defeats path-keyed diffs.** `BaseModel` went from
  `odoo/models.py` (18.0) to `odoo/orm/models.py` (19.0), so `diff --kind methods` reported 188 added
  and nothing else. Use `--by-class`: it keys on the class name and gives the real answer (74 signature
  changes, 61 methods gone from the class, 17 new).

**2. Pick ONE subsystem.** Not "all of Odoo". Documenting 628 modules produces pages nobody reads
and an index that rots. Start where you actually work: `sale.order` → picking → invoice, or
`account.move` posting and reconciliation.

**3. Generate the module pages for that subsystem only**:

```bash
python3 scripts/code_extract.py --odoo-version 19.0 --addons-path … \
  --modules sale,sale_stock,account,stock --markdown --out <wiki-root>/inventory
```

(Non-base majors get their pages under `inventory/modules/<version>/` with a version-suffixed slug,
because wikilinks resolve by basename and two `sale.md` would make every link to them ambiguous.)

**`--modules` is a scoped refresh, not a scoped inventory.** Re-running with `--modules sale` replaces
only those modules' rows and leaves the rest of that version intact (verified: 1.428 modules stay,
counts recomputed, refs merged, and the run prints `PARTIAL refresh: N replaced — 19.0 still holds M`).
Two consequences to keep in mind:

- **A module deleted upstream is only detected by a FULL re-extraction.** A scoped run can only drop
  what it was asked for and did not find.
- **Never use `--modules` for the first extraction of a version.** You would get a one-module
  inventory labelled `19.0`, and every query about anything else would answer as if it did not exist —
  confidently. The extractor warns loudly when it happens.

**4. Write layer B, querying as you go.** For each page: query, read the real code at the cited
lines to understand the *why*, then write the explanation with citations. Reading source is still
required — the inventory tells you *what and where*, never *why*.

**5. Verify against a live instance when the claim is about the composed model** (see below).

**6. Close the loop** as in any ingest: `index.md`, `log.md`, lint.

## Citations

Format: `` `sale/models/sale_order.py:1411`, `SaleOrder._prepare_invoice` @d1d1610 `` — path, line, short SHA. — path, line, short SHA. The SHA is
not decoration: without it a citation silently rots the next time the repo moves.
`code_query.py` prints every row in exactly this shape so it can be pasted straight into a page.

**Inside a page, a bare `path:line` inherits the page's `source_ref`** — repeating the SHA on a hundred
references is noise, and noise is exactly why the rule gets ignored (measured on a real pilot: 68 of
137 references had dropped the SHA within hours of writing). A **delta page still needs the explicit
`@sha` on every reference**, because two commits are in play and a bare one cannot say which.

Verify them, don't trust them: `wiki_cite_check.py <root>` resolves every reference with
`git show <sha>:<path>` and prints the line, so "the citation is present" and "the citation is true"
stop being the same claim. The same pilot found a 18.0 path pinned with a 19.0 SHA, a reference to a
licence header, and an off-by-one onto a blank line — none of which `wiki_lint.py` can see.

Layer-B pages carry in frontmatter:

```yaml
type: subsystem            # or model / mechanism / pattern / synthesis
odoo_version: "19.0"
layer: ce                  # ce | ee | oca | custom — which tree the claims come from
source_repo: $CE19
source_ref: d1d1610332a1596d026fb0a42ec236d1a79c71cc
source_paths: [addons/sale, addons/sale_stock]
inventory_ref: inventory/meta.json
verified_runtime: false    # true only if checked against a live instance, with the date
```

A **delta** page carries both ends instead of one `odoo_version`:

```yaml
type: delta
introduced_in: "17.0"      # first major where the new behaviour is the norm
last_seen_in: "16.0"       # last major with the old one ("" if it never existed before)
layers: [ce, ee]
verified_runtime: false
```

and at least two `@sha` citations in the body, one per side — `wiki_lint.py` errors otherwise.

## What static extraction cannot know

Layer A is the *static* picture. Being explicit about its blind spots is what keeps the wiki
honest — each of these is a way a code wiki confidently lies:

- **The composed field list of a model.** Static extraction sees which modules *would* add fields;
  the real list depends on which modules are **installed**. Only `fields_get` / `ir.model.fields`
  on a live instance knows.
- **`selection_add`** extensions of Selection fields, and the resulting full value list.
- **MRO order of `_inherit` chains** — resolved at registry build time from the dependency graph
  and install order, not from any single file. Static tells you *who* overrides; the effective
  order of `super()` calls needs the loaded registry.
- **`_inherits` delegation**: which fields are reachable through the delegate, and the write path.
- **Runtime `patch()` on JS classes and Python monkeypatching** in `__init__` hooks.
- **What ACLs and record rules a database ENFORCES.** The rows the code ships *are* extracted
  (`security <model>`, and the `acls`/`rules` diff kinds — see below), which is a genuine answer with
  citations. What an install enforces is another thing: the union of the *installed* modules, minus the
  uninstalled ones, plus whatever an admin or Studio edited. Never turn a `security` result into
  "user X can do Y" without a live instance.
- **Studio / DB-side customisations**, `ir.config_parameter`-driven behaviour, and data that
  changes flow (payment terms, fiscal positions, automations).
- **Anything computed from company or context** (`company_dependent`, `groups=` per user).

So: write `verified_runtime: false` by default, and when a claim really matters, check it against a
live instance (MCP / odoo-pilot: `fields_get`, `ir.model.fields`, `ir.rule`) and record the date
and instance in the page. A claim about the *code* can be static; a claim about *behaviour in a
given install* cannot.

## Security: extracted, with a hard boundary

`security/*.csv` files are detected **by header** (a `perm_read`-ish column), not by filename, so a
module shipping ACLs under another name is still caught, and ordinary data CSVs (countries, rates) are
skipped after reading one line. Record rules come from `<record model="ir.rule">` with their
`domain_force`, `groups` and `perm_*`.

Three decisions worth not re-litigating:

- **`eval=` is stored, never evaluated.** `eval="[Command.set([ref('base.group_user')])]"` is a program;
  guessing its result would be a lie. The raw text is what a human can check against the file.
- **`model_ref` keeps the raw xmlid**, and `model` is resolved against the extracted `models` table.
  Reversing `model_sale_order` by replacing underscores is ambiguous — model names contain underscores
  too. Unresolvable refs stay NULL instead of being guessed.
- **An absent `perm_*` means opposite things** in the two tables: on `ir.rule` the field defaults to
  True, on `ir.model.access` there is no default. `code_query.py security` prints the defaulted ones in
  UPPERCASE for exactly that reason.

This is what makes a permissions change visible to `diff` (`PERMS-CHANGED`, `DOMAIN-CHANGED`) — a class
of change that "only fields and functions" filters silently drop, and that breaks production.

## Base version and deltas

Mixing majors inside a page is silent poison. But *forbidding* the mix throws away the most valuable
thing a shop that migrates owns — **what changed between majors** — and pushes it back into a
hand-written cheat sheet that is verified once and expires quietly. So the layout is **asymmetric**:

| | layer A (inventory) | layer B (prose) |
|---|---|---|
| **base version** | full | full: subsystems, models, mechanisms, patterns |
| **older majors** | full (it costs ~30 s each) | **deltas only** — never "how it worked in 15" |

- **The base is declared once**, in the wiki's `CLAUDE.md` and in `inventory/meta.json`
  (`base_version`). Base pages don't stamp a version in every sentence; the noise appears only where
  there is a real difference. `wiki_lint.py` enforces the invariant instead: a base-scoped page that
  mentions another major without `applies_to`/`introduced_in` is flagged `unversioned-claim`.
- **A mechanical difference is a query, not a page.** "This field is gone in the base" is a free,
  always-current `diff`. Writing it as prose builds a *worse* RAG: one more hop, no more synthesis,
  and it starts rotting immediately. A `deltas/` page is earned only when there is something a query
  cannot produce — why it changed, the migration recipe, what breaks silently.
- **Deltas are filed by change, not by version.** `d-attrs-states-removed.md` with
  `introduced_in: 17.0`, not a `v17/` folder: a change that survives three majors would otherwise be
  written three times. `migrations/m-16-to-19.md` is a *view* over the deltas whose `introduced_in`
  falls inside the jump — an aggregation, not new facts.
- **And that view is GENERATED, not written**: `wiki_migration.py --from 17.0 --to 19.0 --write`. A
  hand-written checklist drifts the moment a new delta lands, and a migration checklist that silently
  misses a delta is worse than none. The judgement (order of work, risk, what to grep) lives in a
  `<!-- CURATED -->` block the script preserves; the page's `confidence` is **derived** from the weakest
  delta it aggregates, so a page that looks authoritative because it is generated still declares its
  floor. Name it with majors only: a dot in a page slug breaks link resolution — `Path("m-17.0-to-19.0")
  .stem` silently drops the last `.0`, and every link to that page resolves to nothing.
- **Both ends, both SHAs.** A delta cites the version where the old shape was true *and* the one
  where it changed. `wiki_lint.py` errors on a delta with a single `@sha`: with one citation nothing
  has been shown to change.
- **Every claim shows its layer.** A method that only exists in EE is useless to a CE client, and
  an OCA-only field is a bug waiting to happen. `code_extract.py` records the layer per module and
  a separate SHA per repo *and per version*, so citations to EE 17 and CE 19 resolve to different
  commits — keep that visible in the wiki.
- **Cross-reference, don't re-derive.** `odoo-deep-knowledge` already holds ~5.000 gotchas verified
  against source, and `odoo-version-api-reference` holds the API diffs. Link to them; a page that
  paraphrases them adds a second copy that will drift.

### The two cross-version queries

```bash
# in which majors does this exist, with what shape, and where did it change?
python3 scripts/code_query.py <inv> lifespan sale.order.commitment_date
python3 scripts/code_query.py <inv> lifespan _prepare_invoice --kind method

# what changed between two majors: summary first, then drill down
python3 scripts/code_query.py <inv> diff 17.0                                  # counts per kind
python3 scripts/code_query.py <inv> diff 17.0 --kind fields  --model sale.order
python3 scripts/code_query.py <inv> diff 17.0 --kind methods --module sale     # SIG/DECO/BODY-CHANGED
python3 scripts/code_query.py <inv> diff 17.0 --kind xmlid   --module sale     # deleted views
```

`lifespan` is the one that replaces a hand-kept cheat sheet: the real question when writing code is
never "what happened in 17", it is **"this thing I am about to write, since when is it valid"** — and
that is a `GROUP BY` over the version axis, with a citation per row.

### What the diff cannot see (the lie this layout makes easy)

- **Same signature, different behaviour.** `body_sha1` is a normalised hash of the method body
  (AST-unparsed, docstring dropped, so a reformat is not a change), and `BODY-CHANGED` is therefore
  a **pointer to go read the method** — never a claim about *what* changed. Renamed locals do flip
  it; that false positive is the price of catching real logic changes.
- **A symbol present in both majors is not proof it means the same thing.** With six versions in the
  inventory this is the easiest wrong answer to give: structurally correct, functionally misleading.
  Semantics are prose work, cited to both SHAs.
- **Absent ≠ never existed.** The inventory only knows the addons-paths it was given: a module in EE
  but extracted CE-only reads as absent. `lifespan` prints that caveat with every result.
- Everything in [What static extraction cannot know](#what-static-extraction-cannot-know) applies to
  each side of a diff, twice.

## Page taxonomy

| Type | Lives in | Earns a page when… | Must contain |
|---|---|---|---|
| module inventory | `inventory/modules/` | generated; never authored | — |
| subsystem | `wiki/subsystems/` | a flow crosses ≥3 modules | the flow, the hand-off points, where it breaks |
| model | `wiki/models/` | the model is load-bearing and heavily extended | who defines/extends it, key fields, hot methods, extension etiquette |
| mechanism | `wiki/mechanisms/` | machinery you'd otherwise re-derive (compute/depends, cache, cron, MRO) | how it actually works + the failure it causes when misused |
| pattern | `wiki/patterns/` | there is a right way and a tempting wrong way | the correct shape, the anti-pattern, why |
| delta | `wiki/deltas/` | a difference between majors has a *story* a `diff` can't produce | both shapes with **both** SHAs, why it changed, the migration recipe, what breaks silently |
| migration | `wiki/migrations/` | you actually do that version jump | the ordered list of deltas in the jump + what to check first. No new facts |
| synthesis | `wiki/syntheses/` | a query answer worth keeping | question, answer, citations, what would change it |

A model page's job is *not* to duplicate the field table — that is layer A, and duplicating it
guarantees drift. Link to the generated page and spend the words on what the fields *mean* and
which ones are traps.

## Drift

The failure mode of a code wiki is silent expiry: the repo advances, the prose stays, and it reads
just as confident as before.

```bash
python3 scripts/code_query.py <wiki-root>/inventory drift
python3 scripts/code_query.py <wiki-root>/inventory drift --version 19.0   # just the base
```

Compares each recorded SHA against the working tree's HEAD, **per (version, layer)**; when stale it
lists how many source files changed and in which top-level modules — which tells you exactly which
pages to re-verify first. Exits non-zero when stale, so it fits in a pre-commit hook or a cron.

With several majors extracted, read the output by version: old majors are frozen in practice, so
drift there usually means someone moved the branch. If the base is current and only old versions are
stale, the base prose is safe and **the deltas are what rot**.

Then: re-run `code_extract.py`, and treat layer-B pages whose `source_ref` is older than the new
one as **suspect until re-checked** — not wrong, suspect. `wiki_lint.py` reports the mismatch as
`stale-ref`. Re-verify in dependency order, and log the pass.

### When the base itself moves (weekly resync)

Old majors are frozen; **the base is a live branch**, so this is the recurring case. What survives
and what doesn't:

- **Citations do not break.** `git show <sha>:path` resolves as long as the commit stays reachable —
  that is what pinning buys. Verified: a week-old citation still resolves.
- **A line number without its SHA does break.** `:1411 @<old sha>` is checkable forever; `:1411`
  alone is a lie the moment the file moves.
- **What ages is the prose**, and only where the cited code actually changed.

Measured on odoo/odoo 19.0, a real 7-day window (174 commits, 293 changed py/xml/js files across 86
modules) restricted to `sale`+`sale_stock`+`stock`+`account`: **5 fields added, 3 methods added, 14
methods BODY-CHANGED, 0 SIG-CHANGED, 0 removals of anything, 0 xmlid or model changes.** Within a
stable major, a week of upstream breaks nothing structural. Budget the resync as minutes, not a
re-read.

The protocol, with no new tooling:

```bash
# 1. did it actually move?
python3 scripts/code_query.py <inv> drift

# 2. sparse worktree at the OLD sha — retroactive, no need to have snapshotted anything
git -C $CE19 worktree add --no-checkout --detach /tmp/wt-old <old-sha>
cd /tmp/wt-old && git sparse-checkout init --cone \
  && git sparse-checkout set addons/sale addons/account addons/stock && git checkout
#    measured: 4 modules ≈ 108 MB, seconds

# 3. old tree as a temporary LABEL (the version axis is free text), new tree as the base
python3 scripts/code_extract.py --odoo-version "19.0@<old-sha-short>" \
  --addons-path /tmp/wt-old/addons --modules sale,account,stock --out <inv>
python3 scripts/code_extract.py --odoo-version 19.0 --base \
  --expect-ref ce=<new-sha> --addons-path … --out <inv>

# 4. symbol-level diff, then clean up
python3 scripts/code_query.py <inv> diff "19.0@<old-sha-short>" --to 19.0
python3 scripts/code_extract.py --forget "19.0@<old-sha-short>" --out <inv>
git -C $CE19 worktree remove --force /tmp/wt-old
```

Triage the diff by cost, not by size:

| what the diff says | what it costs you |
|---|---|
| removals, `SIG-CHANGED`, `DECO-CHANGED` on a cited symbol | the page is **suspect**: re-read, re-cite, and if the meaning changed **record the contradiction with both dates** instead of overwriting |
| additions on a documented model | one line, or a new page — decide, or park it in `open-questions.md` |
| `BODY-CHANGED` | a pointer. Read it only if a page claims *behaviour* about that method |
| anything in modules you never documented (~90% of churn) | nothing: refresh the refs and log it |

**`--expect-ref` is not optional in this flow.** The extractor reads the *working tree* and stamps
HEAD's SHA: hand it a SHA that is not checked out and you get a perfectly self-consistent inventory
*of a different tree*, with no error. The same check refuses on a **dirty** addons path (uncommitted
files are parsed but the recorded SHA does not contain them, so citations into them are wrong);
`--allow-dirty` accepts it and the count is recorded in the version row.

`--forget` exists so temporary snapshot labels don't accumulate; it refuses to forget the base.

## Anti-patterns

- **Ingesting the whole tree into layer B.** Token bonfire; 90% of the pages are never read.
- **Retyping structure into prose.** The one rule. Everything else is recoverable; this isn't.
- **Citations without a SHA.** They rot invisibly.
- **Mixing majors or layers in one page.** Produces answers that are right for a tree nobody has.
- **Writing "how it worked in <old major>" prose.** Six wikis to maintain and nobody reads five of
  them. Old majors get deltas, or a query plus reading the code at that SHA.
- **A delta page for a mechanical difference.** If `diff` already answers it, the page adds a hop and
  starts drifting the day it is written.
- **A delta with one citation.** Nothing has been shown to change; it is an undated opinion.
- **"We'll add the older versions later."** That is how this dies: you keep a base-only wiki plus the
  hand-written cheat sheet you already had. Extract them up front (it is minutes) and tie the deltas
  to the pilot below, so their absence *fails* instead of being postponed in silence.
- **Hand-editing `inventory/`.** It is regenerated; your edit disappears and your trust in the
  layer goes with it.
- **Treating static output as runtime truth** (see the blind spots above).
- **Skipping `drift`** because the wiki "was fine last month".

## Piloting it before trusting it

Do this before scaling to more subsystems, because it is cheap and it is the only honest check:

1. Pick one subsystem and build layer A (every major you care about) + 10-20 layer-B pages on the
   base.
2. Write **20 questions with known answers**: ~8 structural ("which modules override
   `_prepare_invoice`", "what type is `sale.order.commitment_date`"), ~7 behavioural ("what breaks
   if I compute a stored field from a non-stored one"), and **5 cross-version** ("since which major
   does X exist", "what breaks in this flow going 16 → base", "which views in this module were
   deleted"). The cross-version five are not optional: they are what makes a missing delta *fail*
   instead of quietly never happening.
3. Answer them from the wiki only. Score two things separately: **answer correct**, and
   **citation resolves to the line that proves it** (both sides, for the cross-version ones).
4. Compare against the baseline: grepping the source directly, and asking
   `odoo-deep-knowledge`.

If the structural half isn't near-perfect, the inventory or the citation rule is being bypassed —
fix that before writing another page. If the behavioural half doesn't beat the baseline, the wiki
is prose-for-its-own-sake and the honest move is to say so and stop.

A cheap first check that needs no prose at all: run `lifespan` on the claims of a cheat sheet you
already trust and see whether it reproduces them. Measured while building this: `res.groups`
`category_id` present 16.0-18.0 at `odoo/addons/base/models/res_users.py:191`, `Groups.category_id` @99f5a33c64f8, absent in 19.0, with
`privilege_id` appearing at `odoo/addons/base/models/res_groups.py:36`, `ResGroups.privilege_id` @d1d1610332a1 — the chuleta claim,
regenerated with citations instead of recalled.
