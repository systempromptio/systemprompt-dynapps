# systemprompt-web-admin-brain

The Brain section of the admin: `/admin/brain/*`, for console admins only (B10). It shows the whole Brain of the instance
and the technical layer the desktop app's Brain view leaves out. It reads the Brain in the same process, through
`dynbrain::console::Console` (published by the extension when it composes the Brain), never through the REST API. The
tree, the document and the files go through the Brain's operations as the signed-in admin, so they read the same on a
native and on a proxy instance. Everything else reads the native index and, on a proxy instance, says the index lives at
the source and queries no table (B11). The content is never edited here (B12).

## Pages and where each number comes from

| Page | Shows | Source |
| --- | --- | --- |
| `/admin/brain` | mode, counters (articles, documents indexed, chunks, embeddings and synopses pending, errors), providers and models, the confidence threshold, searches with no result (7 days), embedding cost (30 days), spaces | `repo.count_index`, `retriever.settings()`, `dynbrain_search_log`, `dynbrain_embedding_usage`, `list_spaces` and `list` |
| `/admin/brain/spaces` | every space and article as a collapsible tree | `list_spaces` and `list`, as the admin |
| `/admin/brain/articles/{id}?lang=&tab=` | the tree beside one article in one language | `get` as the admin; the tabs below |
| tab Content | id, path, revision, description, Markdown (read only) | `get` |
| tab Chunks | per chunk: order, `heading_path`, words, `fts_config`, code, embedding state, fingerprint; opened: the contextual prefix (computed with the indexer's `prefix`), synopsis, text | `repo.list_chunk_infos` |
| tab Ingest | the five steps, the jobs with attempts and last error, Reindex and Retry | chunks and `dynbrain_index_job` |
| tab Files | name, type, size, language | `asset_list` as the admin |
| tab Access | own and effective grants; the form adds, changes or removes an own row | `list_grants`, `set_grants` as the admin (a proxy answers `managed at the source`) |
| tab Usage | times retrieved and cited in 30 days | `dynbrain_search_log` |
| tab History | the revisions of the variant | `repo.list_revisions` |
| `/admin/brain/search` | the search lab | `Retriever::explain` and `NativeBrain::context` |
| `/admin/brain/ingest` | the global queue, filter by state, Retry | `dynbrain_index_job` |
| `/admin/brain/usage` | documents most returned and cited, searches with no result, embedding cost; counts only | `dynbrain_search_log`, `dynbrain_embedding_usage` |

## The search lab

It runs `Retriever::explain`, which shares `stage` with the real search, so the order shown is the order the operations
return. Per candidate: rank and cosine of the vector arm, rank and `ts_rank` of the text arm, RRF, the reranker's score
when it ran, and what deduplication did (same document, Jaccard ≥ 0.85, beyond the first ten). A candidate below the
confidence threshold is tagged, not struck: the search does not drop it, it only rates the list. Under it, the context
`answer` would give, with its characters over the 25,000 limit, and the timings of each step.

"Run as" searches with another person's id and permissions and never uses their token. With it set, the lab also runs the
search as the admin and lists as struck rows the results that person cannot read ("No access for X (filtered in SQL)").
Every run writes a row in `dynbrain_admin_audit` (`search_lab`, or `run_as` with the person as subject). A lab run is
not counted in `dynbrain_search_log`.

## Writes

Three only: Reindex (queues the variant's current content again), Retry (a failed job back to pending) and the access
form. Each leaves a row in `dynbrain_admin_audit`; on a proxy instance the first two answer `409`.

## Layering

The crate sits in the `entry` tier of `scripts/lint-layers.sh` (it depends on `dynbrain`). The pages hold no query: the
new ones are in `extensions/dynbrain/src/repository/admin.rs`.

Tests: `tests/integration/admin-brain`.
