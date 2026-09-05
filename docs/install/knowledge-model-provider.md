# Knowledge bank: the model provider and how to swap it

The knowledge bank calls its model provider from inside the MCP process, never
through the AI gateway, through one seam: the `ModelProvider` trait in
`extensions/mcp/knowledge-bank/src/model/mod.rs` (`embed`, `converse`, `rerank`,
`embed_model`, `embed_dim`, `supports_chat`). Two implementations ship, selected
by `model.provider` in `services/knowledge-bank/config.yaml`; nothing in
ingestion or retrieval names either.

## `local` (shipped default)

Free, deterministic, in-process, offline, no credential. Embeddings are
feature-hashed word n-grams: every token and every run of up to `ngram` adjacent
tokens is hashed, the hash picks a slot and a sign, the slot accumulates
`ln(1 + tf)`, and the vector is L2-normalised. Reranking is BM25 over the
candidate set with document frequencies taken from the candidates themselves,
normalised to `[0, 1]`, ties in fusion order. There is no chat model:
`supports_chat` is `false`, and the pipeline reads that once and takes the same
deterministic fallbacks it takes when a remote chat model fails — script chunking
for transcripts, no enrichment prefix, frontmatter-or-mtime dating — without a
warning per document.

What it is good for: real tools over real pgvector everywhere, CI, demos on the
shipped fixtures, and any corpus where lexical overlap between question and
answer is the norm (specifications, runbooks, tickets). What it is not: a
semantic embedding. A paraphrase that shares no words with its answer is found by
the BM25 arm only if it shares stems, and by the vector arm not at all.

```yaml
model:
  provider: local
  local:
    embed_model: local.hashed-ngram-v1   # cache key; bump it if the hashing changes
    embed_dim: 1024                      # must match the schema's vector(1024)
    ngram: 2                             # 1 = bag of words
```

## `bedrock`

Titan v2 embeddings, a Converse-API chat model for chunking, enrichment and
dating, and Cohere rerank, all in `eu-central-1` (the only EU region carrying a
rerank model; a non-EU region is refused at load when this provider is active).
Needs the `bedrock_api_key` secret — a long-term API key sent as a bearer, not
sig-v4 — and boot fails naming it when the secret is absent.

## Swapping

1. Set `model.provider: bedrock` (or back to `local`). The other provider's
   block may stay in the file; only the active one is validated for its
   provider-specific rules.
2. `systemprompt admin config secret set bedrock_api_key <key>` (Bedrock only).
3. Restart. The boot log's `knowledge-bank backend ready` line shows
   `provider`, `embed_model` and `chat`.
4. **Rebuild the index.** Embeddings are cached per `(embed_model, text_hash)`
   in `knowledge_embedding_cache`, so nothing is reused across a swap, and
   vectors from two models must never share an index. Each user runs
   `sync_knowledge {rebuild: true}`; an administrator can run it for the files
   categories.

Both providers produce 1024-wide vectors, which the `vector(1024)` column
requires; a provider configured to any other width is refused at load.

## Adding a third provider

Implement the trait in a sibling of `src/model/local/` and `src/model/bedrock/`,
add the variant to `ProviderKind` (`src/config/model.rs`) with its config block,
and add the match arm in `build_provider`. Retries, concurrency caps and any
residency guard live with the provider. If it has no chat model, return `false`
from `supports_chat` and the pipeline degrades exactly as it does for `local`.

## Reranking and index-time LLM tasks

`retrieval.rerank_enabled: false` serves the fusion order with no rerank call;
`dates.extract_via_llm: false` and `chunking.enrich: false` drop the two
index-time LLM tasks for a provider that has a chat model. For `local` those
three settings change nothing, since it has no chat model to call.
