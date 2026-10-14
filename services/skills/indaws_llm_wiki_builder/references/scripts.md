# Scripts — qué hace cada uno

Todos son deterministas y se **ejecutan**, no se leen: cuestan 0 tokens de contexto. `-h` en
cualquiera da su interfaz exacta.

| Script | Purpose |
|---|---|
| `scripts/wiki_scaffold.py` | Layout + templates + draft schema. Idempotent (`--force` to overwrite). |
| `scripts/wiki_lint.py` | Mechanical health check. `--json`, `--strict` (exit 1 on errors). |
| `scripts/wiki_cite_check.py` | Code wikis: resolves every `path:line @sha` against `git show` and prints the cited line. `lint` only checks a SHA is *present*; this checks it is *true*. |
| `scripts/wiki_search.py` | BM25 over pages, accent-folded ES/EN. `--qmd` delegates to qmd. |
| `scripts/wiki_index.py` | Rung 3: builds the local vector store over one or more wikis. Chunks and diffs locally (stdlib, ms), delegates only the embeddings — to `dynserver` via `jobrun`, or to any endpoint. Incremental by content hash; **refuses to mix models** (exit 2) and **writes nothing if the endpoint dies mid-batch**. `--dry-run`, `--status`, `--local`, `--reembed`. |
| `scripts/wiki_embed_job.py` | The payload `wiki_index.py` sends to the embedding host. Stdlib only (no numpy, no venv). Not called by hand. |
| `scripts/wiki_vsearch.py` | Query the store: embeds the question (≈1,1 s incl. ssh), dot product, top-K with `file:line`. **Falls back to BM25 and says so** if the endpoint is down; `--no-fallback` to fail instead. Read the measured limits in [search-qmd.md](search-qmd.md) before trusting it — it is strong on paraphrase and on passages inside big documents, weak on inferential queries. |
| `scripts/wiki_dupes.py` | Consolidation, not retrieval: which files' content is largely present in another file (near-duplicates, superseded versions, PDF/HTML twins). Reuses the stored vectors, so it costs nothing. **Candidates, not verdicts.** |
| `scripts/pdf_probe.py` | PDF sources: inventory before reading — pages, text layer vs scan, TOC, and per page chars / large raster images / vector paths / tables, plus a reading plan and a text-vs-image cost estimate. |
| `scripts/pdf_extract.py` | PDF sources: `sections` (TOC → page ranges), `text` (`--pages`/`--section`, layout-preserving), `render` (specific pages → PNG, for the few you must *look* at), `ocr` (scans, via tesseract). Writes to `.cache/`. |
| `scripts/wiki_log.py` | Append a well-formed log entry; `--tail N` to read recent ones. |
| `scripts/wiki_coverage.py` | Code wikis: **medidor honesto de progreso** — de todos los símbolos que la capa A conoce en una ruta, cuántos están al menos NOMBRADOS en la prosa. Determinista, 0 tokens. Dos avisos que importan más que el número: nombrar no es explicar (es una cota superior), y 100% es el objetivo equivocado — documentar 793 símbolos del ORM produce vertedero. Sirve para ver *qué* huecos son decisiones, con `--missing`. |
| `scripts/wiki_migration.py` | Code wikis: generate `wiki/migrations/m-<from>-to-<to>.md` as a **view** over the delta pages' frontmatter, preserving a hand-written `<!-- CURATED -->` block. |
| `scripts/code_extract.py` | Code wikis: AST/XML/JS/CSV inventory of a source tree → SQLite + JSONL, pinned to the git SHA **and to `--odoo-version`** (several majors in one inventory; re-running replaces one). Includes the **security data** (ACL rows, record rules with domains). |
| `scripts/code_query.py` | Query that inventory (`model`, `inherits`, `overrides`, `field`, `security`, `drift`…) plus the cross-version pair **`diff`** and **`lifespan`**; prints `file:line @sha`. |

| `scripts/check_upstream.py` | Are the sources this skill derives from still what we copied? Hash of the raw bytes vs the pins in the script (gist de Karpathy + `second-brain`). `--update` prints the new pins. Exit 1 on drift **or** on unreachable — no red no es "sin cambios". Veredicto de la última comparación: `references/upstream-sources.md`. |
