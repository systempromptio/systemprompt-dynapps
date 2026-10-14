# Operations: Ingest, Query, Lint

The three verbs that keep an LLM wiki alive. Read the section you need; they are independent.

- [Ingest](#ingest) — a new source enters the wiki
- [Query](#query) — a question is answered from the wiki
- [Lint](#lint) — the wiki is health-checked
- [Batch ingest](#batch-ingest-when-supervision-is-not-affordable)
- [Sources with images](#sources-with-images)
- [Sources you have to fetch yourself](#sources-you-have-to-fetch-yourself)
- [What a failing wiki looks like](#what-a-failing-wiki-looks-like)

---

## Ingest

The operation that makes or breaks the wiki. Writing the summary is the easy 20%; the other 80%
is rewiring the wiki around what the source changed.

### 1. Land the source in `raw/`

If the user handed you a URL, fetch it to markdown and save it under `raw/` (with the images if
they matter — see [below](#sources-with-images)). If they handed you a file outside the wiki,
copy it in. The wiki must be self-contained: a source that lives only behind a URL will be a
dead citation within a year.

Name it so a human can scan the directory: `raw/YYYY-MM-DD-slug.md`, date = publication date if
known, otherwise the day you ingested it.

### 2. Read it, then talk before writing

Read the whole source. Then tell the user what you found — the key takeaways, what surprised
you, what contradicts what the wiki already says, and what you propose to write. **Wait for
their steer.**

This step feels skippable and isn't. The user knows why they picked this source; they will
redirect your emphasis in one sentence. Discovering that after you have written 12 pages costs
far more than asking. (If the user explicitly asked for unsupervised/batch ingest, skip the
discussion — that's their call, and it's what [batch mode](#batch-ingest-when-supervision-is-not-affordable) is for.)

### 3. Write the source page

One page per source in `wiki/sources/`, named `s-YYYY-MM-DD-slug.md` — the `s-` prefix makes
source citations recognisable at a glance in any page. Format in
[page-formats.md](page-formats.md).

The source page is the citation target for every claim the source supports, so it must contain
the claims themselves, not a vague abstract. Concrete numbers, names, dates, quotes with
locations. If a future session cannot verify a wiki claim against this page, the page is too
thin.

### 4. Rewire — the actual work

Walk the source's content against the existing wiki and update everything it touches:

- **Entities mentioned** — update their pages with what this source adds. Create pages for
  entities that now matter enough to have one.
- **Concepts touched** — revise the explanation, add nuance, add the example, note the
  disagreement. Promote a concept to its own page when it starts appearing in three or more
  pages, or when it needs explaining more than once.
- **Contradictions** — where the source disagrees with an existing claim, record both with
  dates and state which you currently believe and why (superseded-claim format in
  [page-formats.md](page-formats.md)). This is the highest-value output of an ingest and the
  first thing sloppy ingests drop.
- **Cross-references** — links go *both* ways. A new entity page nobody links to is born an
  orphan. When you write `[[x]]` into page A, ask whether page X should link back to A.
- **`wiki/overview.md`** — if the source moves the overall picture, move it. If the thesis
  didn't change, don't touch the page just to look busy.
- **`open-questions.md`** — cross off what this source answered; add what it opened up.

A substantive source touches 10-15 pages. That number is a smell test, not a quota: don't pad
pages with filler to hit it, and do tell the user plainly when a source turned out to be thin
("this added one data point, nothing else moved").

### 5. Close the loop

```bash
python3 scripts/wiki_log.py <wiki-root> --op ingest --title "<source title>" \
  --body "pages: s-…, [[entity-a]], [[concept-b]] | contradiction: X vs Y | new: [[concept-c]]"
```

- Update `index.md`: add the new pages with their one-line summaries, refresh the summaries of
  pages whose scope changed.
- Append to `log.md` (the script keeps the `## [YYYY-MM-DD] op | Title` prefix that makes the
  log greppable — hand-writing entries drifts from the format and breaks
  `grep "^## \[" log.md | tail -5`).
- Run `python3 scripts/wiki_lint.py <wiki-root>` and fix what it reports. Cheap here,
  expensive in six months.
- Tell the user what changed, in a few lines: pages touched, contradictions found, questions
  opened. Not a wall of diff.

### Ingest failure modes

| Symptom | What actually went wrong |
|---|---|
| Only the source page changed | You indexed instead of ingesting. The synthesis didn't move. |
| Contradiction silently resolved | You overwrote history. Restore both versions with dates. |
| New pages nobody links to | You forgot inbound links; lint will report them as orphans. |
| Every page "updated" | Padding. Touching a page without adding information is noise in git history. |
| Index/log skipped | The work is invisible to the next session. It effectively didn't happen. |

---

## Query

### 1. Retrieve, cheapest first

Read `index.md`. At moderate scale (~100 sources, a few hundred pages) the index plus targeted
reads genuinely outperforms embedding infrastructure, because your index entries were written
by something that understood the pages. Then drill into the 3-8 pages that look relevant.

Only when the index isn't enough: `scripts/wiki_search.py` (BM25), then rung 3 if the wiki has
outgrown that — the local vector store (`wiki_index.py` + `wiki_vsearch.py`, embeddings delegable
to another machine) or `qmd`. Ladder, measured limits and setup in [search-qmd.md](search-qmd.md).

### 2. Answer with citations

Answer from the wiki, citing pages: `[[concept-x]]`, and through them the sources
`(→ [[s-2026-08-03-slug]])`. Two things you owe the user:

- **When the wiki doesn't know, say so.** "The wiki has nothing on this" is a useful answer; it
  tells the user what to source next. Filling the gap from your own prior knowledge, unmarked,
  poisons the wiki's credibility — if you do use outside knowledge, label it as not-from-wiki.
- **When the wiki's answer rests on one source, say that too.** Confidence is part of the answer.

Output format follows the question: prose, a comparison table, a chart, Marp slides, a canvas.
Ask if it's ambiguous and expensive.

### 3. File it back if it's worth keeping

The insight that makes queries compound: **a good answer is a wiki page.** A comparison table
you just built, a connection you discovered, an analysis the user reacted to — that is new
synthesis, and it dies in chat history unless you file it.

File into `wiki/syntheses/` when the answer (a) took real synthesis across pages, (b) will
plausibly be asked again, or (c) the user reacted to it. Then index it and log the query. Skip
for trivial lookups — filing everything turns `syntheses/` into landfill.

---

## Lint

### 1. Mechanical pass (script, ~0 tokens)

```bash
python3 scripts/wiki_lint.py <wiki-root>            # human report
python3 scripts/wiki_lint.py <wiki-root> --json     # machine-readable
```

It reports: broken wikilinks, orphan pages, index drift (both directions), malformed or missing
frontmatter, citations to source pages that don't exist, raw sources never ingested, malformed
log entries, stubs and `TODO` markers.

Fix these first. They are unambiguous, and a wiki full of mechanical rot makes the judgement
pass unreadable.

### 2. Judgement pass (only you can do this)

Read the index, the overview, and sample widely — including pages the script says are fine.
Look for:

- **Contradictions between pages** that nobody flagged. Two pages, both confident, incompatible.
- **Stale claims**: a page still asserts something a newer source superseded. The script can see
  dates; only you can see meaning.
- **Concepts without a page**: something referenced across many pages, explained nowhere. Promote it.
- **Near-duplicate pages**: two pages circling the same idea. Propose a merge — sprawl makes
  retrieval worse, and it's the most common slow death of a growing wiki.
- **Thin synthesis**: `overview.md` still reads like it did at source #5 though 40 have landed.
- **Evidence gaps**: load-bearing claims resting on one source, or on the weakest source.
- **Questions worth asking next**, and **sources worth finding** — a web search now often fills
  a gap the user didn't know they had. This is where lint earns its keep: it turns maintenance
  into direction.

### 3. Report and record

Give the user a prioritised list — what you fixed, what needs their decision (merges, which of
two contradictory sources to trust, which gap to source next). Update `open-questions.md`, then:

```bash
python3 scripts/wiki_log.py <wiki-root> --op lint --title "health check" --body "…"
```

Cadence: every ~10 ingests, or when something feels off. It is cheap and it compounds; a wiki
that is never linted decays into exactly the folder-of-summaries the pattern exists to avoid.

---

## Batch ingest (when supervision is not affordable)

Sequential ingest with discussion produces better wikis. But 40 papers at once is real, and
this is how to do it without corrupting the wiki:

**Phase 1 — extract, in parallel.** One subagent per source. Each one may write **only its own
`wiki/sources/s-*.md` page**, and returns structured takeaways: entities touched, concepts
touched, claims with citations, contradictions it suspects, pages it proposes to create.

**Phase 2 — integrate, serially.** You apply the rewiring yourself, one source at a time, in
date order (so contradictions resolve in the right direction), then update `index.md` and
`log.md` once at the end.

Why the split: `index.md`, `log.md`, `overview.md` and popular entity pages are exactly what
every agent needs to touch, so parallel writers collide there and silently lose each other's
edits. Also, cross-source contradictions are only visible to whoever sees all the extractions —
that's you, in phase 2.

Before launching a fan-out, check the context cost (see the pre-flight rule in the user's global
config): N subagents × a large context is how a batch ingest becomes surprisingly expensive.

---

## Sources with images

Images carry real information in slide decks, papers and clipped articles.

- Save them locally to `raw/assets/` — a URL will rot, and a broken image is an unrecoverable
  citation. Obsidian: Settings → Files and links → *Attachment folder path* = `raw/assets/`, and
  bind *Download attachments for current file* to a hotkey (Karpathy's tip; Ctrl+Shift+D).
- You cannot read markdown and its inline images in one pass. Read the text first, then view the
  images that matter as a second step, then write. Clunky, but it's the difference between
  "Figure 3 shows the trend" and knowing what the trend is.
- Reference images from wiki pages by relative path so both Obsidian and git stay happy.

---

## PDF sources — nunca abrir a ciegas

`Read` sobre un PDF renderiza **cada página como imagen** (~1.7K tokens/página): un manual de 300
páginas son ~500K tokens por mirarlo. El orden es probar, extraer, y sólo mirar lo imprescindible.

1. `pdf_probe.py <fichero>` — 0 tokens: nº de páginas, si hay capa de texto o es escaneo, TOC, y por
   página caracteres / imágenes ráster grandes / paths vectoriales / tablas, más un plan de lectura y
   la comparación de coste texto-vs-imagen.
2. `pdf_extract.py sections` → rangos de página del TOC · `text --pages/--section` para lo que vas a
   leer · `render` sólo para las páginas que hay que **ver** · `ocr` si es escaneo (tesseract).

Dos cosas que esto compra y que mirar por encima no da:

- **Los esquemas y las curvas suelen ser vectoriales**, así que `pdfimages` no reporta nada
  justamente en las páginas que merece la pena mirar. `probe` cuenta paths, no sólo ráster.
- **El texto extraído desalinea las tablas multi-columna.** Cualquier número que vayas a citar se
  confirma haciendo `render` de esa única página.

Texto derivado y PNGs van a `.cache/`: **nunca a `raw/`**, que es inmutable, y nunca a git.

---

## Sources you have to fetch yourself

When the user gives a URL rather than a file: fetch it, convert to markdown, save under `raw/`
with the original URL and the retrieval date in the frontmatter. Note in the source page if the
fetch was partial (paywall, JS-rendered page, truncated) — a silently truncated source becomes a
confidently wrong wiki page.

---

## What a failing wiki looks like

Recognise these early — they are the difference between a compounding artifact and a folder of
summaries. Most are what a lint pass exists to catch.

| Symptom | Diagnosis | Fix |
|---|---|---|
| **Summary landfill** — `wiki/sources/` grows, everything else stale | ingests never rewired | rewire retroactively, source by source, oldest first |
| **Orphan farm** — pages with no inbound links | one-directional linking | add inbound links from the pages that should mention them, or merge them away |
| **Index drift** — pages missing from `index.md` | ingest closed without updating the index | rebuild the missing entries; index is retrieval |
| **Uncited synthesis** — confident prose, no `→ [[s-…]]` | claims written from the model, not the sources | trace each claim to a source page or mark it as unsourced |
| **Taxonomy sprawl** — 40 near-duplicate concept pages | nobody consolidated | propose merges to the human; then write the consolidated rule into the schema |
| **Frozen schema** — `CLAUDE.md` unchanged though the workflow changed three times | corrections stayed in chat | amend the schema; otherwise the next session repeats the mistakes |
| **Thin overview** — reads like it did at source #5 | nobody rewrote the synthesis | rewrite it from the current findings |
