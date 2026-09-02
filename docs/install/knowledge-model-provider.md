# Knowledge bank: swapping the model provider

The knowledge bank calls its model provider from inside the MCP process, never
through the AI gateway, through one seam: the `ModelProvider` trait in
`extensions/mcp/knowledge-bank/src/model/mod.rs` (`embed`, `converse`, `rerank`,
`embed_model`, `embed_dim`). Bedrock in eu-central-1 is the shipped
implementation; nothing in ingestion or retrieval names it.

## What a swap involves

1. **Implement the trait** in a sibling of `src/model/bedrock/` and add the
   variant to `ProviderKind` (`src/config/model.rs`) and the match in
   `build_provider`. Retries, concurrency caps and the residency guard live with
   the provider, so the new one carries its own.
2. **Configure it** in `services/knowledge-bank/config.yaml` under `model:` —
   `provider:` selects the implementation; per-provider settings sit beside it.
   An unknown `provider:` fails the boot with the accepted values.
3. **Rebuild the index if the embedding model changes.** Embeddings are cached per
   `(embed_model, text_hash)` in `knowledge_embedding_cache` and the `vector`
   column is sized by `embed_dim`, so a new embedding model or width means
   `sync_knowledge {rebuild: true}` by each user (or an operator run of every
   files category) after the schema's vector width is updated.
4. **Keep the secret in the profile store.** The credential is read by name at
   boot (`bedrock_api_key` today); a new provider reads its own secret the same
   way. Nothing is read from the YAML.

## Reranking and index-time LLM tasks

`retrieval.rerank_enabled: false` serves the fusion order with no rerank call;
`dates.extract_via_llm: false` and `chunking.enrich: false` drop the two
index-time LLM tasks (the `llm` chunk strategy falls back to the script splitter
when the model fails). A provider without a reranker or chat model is therefore
usable for embeddings alone.
