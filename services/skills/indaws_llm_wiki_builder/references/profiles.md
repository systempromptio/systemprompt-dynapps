# Domain profiles

The pattern is domain-agnostic; the *taxonomy* is not. A book companion and an investment thesis
want different page types, different ingest rhythms and different definitions of "done". Pick the
closest profile, adapt it with the user, and write the result into the wiki's `CLAUDE.md` — the
profile is a starting point, the schema is the law.

`wiki_scaffold.py --profile <name>` seeds the layout and schema draft for: `personal`, `research`,
`book`, `team`, `code`, `generic`.

---

## `personal` — self, goals, health, psychology

**Sources:** journal entries, articles, podcast notes, therapy notes, lab results, book
highlights, workout/sleep exports.

**Directories:** `wiki/themes/` (recurring patterns: sleep, motivation, conflict at work),
`wiki/practices/` (things tried, and whether they worked), `wiki/people/`, `wiki/timeline/`
(dated episodes), `wiki/syntheses/`.

**What earns a page:** a pattern that has recurred three times. A practice you've actually tried.
A person who shows up across entries.

**Special care:** this wiki is sensitive by nature. Keep it local, keep it in a private repo, and
don't push it anywhere the user didn't ask for. Be careful with framing — you are writing about a
person, using the language they use about themselves. Contradictions here are usually growth, not
error: date them and keep both, since the change *is* the insight.

**Lint focus:** patterns claimed on one entry only; conclusions that hardened into identity claims
without evidence; practices marked as working with no follow-up.

---

## `research` — going deep on a topic for weeks or months

**Sources:** papers, preprints, reports, datasets, talks, expert threads.

**Directories:** `wiki/concepts/`, `wiki/entities/` (authors, labs, orgs, systems),
`wiki/methods/`, `wiki/findings/` (one page per claim, with the evidence for and against),
`wiki/syntheses/` (comparisons, literature maps), plus a prominent `wiki/overview.md` holding the
**current thesis**.

**What earns a page:** any claim that two sources address, any method you'd need to explain twice,
any result the thesis depends on.

**Special care:** this is the profile where contradiction handling earns its keep — papers
disagree constantly. Record effect sizes, sample sizes and dates, not vibes. Distinguish "the
paper found" from "the paper claims". Track retractions and replication status if the field has
them. `confidence:` should mostly not be `high`.

**Lint focus:** thesis drift (overview no longer matches findings); load-bearing claims resting on
one preprint; findings nobody has revisited since source #3.

---

## `book` — reading a book or watching a series, chapter by chapter

**Sources:** chapters you file as you read (your notes, plus quotes).

**Directories:** `wiki/characters/`, `wiki/places/`, `wiki/themes/`, `wiki/chapters/` (one page
per chapter, doubling as the source page), `wiki/threads/` (plot threads and their state),
`wiki/syntheses/`.

**What earns a page:** a character who acts twice. A place where something happens. A theme that
recurs.

**Special care:** **spoiler discipline** — pages should reflect only what has been read so far, or
mark later revelations explicitly. Getting this wrong ruins the book, which makes it the one
convention to state loudly in the schema. Keep a "state as of chapter N" line on thread pages;
that's what makes the wiki useful while reading rather than after. Fan wikis like Tolkien Gateway
are the target shape: thousands of interlinked pages, except you get it as a by-product of reading.

**Lint focus:** threads left dangling; characters introduced and never linked; "state as of"
markers that fell behind the reading.

---

## `team` — internal wiki maintained by an LLM

**Sources:** meeting transcripts, Slack threads, project docs, customer calls, decision records,
incident write-ups.

**Directories:** `wiki/projects/`, `wiki/decisions/` (one page per decision: what, when, why,
who, superseded-by), `wiki/people/`, `wiki/customers/`, `wiki/processes/`, `wiki/syntheses/`.

**What earns a page:** a decision. A project. A recurring process. A customer.

**Special care:** the value here is **decision provenance** — six months later nobody remembers
why, and the wiki does. Attribute claims to who said them and when, since in team knowledge "who
thinks this" is part of the fact. Watch confidentiality: a wiki fed by customer calls is a wiki
full of things not everyone should read; agree the boundary before ingesting, and keep humans in
the loop for review if the wiki is shared. Never let it become the place where personnel opinions
accumulate.

**Lint focus:** decisions with no outcome recorded; projects whose status is months stale;
processes documented from one meeting and never verified.

---

## `code` — a source tree (Odoo CE/EE/OCA, client modules)

**Read [code-profile.md](code-profile.md) before starting one of these.** It is the profile with
the most specific rules, because "know the structure without being wrong" is an extraction problem,
not a synthesis one.

**Sources:** the code itself — referenced by repo + commit SHA, never copied into `raw/`.

**Directories:** `inventory/` (layer A: generated by `code_extract.py`, never hand-edited) plus
`wiki/subsystems/`, `wiki/models/`, `wiki/mechanisms/`, `wiki/patterns/`, `wiki/deltas/`,
`wiki/migrations/`, `wiki/syntheses/`.

**What earns a page:** a flow crossing ≥3 modules; a load-bearing model; machinery you'd otherwise
re-derive; a right-way/wrong-way pair. Never a page that just restates the generated field table,
and never a delta page for a difference `code_query.py diff` already answers.

**Special care:** the whole profile hangs on one rule — **you never retype structure**; signatures,
field lists and `_inherit` chains come from a layer-A query with its `file:line @sha` citation.
Beyond that: **one base major for the prose and every other major as a queryable delta** (layer A
holds them all; `deltas/` pages cite both SHAs), every claim tagged with its layer (ce/ee/oca/custom),
`verified_runtime: false` until checked against a live instance, and a `drift` check whenever the
repo moves. Static extraction cannot know the composed registry — the blind-spot list is in
[code-profile.md](code-profile.md).

**Lint focus:** `stale-ref` (inventory SHA moved, checked per version), structural prose without a
citation, `delta-single-sha` (a delta that cites only one side), `unversioned-claim` (a base-scoped
page talking about another major), pages whose `source_paths` no longer exist, subsystem pages that
never got a runtime verification.

---

## `generic` — everything else

Competitive analysis, due diligence, trip planning, course notes, hobby deep-dives, a legal
matter, a house renovation. Start with `entities` / `concepts` / `sources` / `syntheses` and let
the taxonomy emerge from the first five ingests, then consolidate deliberately in a lint pass and
write it into the schema. Emergent structure beats a taxonomy invented before you've read
anything — but only if someone actually consolidates it later.

---

## Note on Odoo / client projects: which tool for which knowledge

Three things get confused here, and mixing them is how both copies become untrustworthy. Draw the
boundary explicitly in the wiki's schema:

| Knowledge | Where it belongs |
|---|---|
| Client's code/config **inventory** (regenerable from repo + DB) | `.ai-memory/` via `odoo-context-briefing` |
| **Structure** of a source tree (models, fields, methods, per SHA) | `code` profile, layer A — generated by `code_extract.py` |
| **Why/how** of code: mechanisms, flows, right-vs-wrong ways | `code` profile, layer B — the wiki |
| **Business context**: decisions, rejected options, how the client really operates | `team`-style wiki (not regenerable from anything) |
| Odoo gotchas already verified against source | `odoo-deep-knowledge` — link, don't re-derive |

The rule of thumb: if a script can regenerate it, a human (or an LLM) must not be writing it by
hand; and if no artifact can regenerate it, it is exactly what the wiki is for.
