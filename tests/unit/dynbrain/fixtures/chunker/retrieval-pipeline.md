# Retrieval pipeline

`dynapps.knowledge.chunk._hybrid_search(query, space_ids, limit, dedup, rerank)` is the
entry point. It embeds the query (cached) and calls `_hybrid_search_with_vector`, which
runs two SQL arms and fuses them.

The candidate pool (`inner_limit`) is **100** (`CANDIDATE_POOL`): both arms and the
reranker work over 100 candidates before the final `limit` is applied. Widening it from
30 to 100 raised the reranker's ceiling from 96% to 100% at no extra billing.

## 1. Vector arm (semantic, cross-lingual)

```sql
SET LOCAL hnsw.ef_search = 200;             -- raised from pgvector's default of 40
SELECT id FROM dynapps_knowledge_chunk
WHERE embedding IS NOT NULL {space}
ORDER BY embedding <=> %(vec)s::vector, id  -- cosine distance, HNSW; id tie-break
LIMIT %(inner_limit)s
```

Uses the multilingual embedding space, so the query language is irrelevant — a Dutch
query retrieves Spanish content. This is the arm that makes "one query, all countries'
knowledge" work.

**`hnsw.ef_search` is set to 200** per query (`HNSW_EF_SEARCH`, tunable via
`dynapps_knowledge_rag.hnsw_ef_search`). HNSW is an approximate index; pgvector's
default of 40 silently cost ~7 points of document recall against an exact scan on this
corpus. The `id` tie-break makes the order deterministic when distances tie (identical
chunks, mirrored translations).

## 2. FTS arm (lexical, per document language)

```sql
SELECT id, ts_rank(search_vector,
           plainto_tsquery(COALESCE(fts_config,'simple')::regconfig, %(q)s)) AS r
FROM dynapps_knowledge_chunk
WHERE search_vector @@ plainto_tsquery(COALESCE(fts_config,'simple')::regconfig, %(q)s)
      {space}
ORDER BY r DESC
LIMIT %(inner_limit)s
```

`fts_config` is a per-row column (`spanish`, `dutch`, …), so each document is matched
with its own language stemming. Adds exact-term precision (codes, product names,
acronyms).

## 3. Reciprocal Rank Fusion

```python
scores[id] += 1.0 / (RRF_K + rank)   # RRF_K = 60, for both arms
```

RRF fuses by **rank**, not raw score, so the incompatible scales of cosine distance and
`ts_rank` never need normalising. A document ranked well by either arm rises to the top.

## 4. Reranking (optional, biggest precision gain)

If a reranker provider + key is configured, the fused candidates (the 100-candidate
pool) are re-scored by a cross-encoder that reads the query and each chunk's `content`
together. Providers are pluggable through the registry:

| Provider        | Default model                          |
| --------------- | -------------------------------------- |
| **ZeroEntropy** | `zerank-2` (gbrain's default reranker) |
| **Cohere**      | `rerank-v3.5`                          |
| **Voyage**      | `rerank-2.5`                           |

On no key / API error it is a graceful **fail-open** no-op that keeps the fused order
(the pipeline degrades but never breaks).

## 5. Deduplication

For list / MCP results (`dedup=True`, default) the hits are collapsed so each
**canonical document** appears at most `max_per_doc` (1) times, and near-identical
chunks (word-set Jaccard ≥ 0.85 — e.g. the es/en mirror of an untranslated page) are
dropped. The document identity is `(dynapps_repo_id, dynapps_source_path)`, so the same
file imported into several language spaces collapses to one result.

For RAG **context** building pass `dedup=False` to keep every matching chunk.

## 6. Answer-context assembly

Retrieval ranks _chunks_, but a single chunk is rarely enough to answer from — the
passage that answers often sits elsewhere in the same document. `_build_context` turns
the ranked documents back into readable text for an answering model:

- Every retrieved document gets an **equal quota** of `CONTEXT_QUOTA_CHARS` (**25 000**
  characters, `dynapps_knowledge_rag.context_quota_chars`).
- If the document fits its quota, the whole document goes in; otherwise its most
  relevant **complete** chunks are selected and put back in reading order, with a
  `[...]` mark where a chunk was skipped. **Text is never cut mid-passage** — a
  mutilated passage is worse than an absent one.
- The quota is **per document**, not a fixed total split across sources, so it does not
  shrink as the caller asks for more sources. This matters: the answering passage
  reaches the model reliably only above ~20 000 chars per document, and a fixed total
  would drop below that cliff as the source count grows.
- An optional hard ceiling on the whole assembled context is `context_budget_chars`
  (unset by default: the caller already bounds cost by choosing how many sources it
  wants).

Selecting within a document costs **no extra embedding call** — it reuses the query
vector from retrieval.

## Order of operations

```
embed query (cached)
  → vector arm  ┐
  → FTS arm     ┴→ RRF fusion → rerank → dedup → [:limit]
```

Rerank runs before dedup so the most relevant chunk per document is the one kept.
