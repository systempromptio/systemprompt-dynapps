# Retrieval: index first, then search

A wiki's retrieval strategy should match its size. Building search infrastructure for 30 pages is
wasted work; relying on `index.md` alone at 800 pages loses information. Climb the ladder only
when the rung you're on stops working.

## The ladder

| Rung | Use when | Cost |
|---|---|---|
| **1. `index.md`** | Up to roughly 100 sources / a few hundred pages | One file read. Free. |
| **2. `scripts/wiki_search.py`** | The index doesn't surface it, or you need to find a phrase | Pure-Python BM25, no dependencies, no models. |
| **3a. Local vector store** (`wiki_index.py` + `wiki_vsearch.py`) | Passages *inside* big converted documents; several wikis queried at once | An Ollama endpoint (delegable to another machine) + ~90 chunks/min of its CPU. numpy locally. |
| **3b. `qmd`** | Same job, zero code, self-contained | Node ≥22 + ~2GB of local GGUF models, embedded on this machine. |

**Why the index works better than it sounds.** Its one-line summaries were written by something
that had read the page and understood it — that is a stronger signal than chunk-level embedding
similarity, and it comes with the structure (category, source count, dates) attached. Read the
index, pick 3-8 pages, read them. Most queries end here.

**When rung 1 breaks:** you find yourself reading pages that turn out irrelevant, or the user
mentions something you can't locate. Both are symptoms of an index whose summaries have gone
stale — so before installing search, ask whether the real fix is a lint pass that rewrites the
index entries.

## Rung 2 — `wiki_search.py`

```bash
python3 scripts/wiki_search.py <wiki-root> "lock contention workers"
python3 scripts/wiki_search.py <wiki-root> "rappel" --type concept -n 10
python3 scripts/wiki_search.py <wiki-root> "thesis" --json
```

BM25 over page bodies with frontmatter-aware boosts (title and tags weigh more), stemming-free so
it behaves the same in Spanish and English, with matching-line snippets. No index to maintain: it
walks the wiki each run, which is fine up to a few thousand files.

It finds **words you can name**. It will not find "the paper that argued the opposite of X" — and,
measured, neither does rung 3 (see below). For that, you still read.

## Rung 3a — the local vector store

Three scripts, no service, no database:

```bash
# 1. index (chunking and diffing here, embeddings delegated; only what changed travels)
python3 scripts/wiki_index.py /opt/PROYECTS/*/wiki
python3 scripts/wiki_index.py <root>... --dry-run        # report the work, embed nothing
python3 scripts/wiki_index.py --status

# 2. query
python3 scripts/wiki_vsearch.py "why did throughput stall" -n 8
python3 scripts/wiki_vsearch.py "..." --wiki ROTGER --json

# 3. consolidation, not retrieval — near-duplicates and superseded files
python3 scripts/wiki_dupes.py --wiki ROTGER
```

Store: `~/.claude/wiki-index/{meta.json, chunks.jsonl, vectors.f32}`. Config by env:
`WIKI_EMBED_HOST` (default `dynserver`), `WIKI_EMBED_MODEL` (default `nomic-embed-text`),
`WIKI_EMBED_URL` (talk HTTP directly instead of over ssh), `WIKI_INDEX_STORE`.

### The inversion this makes possible

The ladder above says "build search when `index.md` stops working" — i.e. late, on a wiki already
written. That was right when indexing meant installing infrastructure. When embedding costs an hour or two of
another machine's CPU **that you are not sitting in front of**, you can index the raw material on
day 0, before writing a single page, and then ingest *by question* instead of *by file*:

> `wiki_vsearch.py "<topic>"` → the eight passages on that topic across fifteen documents, with
> provenance → **one page that cites all of them.**

Reading file by file forces writing page by page and merging later. Asking the corpus gives you
the cross-source view from the start, which is what a wiki page is supposed to be.

### No database needed

10k chunks × 768 dims = **31 MB**. A dot product over that is milliseconds. pgvector, HNSW and a
Postgres exist for products serving many users over millions of rows; a personal corpus is a
`.npy` file. Don't import the product architecture into the tooling.

### What it does and does not do — measured 2026-08-21

Corpus: 11 wikis, 1.474 files, 10.109 chunks, 2,33M words. Embedded by `nomic-embed-text`
(768 dims, with the `search_document:`/`search_query:` prefixes) on a 12-thread CPU host with no GPU,
while that host was also running 15 containers.

| | Measured |
|---|---|
| Embedding throughput | **78–143 chunks/min**, and it *degrades with job size*: 143 for a 200-chunk job, 78 for a 5.000-chunk one. Whole corpus: **118 min wall clock, 91/min average**. Plan with 90, not 143 |
| Query, end to end | **1,1 s** including the ssh round trip |
| Degradation to BM25 when the endpoint is down | **0,2 s**, announced on stderr, `"mode"` in the JSON |
| Store size | 31 MB |

**Where it wins.** Six queries that paraphrase what the target page says: **6/6 at rank #1**,
against 5/6 for BM25 with worse ranks (#3, #4, one miss). And it retrieves *passages inside huge
documents*: a 479-page PDF converted to one 1,2 MB markdown answered with `AF Rotger.md:12115`,
a line no page-level search would ever have surfaced. That is the capability rung 2 does not have.

**Where it fails.** Four queries with **no lexical overlap** with the target page — the exact case
semantic search is supposed to own ("the client can't issue invoices during the switchover" →
the page about starting in one block with the old system switched off): **0/4**. BM25 got 1/4.
Neither works; the vector arm did not deliver the thing that motivated it.

Two hypotheses tested and **rejected** as the cause:

- *Cross-corpus contamination* (11 wikis in one store, the big ones crowding the top-K):
  re-running scoped with `--wiki` turned one MISS into a #4. Not it.
- *Duplicate mass* (`wiki_dupes.py` found 4 transcripts indexed twice, once as `raw/X.md` and
  once as its own conversion `.cache/text/X.md.txt`): removing 601 duplicate chunks changed the
  hard set **not at all** — still 2/6. Worth doing anyway; not the cause.

What is left, and is **untested**: the model. Every score in the run sits between 0,68 and 0,75
whether the hit is right or wrong — almost no dynamic range, which is what a model with weak
discrimination on this language looks like. `bge-m3` (genuinely multilingual, 1024 dims) and a
cross-encoder reranker are the two things to try before concluding anything general about
semantic search in Spanish. **Neither has been measured. Do not repeat this section's numbers as
if they were about "vector search"; they are about one model.**

**So: use rung 3a for what it demonstrably does** — finding passages in large documents, and
paraphrase queries across several corpora — and keep reading for the rest. Run the five-query
spot-check on your own corpus before trusting it; the script makes that cheap and the numbers
above are exactly why.

### Three guards that are not optional

- **Model fingerprint.** The store records which model built it and **refuses to search or extend
  it with another** (exit 2). Vectors from two models are perfectly comparable and mean nothing:
  a model swap does not break search, it makes it rank by noise, which is worse.
- **All-or-nothing indexing.** If the endpoint is unreachable mid-batch, the store is left exactly
  as it was. The fallback for indexing is "don't index", never "index badly" — that is how an
  index quietly ends up with holes that nothing reports. (Verified by an accidental SIGTERM at
  chunk 48/136: nothing was written.)
- **Announced degradation.** A query that could not embed falls back to BM25 **and says so**. A
  silent degradation is a lie with a different name.

### Gotchas paid for once

- **Asymmetric models need their prefixes.** `nomic-embed-text` scores near-randomly without
  `search_document:` / `search_query:`. A vendor layer that "just posts the text" is how the model
  gets written off as useless when the configuration was the problem. `PREFIXES` in
  `wiki_index.py` is per model; anything unlisted is treated as symmetric.
- **Don't index a conversion of something already indexable.** `X.md.txt` next to `raw/X.md` is
  the same document twice. Excluded by pattern now, but the class is broader than the pattern.
- **`.cache/` is indexed on purpose** — that is where `doc2md`'s output of PDF/HTML/pptx lives, and
  it is the material worth indexing most. It got included by accident first; now it is a decision.
- **`SUFFIXES` is `.md`/`.txt` only.** PDF, HTML, docx and pptx must go through `doc2md` first, and
  **nothing checks that they did**. A wiki whose `raw/` is 60% PDF has 60% of its corpus outside
  the index and no warning about it.
- **A shrinking corpus still rewrites the store.** Files removed or newly excluded need no
  embedding call, which is not the same as needing no work.

## Rung 3b — `qmd`

[qmd](https://github.com/tobi/qmd) is an on-device search engine for markdown: BM25 + vector +
LLM re-ranking, no cloud. It has a CLI (shell out to it) and an MCP server (native tools).
Verified against its README (2026-08-03):

```bash
npm install -g @tobilu/qmd          # or: bun install -g @tobilu/qmd

qmd collection add <wiki-root> --name mywiki
qmd context add qmd://mywiki "<one line on what this wiki is about>"
qmd embed                            # downloads ~2GB of GGUF models on first run
```

Then:

```bash
qmd search "lock contention" -c mywiki      # BM25 only, fast
qmd vsearch "why did throughput stall"      # vector / semantic
qmd query "when does a wiki beat RAG" -c mywiki   # hybrid + rerank, best quality
qmd query "…" --format json -n 10           # structured output for agents
qmd get "wiki/concepts/x.md" --full         # retrieve a document
qmd update                                  # re-index after ingests
qmd status                                   # index health
qmd doctor                                   # diagnose the install
```

Notes that matter in practice:

- **`qmd update` after every ingest session**, or search silently answers from a stale index. Put
  it in the wiki's `CLAUDE.md` as part of the ingest checklist. You can attach a pre-update hook
  per collection: `qmd collection update-cmd mywiki 'git pull --ff-only'`.
- **`qmd context add`** is not decoration: the context string is returned alongside matches and is
  what lets an agent judge relevance. Write a real sentence.
- **MCP integration** for Claude Code: `claude plugin marketplace add tobi/qmd` then
  `claude plugin install qmd@qmd` (or configure the `qmd mcp` stdio server manually in settings).
- **Requirements**: Node ≥22 (Bun also supported), macOS needs Homebrew SQLite for extensions,
  models cached in `~/.cache/qmd/models/`.

**3a or 3b?** qmd is less work and self-contained, and it has a reranker, which is one of the two
things rung 3a is missing. It embeds on *this* machine, so it competes with whatever else the
laptop is doing, and it does not chunk your `doc2md` output unless you put it under the wiki root.
Take 3a when the embedding must happen elsewhere or the corpus is several wikis; take 3b when you
want retrieval today and don't care where the CPU goes.

## Non-English wikis

Treat "semantic search will handle Spanish" as an assumption to verify, not a given. Two in-house
measurements, both on Spanish content, both partial:

- **e5-small, cross-lingual retrieval**: ~10% against ~95% for keyword search on the same corpus.
- **nomic-embed-text with prefixes, monolingual ES** (2026-08-21, above): 6/6 on paraphrase
  queries, 0/4 on inferential ones.

Neither is a measurement of qmd, whose default model (`embeddinggemma-300M`) is also
English-optimised — for a Spanish wiki that affects the *vector* leg only, BM25 keeps working in
any language. Switching qmd's model requires re-embedding, since vectors aren't cross-compatible:

```bash
export QMD_EMBED_MODEL="hf:Qwen/Qwen3-Embedding-0.6B-GGUF/Qwen3-Embedding-0.6B-Q8_0.gguf"
qmd embed -f
```

Whatever you choose, spot-check it: run five queries you know the answers to and see whether the
right pages come back — **including at least two whose wording does not resemble the target
page**, which is where both models measured here fell over. Assumed-working retrieval is how a
wiki quietly stops being used.

## Don't build a fourth rung

If rung 3 plus a good index doesn't cover it, the problem is almost never retrieval technology —
it's that the wiki has grown without consolidation. 900 sprawling pages retrieve worse than 300
well-merged ones. Fix it with a lint pass, not with a bigger embedding pipeline.

`wiki_dupes.py` is the tool for finding *what* to consolidate: it reports files whose content is
largely present in another file (cosine over chunks that are already embedded, so it costs nothing
extra), which is the class a word-overlap check misses — v5 versus v7 of the same deck, a PDF and
its HTML twin, a source whose every point already went into a page. It reports **candidates, not
verdicts**: two documents saying the same thing may be a duplicate to delete, a translation to
keep, or a decision that got reaffirmed, and only reading them tells you which.
