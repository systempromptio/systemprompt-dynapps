# Page formats and conventions

Conventions exist so that a session six months from now writes pages that fit the ones you wrote
today, and so `wiki_lint.py` can check mechanically what a human shouldn't have to. They are
defaults: if a wiki needs different ones, change them **in that wiki's `CLAUDE.md`** so they
become the local law.

- [Naming](#naming)
- [Frontmatter](#frontmatter)
- [Links and citations](#links-and-citations)
- [Contradictions and superseded claims](#contradictions-and-superseded-claims)
- [Page types](#page-types) — source · entity · concept · synthesis · overview · index · log · open-questions

## Naming

| Page type | Path | Example |
|---|---|---|
| Source | `wiki/sources/s-YYYY-MM-DD-slug.md` | `s-2026-08-03-karpathy-llm-wiki.md` |
| Entity | `wiki/entities/slug.md` | `andrej-karpathy.md`, `obsidian.md` |
| Concept | `wiki/concepts/slug.md` | `retrieval-augmented-generation.md` |
| Synthesis | `wiki/syntheses/slug.md` | `rag-vs-compiled-wiki.md` |

kebab-case, no dates except on source pages, no spaces (they survive Obsidian but not shell
tools). One subject per page: if a page needs "and" in its title, it's two pages. The `s-`
prefix on sources means a citation is recognisable without opening it.

## Frontmatter

Every page carries YAML frontmatter, and it **must start at byte 0** — a blank line or stray
character before the opening `---` and every tool (Obsidian Dataview, the lint script) sees no
metadata at all. It powers Obsidian Dataview queries, lets `wiki_lint.py` detect staleness and
orphans, and gives a future session the page's provenance without reading the whole thing.

```yaml
---
type: source | entity | concept | synthesis | overview | meta
title: Human readable title
created: 2026-08-03
updated: 2026-08-03
sources: [s-2026-08-03-karpathy-llm-wiki]   # source pages this page rests on
tags: [knowledge-management, llm]
status: stub | draft | solid
confidence: low | medium | high
---
```

Source pages add provenance:

```yaml
type: source
source_path: raw/2026-08-03-karpathy-llm-wiki.md
url: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
author: Andrej Karpathy
published: 2026-08-03
retrieved: 2026-08-03
```

Field discipline that actually matters:

- **`updated`** — bump it whenever you change content. It is how lint and the human spot pages
  the wiki has left behind.
- **`sources`** — the page's evidence base. A non-source page with an empty `sources` list is
  either an unsourced opinion or a stub; both should be visible as such.
- **`status`** — `stub` is honest and useful: it tells the next session where to invest.
- **`confidence`** — `low` on a single-source claim is not weakness, it is the wiki working. See
  the honesty rule in SKILL.md.

## Links and citations

- **Wikilink** between pages: `[[concept-slug]]`, or `[[concept-slug|readable text]]`. Obsidian
  resolves them, the graph view renders them, and `wiki_lint.py` builds the link graph from them.
- **Citation** of evidence: put the source page inline right after the claim.

```markdown
Compiling knowledge once and maintaining it beats re-deriving it per query, because the
cross-references and contradiction flags already exist (→ [[s-2026-08-03-karpathy-llm-wiki]]).
```

- **Quote** with a location so it can be verified: `> "…" (→ [[s-…]], § Architecture)`.
- **Links go both ways.** After writing `[[x]]` in page A, ask whether X should point back at A.
  One-directional linking is how orphan farms grow.
- Never cite `raw/` files directly from concept or entity pages — cite the source page, which
  cites the raw file. One indirection, so re-reading a source updates the wiki in one place.

## Contradictions and superseded claims

The wiki's most valuable content is knowledge that *changed*. Never resolve a contradiction by
deleting the loser.

```markdown
## Claim: throughput scales linearly with workers

**Current (2026-08-03):** scales sub-linearly past 8 workers; lock contention dominates
(→ [[s-2026-08-03-benchmark-report]]).

**Superseded (2026-05-11):** reported linear scaling to 32 workers
(→ [[s-2026-05-11-vendor-whitepaper]]). Kept because the vendor claim is still what the docs
say; the benchmark measured a newer build, which is why it's believed over the whitepaper.
```

When you can't decide which side is right, say so explicitly, put it in `open-questions.md`, and
tell the user — that is a decision they may have context for.

## Page types

### Source page — `wiki/sources/s-*.md`

Purpose: the citable record of one source. Dense enough that any wiki claim citing it can be
verified here.

```markdown
---
type: source
title: "LLM Wiki (gist)"
created: 2026-08-03
updated: 2026-08-03
source_path: raw/2026-08-03-karpathy-llm-wiki.md
url: https://gist.github.com/karpathy/…
author: Andrej Karpathy
published: 2026-08-03
retrieved: 2026-08-03
tags: [knowledge-management]
status: solid
confidence: high
---

# LLM Wiki (gist)

**What it is:** an idea file describing a pattern for LLM-maintained personal knowledge bases.

## Key claims
- RAG re-derives knowledge per query; a wiki compiles it once and keeps it current.
- Three layers: immutable raw sources, LLM-owned wiki, co-evolved schema file.
- Maintenance cost, not reading cost, is why humans abandon wikis.

## Notable quotes
> "Obsidian is the IDE; the LLM is the programmer; the wiki is the codebase." (§ The core idea)

## Numbers and specifics
- A single source ingest may touch 10-15 wiki pages.
- Index-first retrieval works to ~100 sources / hundreds of pages without embeddings.

## Relevance to this wiki
Founding source for [[llm-wiki-pattern]]; contradicts nothing yet.

## Links
[[llm-wiki-pattern]] · [[retrieval-augmented-generation]] · [[andrej-karpathy]] · [[obsidian]]
```

### Entity page — `wiki/entities/*.md`

A person, org, product, tool, place, character. Stable identity, accumulating facts.

Sections that tend to earn their place: what it is (one line) · why it matters here · key facts
(dated, cited) · relationships to other entities · open questions · appears in (source pages).

### Concept page — `wiki/concepts/*.md`

An idea, mechanism, theme, method. This is where synthesis lives, so it is the page type most
worth writing well.

Sections: definition in your own words · why it matters · how it works · evidence for /
against (cited, dated) · relation to neighbouring concepts (with links, and how they differ) ·
open questions.

The test of a good concept page: someone reading only this page can use the idea, argue against
it, and knows what the wiki is unsure about.

### Synthesis page — `wiki/syntheses/*.md`

A comparison, an analysis, a filed-back answer. Includes the question it answers and, if it came
from a query, the date — a synthesis is a snapshot and may age.

```markdown
---
type: synthesis
title: "RAG vs compiled wiki"
created: 2026-08-03
updated: 2026-08-03
sources: [s-2026-08-03-karpathy-llm-wiki, s-2026-06-12-rag-eval-paper]
status: draft
confidence: medium
---

# RAG vs compiled wiki

**Question:** when does maintaining a wiki beat retrieving from raw documents?

| Axis | RAG | Compiled wiki |
|---|---|---|
| Cost per query | re-derives synthesis | reads existing synthesis |
| Cross-document reasoning | rebuilt each time | precomputed, cited |
| Freshness | automatic | depends on maintenance |
| Failure mode | plausible fragments | stale claims |

**Where the wiki wins:** … (→ [[s-2026-08-03-karpathy-llm-wiki]])
**Where it loses:** … (→ [[s-2026-06-12-rag-eval-paper]])
**Open:** no measurement of the crossover point in this wiki → [[open-questions]]
```

### `wiki/overview.md`

One page: the current picture. What the wiki collectively believes, the main threads, the biggest
open questions, links to the hub pages. Rewrite it when the picture moves — it's the page a human
reads first, and the fastest way for a new session to get oriented.

### `index.md`

Content-oriented catalog, organised by category. One line per page: link, one-line summary,
optionally date or source count. This is the retrieval system — keep the summaries genuinely
descriptive, since they are what a future session decides from.

```markdown
# Index

## Concepts
- [[llm-wiki-pattern]] — LLM-maintained wiki as an alternative to per-query RAG (4 sources)
- [[retrieval-augmented-generation]] — retrieve-then-generate baseline and its limits (2 sources)

## Entities
- [[andrej-karpathy]] — author of the LLM Wiki pattern (1 source)

## Sources
- [[s-2026-08-03-karpathy-llm-wiki]] — gist describing the pattern · 2026-08-03
```

### `log.md`

Append-only chronology. Consistent prefix so unix tools work — that's the point of the format:

```markdown
## [2026-08-03] ingest | LLM Wiki (gist)
pages: s-2026-08-03-karpathy-llm-wiki, [[llm-wiki-pattern]] (new), [[obsidian]]
notes: founding source; no contradictions yet

## [2026-08-04] query | when does a wiki beat RAG?
filed: [[rag-vs-compiled-wiki]]
```

`grep "^## \[" log.md | tail -5` → last five operations. Use `scripts/wiki_log.py` so the format
never drifts.

### `open-questions.md`

Known gaps, unresolved contradictions, sources to find, questions to investigate. The wiki's
honest self-assessment, and the most useful page for deciding what to do next. Cross items off
with the date and the source that closed them, rather than deleting — the record of what you were
once unsure about is itself information.
