# dynbrain

The Brain's native store: a systemprompt extension that keeps documentation in the instance itself, searchable later
with the same retrieval process dynbrain runs today. Internally everything is `dynbrain`; the core's own `brain_*`
tables, `/api/v1/brain` and `brain` toolset are the core's organisational memory and are a different thing.

## What exists

The schema, the repository, the `Brain` trait with its native implementation, the HTTP API (`/api/dynbrain/v1`), the
files of articles, the indexing and the search. The `brain` MCP server comes in a later phase.

- `DynbrainExtension` (`dynbrain`, "Brain"), depends on `users` and `web`; registers the schema and its migrations.
- `BrainRepository` (`repository/`): spaces, articles, variants, revisions, grants and the access closure.
- `access`: levels, subjects, grants and the inheritance rule as a pure function (`closure`, `subtree`).
- `properties`: the typed property schema of a space and the check of a variant's values.
- `language`: language codes and the text-search configuration of each (`fts_config`, the map dynbrain uses).
- `content`: the content hash of a variant.
- `assets`: the files of articles: link signing, `asset://` references, the type read from the bytes.
- `ingest`: the indexing (see Indexing).

The tools of the Brain are served by the MCP server `extensions/mcp/brain` (see its `README.md`). The admin's Brain section
(`extensions/web/admin/brain`) reads the Brain in the same process through `console` and the retriever's `explain`.

## Data model

| Table | What it holds |
| --- | --- |
| `dynbrain_space` | slug, name, default language, languages, property schema (JSON), creator |
| `dynbrain_article` | the tree: space, parent (same space), position, slug, icon, retrieval mode, `inherit_access`, trash stamp |
| `dynbrain_article_variant` | one per article and language: title, description, Markdown body, front matter, property values, content hash, revision |
| `dynbrain_article_revision` | every write of a variant, append-only (a trigger refuses updates) |
| `dynbrain_grant` | target (a space, or one of its articles), subject (user, role or group), level (none, read, write, manage) |
| `dynbrain_access` | the access closure: article × subject × level after inheritance |
| `dynbrain_chunk` | retrieval chunks: `heading_path`, content, synopsis, `fts_config`, generated `search_vector`, `embedding vector(1536)` and its fingerprint |
| `dynbrain_rag_rule` | eligibility rules, denied by default |
| `dynbrain_index_job` | the indexing queue, idempotent by content hash |
| `dynbrain_query_cache` | query embeddings |

The schema starts with `CREATE EXTENSION IF NOT EXISTS vector`: pgvector is required. Where the extension is not
trusted, the operator creates it before the install (the chart's CNPG `Database`).

There are no versions (branches or tags) of a space and no Git synchronisation: a native space is edited here.

## Properties

A space declares `[{name, string?, type, selection?, tags?}]`, where `type` is one of `boolean`, `integer`, `float`,
`text`, `char`, `date` (`YYYY-MM-DD`), `datetime` (RFC 3339), `selection`, `tags`, `user` (a user id) or `link` (an
absolute http(s) URL), and `selection` / `tags` list `[value, label]` pairs. A variant write is refused when a value
does not fit its type or names an undeclared property; `null` unsets a value. The front matter is free and kept as
written.

## Permissions

- Subjects are Governance's own: a user id, a role (`users.roles`) or a group (`groups`, resolved through the
  `user_groups` view, so only live memberships count).
- A grant is written on a space or on one of its articles, at `none`, `read`, `write` or `manage`.
- An article starts from what its parent resolves to (its space, for a top-level article), unless its
  `inherit_access` is off, in which case it starts empty. Its own grants are added on top; an own grant for a subject
  it inherited replaces that level, lower or higher; an own `none` takes the subject away for the article and below,
  until a descendant grants it again. `none` never appears in `dynbrain_access`.
- Turning `inherit_access` off (`set_inherit(article, false, copy_inherited, ..)`) either just cuts, or, with
  `copy_inherited`, first writes what the article inherited as its own grants (an existing own grant for the same
  subject is kept), as the access dialog's checkbox does.
- The space's creator and the `admin` role manage every article, whatever the grants say. Nothing else is granted by
  default.
- `dynbrain_access` holds the result and is the only permission source retrieval reads. It is rewritten for the
  affected subtree in the same transaction as the grant change, the new article or the move that alters it, under a
  lock on the space row, never in the background.
- `BrainRepository::find_level` answers the highest level a user reaches an article at, through a row for them, one of
  their roles or one of their live groups, and only for an active account.

## The Brain trait and the REST API

`brain::Brain` is the contract of the operations. It takes the person it runs as and reads no token. `NativeBrain`
implements it over this store; `search` and `answer` run the hybrid retrieval described under Search. The files of an
article are in the trait too (see [Files](#files)). Input and output types use the field names of the `docs_*` tools,
and what Governance adds (`slug`, `inherit_access`, `level`, `article_count`, `asset_urls`) is a new field.

`/api/dynbrain/v1` is mounted as a public router: it authenticates itself. The bearer is the DynWork desktop JWT
(audience `bridge` or `api`) or the JWT of a live phone; any other credential is `401` before a path, a query or a body
is read. Errors are `{"error": {"key", "message"}}` with the keys `unauthorized` (401), `not_found` (404), `forbidden`
(403), `invalid` and `bad_request` (400), `too_large` (413), `unavailable` (503) and `internal` (500).

A path's id overwrites the same field of the body. `limit` and `offset` page the lists as the `docs_*` tools do.

| Route | What it does | Example |
|---|---|---|
| `GET /spaces?query=&lang=&limit=&offset=` | The spaces the person reaches, with their `level` and `article_count`. | `{"spaces":[{"space":1,"slug":"handbook","name":"Handbook","lang":"es","langs":["es","en"],"article_count":2,"level":"manage"}],"total":1,"shown":1,"offset":0}` |
| `GET /spaces/{id}/outline?node=&depth=2&max_nodes=200&offset=&lang=` | The tree from the space or from `node`, `depth` levels (`-1` is all), at most `max_nodes`. | `{"anchor":{"id":1,"name":"Handbook"},"nodes":[{"id":7,"title":"Guide","lang":"es","depth":1,"parent_id":null,"has_children":true,"child_count":1}],"total":2,"shown":1,"truncated":true,"has_more":true,"offset":0,"hint":"2 nodes match and 1 are shown: pass `node` with one of them to narrow, or `offset` to page"}` |
| `GET /spaces/{id}/articles?lang=&limit=&offset=` | A flat list with each article's `path`. | `{"documents":[{"article_id":7,"path":"handbook/guide","title":"Guide","lang":"es","parent_id":null}],"total":1,"shown":1,"offset":0}` |
| `GET /spaces/{id}/export?format=full\|index&lang=` | The space as Markdown, in tree order. | `{"space":1,"format":"index","mimetype":"text/markdown","markdown":"- Guide (`handbook/guide`)\n","size_bytes":29,"article_count":1}` |
| `GET /spaces/{id}/schema` | The typed properties the space declares. | `{"space":1,"fields":[{"name":"status","type":"selection","selection":[["draft","Draft"]]}]}` |
| `GET /spaces/{id}/grants` | The grants written on the space; needs `manage`. | `{"grants":[{"kind":"group","id":"team","level":"read"}],"access":[],"inherit_access":null}` |
| `PUT /spaces/{id}/grants` | Replaces them with `{"grants":[{"kind","id","level"}]}`; `level` is `none`, `read`, `write` or `manage`. | `{"grants":[{"kind":"user","id":"bob","level":"none"}]}` |
| `GET /articles/{id}?lang=` | The article in that language (or the default one), its `children` and `can_write`. | `{"found":true,"article_id":7,"space":1,"path":"handbook/guide","title":"Guide","description":"","lang":"es","variant":{"lang":"es","is_default":true},"markdown":"...","front_matter":{},"metadata":{},"revision":1,"inherit_access":true,"can_write":true,"children":[]}` |
| `GET /articles/{id}/metadata?lang=` | Authorship, languages, `can_write` and the typed values. | `{"documents":[{"id":7,"title":"Guide","path":"handbook/guide","lang":"es","space":1,"is_synced":false,"can_write":true,"child_count":0,"variant_langs":["es"],"revision":1}]}` |
| `POST /articles` | Creates an article (`201`) under `parent_id`, or at the top of `space`: `{"parent_id","space","title","markdown","position":"first\|last","front_matter","metadata","lang"}`. | `{"article_id":8,"space":1,"parent_id":7,"title":"Install","path":"handbook/guide/install","variant":{"lang":"es","is_default":true}}` |
| `POST /articles/tree` | Creates a nested set (`201`, at most 200 nodes): `{"space","parent_id","nodes":[{"title","markdown","children":[]}],"lang"}`. | `{"parent_id":null,"nodes":[{"title":"A","article_id":9,"path":"handbook/a","children":[]}],"created":1}` |
| `PATCH /articles/{id}` | Edits one language: `title`, `markdown`, `front_matter` (`description` sets the description), `metadata` (`null` clears a key), `lang`. | `{"article_id":7,"title":"Guide","updated":true,"revision":2,"variant":{"lang":"es","is_default":true}}` |
| `POST /articles/{id}/move` | `{"new_parent_id","position":"first\|last"}`; no `new_parent_id` moves it to the top of its space. | `{"article_id":7,"new_parent_id":3,"moved":true}` |
| `POST /articles/{id}/trash` | Trashes the article and its subtree. | `{"article_id":7,"trashed":true}` |
| `GET /articles/{id}/grants` | Its own `grants` (with `none`), the resolved `access` (never `none`) and `inherit_access`; needs `manage`. | `{"grants":[],"access":[{"kind":"user","id":"alice","level":"manage"}],"inherit_access":true}` |
| `PUT /articles/{id}/grants` | Replaces its own grants, as the space route does. | `{"grants":[{"kind":"role","id":"editor","level":"write"}]}` |
| `PUT /articles/{id}/inherit` | `{"inherit","copy_inherited"}`: cut from its parent, optionally keeping what it inherited as its own grants. | `{"article_id":7,"inherit_access":false}` |
| `POST /search` | `{query, limit?, lang?, space?, where?}`: the hybrid search (see Search). | `{"query":"backups","mode":"hybrid","results":[{"article_id":4,"space":1,"path":"handbook/…/backups","title":"Backups","description":"","lang":"en","snippet":"…","score":0.93}],"total":1,"confidence":"high","top_score":0.93}` |
| `POST /answer` | `{question, lang?, space?, max_sources?}`: the assembled context, no model called. | `{"context":"[1] Backups (handbook/backups, lang=en)\n…","citations":[{"path":"handbook/backups","title":"Backups","lang":"en","snippet":"…"}],"mode":"context","confidence":"unrated","top_score":null,"confidence_note":"…"}` |

An article is also found by `path` (`<space slug>/<article slug>/...`) through the trait, which the MCP server uses.

Who may do what: reading needs `read` on the article (or on its space); `create`, `update`, `move` and `trash` need
`write` on the article or its parent, and a top-level article needs `write` on the space; grants and the inheritance
switch need `manage`. A space's level is its owner and the `admin` role (always `manage`), or the space-level grants
for the person, their roles or their groups; an empty space is listed for them and no one else.

## Files

An article carries files: images, PDFs, spreadsheets. Their bytes never travel through a tool call or through `get`,
in either direction.

- **Where the bytes live.** In core's file storage (`ctx.file_storage()`), under `dynbrain/uploads/<upload id>`. In
  production that is S3, the same bucket as chat uploads, the Drive and generated images; there is no bucket, client,
  credential or disk path of the Brain's own, and nothing to configure. `dynbrain_asset` keeps the storage reference,
  the name, the size, the SHA-256 and the type; `dynbrain_upload` keeps the upload slots.
- **Uploading, in two steps.** `request_upload` (`POST articles/{id}/assets/upload-url`, needs `write`) answers a
  one-time `upload_url`, bound to the person, valid 15 minutes, with `max_bytes` (25 MB by default). The file is POSTed
  to it as the raw body, with no bearer token (`curl --data-binary @diagram.png "$upload_url"`); the answer has the
  `upload_id`. `asset_upload` (`POST articles/{id}/assets` `{"upload_id","filename","lang"}`, needs `write`) adopts it
  as a file of the article. An upload id works once, and only for the person it was issued to.
- **Referencing a file.** Markdown names a file `asset://<id>`, as an image (`![Backup diagram](asset://12)`) or as a
  link (`[Template](asset://13)`). The reference never expires, and the stored text never holds a URL. When the
  indexer reads a body, each reference becomes its alternative text (`assets::markdown::index_text`), so no URL reaches
  a chunk or an embedding.
- **Downloading.** `get` and `export` add `asset_urls`, `{"asset://12": "<signed URL>"}` for each file the text names
  and the person may read. `asset_link` (`GET assets/{id}/link`) gives one `download_url`. A signed URL lasts 10
  minutes, carries nothing about the person, and is checked again on use: the file and its article must still exist
  and be out of the trash. `GET assets/{id}` serves the file to its reader with their JWT, for `DynWork` to draw images.
- **Signing.** An HMAC-SHA256 (core's `hmac_sha256`) under a key derived from the instance secret `dynbrain_links`
  (32 random bytes, held by the API and granted to the `brain` server, so a link either signs opens the API's route), over
  the purpose (`upload` or `download`), the target and the expiry. Core's own signed upload and download links open
  only core's document store and check access to a core document, so they cannot serve a Brain file; and an
  extension's token authority can only verify. Without the secret the file operations answer `unavailable`, in the API and
  in the `brain` server alike.
- **Serving.** The type comes from the bytes, never from the client; the name only narrows it within its family (a ZIP
  named `.xlsx` is a spreadsheet). Only PNG, JPEG, GIF and WebP are served `inline`. Everything else, SVG and HTML
  included, is an `attachment`. Every file goes out with `X-Content-Type-Options: nosniff` and
  `Content-Security-Policy: sandbox`. Issuing a link and every signed download are logged.
- **Trash.** Files follow their article: a trashed article's files are not listed, linked or served.
  `NativeBrain::purge_assets` deletes an article's file rows and their bytes, for when an article is purged.

| Route | What it does | Example |
|---|---|---|
| `POST /articles/{id}/assets/upload-url` | A one-time upload URL (`201`); needs `write`. | `{"upload_id":"9f…","upload_url":"https://…/api/dynbrain/v1/uploads/9f…?exp=…&sig=…","method":"POST","max_bytes":26214400,"expires_at":"…","expires_minutes":15,"article_id":7,"filename":null,"instructions":"…"}` |
| `POST /uploads/{upload_id}?exp=&sig=` | The file as the raw body, no bearer (`201`). A form is `400`, an empty body `400`, over the limit `413`, a bad, used or expired link `404`. | `{"upload_id":"9f…","size_bytes":5120,"mimetype":"image/png","sha256":"…"}` |
| `POST /articles/{id}/assets` | Adopts an upload as a file of the article (`201`); needs `write`. | `{"attached":true,"article_id":7,"attachment_id":12,"name":"diagram.png","mimetype":"image/png","size_bytes":5120,"lang":null,"ref":"asset://12","markdown":"![diagram.png](asset://12)"}` |
| `GET /articles/{id}/assets?lang=&offset=&limit=` | The article's files; with `lang`, that language's and the ones for every language. | `{"article_id":7,"files":[{"attachment_id":12,"name":"diagram.png","mimetype":"image/png","size_bytes":5120,"lang":null,"ref":"asset://12","markdown":"![diagram.png](asset://12)"}],"total":1,"shown":1,"offset":0}` |
| `GET /assets/{id}/link?article_id=` | A signed download URL; `not_found` when the file does not exist or the person cannot read it. | `{"found":true,"attachment_id":12,"article_id":7,"name":"diagram.png","mimetype":"image/png","size_bytes":5120,"download_url":"https://…/api/dynbrain/v1/assets/12/raw?exp=…&sig=…","expires_at":"…","expires_minutes":10}` |
| `GET /assets/{id}` | The file, to its reader (JWT). | the bytes |
| `GET /assets/{id}/raw?exp=&sig=` | The file, to the holder of a signed URL, no bearer. | the bytes |

## Indexing

Writing a variant queues it (`dynbrain_index_job`, one row per article, language and content hash); nothing is indexed
inside the request. The scheduler job `dynbrain_index` (every two minutes, under a lease so one replica works at a
time) does the rest, with dynbrain's numbers, which are the specification:

1. **Eligibility, denied by default.** An article's own mode (`always` or `never`) decides first; with `default`, the
   first active `dynbrain_rag_rule` that matches (all articles, a space, or an article and everything below it) decides;
   with none, the article is not indexed and its chunks are removed.
2. **Chunking.** The Markdown, with each `asset://<id>` reference replaced by its alt text (no URL reaches a chunk, a
   synopsis or an embedding), is cut by a port of dynbrain's `chunk_document`: 300 words of new text, 50 of overlap,
   6,000 characters at most, the heading path each chunk sits under. The unit tests compare the chunks with those
   Python produces from three dynbrain documents and an edge-case text.
3. **Synopsis.** A chunk with no heading gets one sentence of at most 30 words from `claude-haiku-5-5`, eight at a
   time, audited as the job's actor; a chunk whose text did not change keeps its synopsis.
4. **Embedding.** The text embedded is the prefix (title, description, heading path or synopsis; none for code) plus the
   chunk; the stored chunk stays clean. `text-embedding-3-large` at 1536 dimensions, in batches of 50 that are split in
   half when one is refused, so one bad text costs a few requests and a refused key costs three. Each vector carries
   its fingerprint (`provider:model:dims`); the cost of every request is recorded in `dynbrain_embedding_usage`.

Trashing an article keeps its chunks and holds back its indexing. `requeue_index` sends every settled job back, for
when the rules change. The `context_strategy` parameter of the scheduler entry is `auto` (default), `heading`, `llm`,
`heading_llm`, `document` or `off`.

**Secrets.** An instance needs `openai` (the platform credential of the `openai` provider in `services/ai`) to embed,
and the `anthropic` key it already has for the synopses. Without the OpenAI key the endpoint is withheld, the chunks wait
unembedded and the search will use only its text arm; without the Anthropic key chunks are embedded with the context
they have. The job `dynbrain_upload_purge` (04:15 UTC) deletes uploads whose link expired a day ago and that no article
adopted, with their bytes.

## Search

`search` and `answer` reproduce dynbrain's `docs_search` and `docs_answer` (`synthesize: false`). The service is
`search::Retriever`; it returns the passages with a `ScoringTrace` (which scoring produced the scores, and why when
the reranker did not) so the admin's search laboratory can build on it without changing the REST or MCP shapes.

- **Two arms, access inside the SQL.** The vector arm (pgvector HNSW, `hnsw.ef_search` 200 set per transaction) and
  the text arm (`plainto_tsquery` in each chunk's own language configuration) return 100 candidates each. Both keep
  only chunks of articles the reader reaches in `dynbrain_access` (a row for them, a role or a group, `read` or more,
  active account) and out of the trash, in the query itself, so a reader who sees little still gets a full pool.
  `space`, `lang` (spaces written in that language) and `where` are applied there too.
- **Fingerprint.** If any stored vector was made by another embedding configuration, the vector arm is switched off
  and the search runs on text alone.
- **Fusion.** Reciprocal Rank Fusion with `k = 60`; ties keep the order first seen.
- **Reranker.** Cohere `rerank-v3.5`, 5 s limit, key = the instance secret `cohere`. Any failure keeps the fused
  order. A circuit breaker per provider and endpoint writes it off for 60 s after a failure on the wire, doubling up
  to 900 s; refusals (400, 401, 403, 422) do not open it.
- **Deduplication.** One passage per article; a passage whose word set overlaps a kept one by Jaccard ≥ 0.85 is
  dropped; within an article the preferred language (`lang`, or the language the query reads as) represents it.
- **Confidence.** `high` / `low` split the reranker's best score at 0.70 (`RetrievalSettings::weak_match_threshold`;
  0 switches it off), with dynbrain's note for `low`. Without reranker scores it is `unrated`, with its own note.
- **Query cache.** `dynbrain_query_cache`, keyed by `sha1(provider:model:normalised query)`; a miss embeds with a 6 s
  limit and one retry, and purges entries unused for 30 days. The query embedder is core's `AiService` with the
  indexer's model (`openai` `text-embedding-3-large`, 1536); core hands it to the routes per request.
- **Keyword fallback.** Without an embedder, or when the query cannot be embedded, the search matches title and body
  (`ILIKE`, access in SQL), `mode: "keyword"`, `confidence: "unrated"`.
- **`where`.** A map of declared properties (any space's); an undeclared key is refused naming the declared ones;
  `false` or `null` means "not set". Lists (`GET spaces/{id}/articles`) follow the same rule.
- **Paths.** A path never names an ancestor its reader cannot read: each run of them is one `…`
  (`handbook/…/holidays`). What a person reads in a space is one query (`list_article_levels`), also for lists, the
  outline and the export.

### The answer context

`answer` calls no model. Each source document gets 25,000 characters. A document that fits is given whole; otherwise
its own chunks most relevant to the question (ranked against the same query vector, no second embedding call), each
whole, back in reading order, with `[...]` where one was left out. Only the best chunk is ever cut, marked
`[...truncated...]`. Blocks are `[n] title (path, lang=xx)` (`lang=?` when unknown), numbered by source position and
separated by `---`; `citations[n-1]` is `{path, title, lang, snippet}`.

## Native mode and proxy mode

An instance answers from one place, set in `services/config/dynbrain.yaml` (read by core as the extension's config
block, `config_prefix: dynbrain`):

- `mode: native` (the default): this instance's own store, everything above.
- `mode: proxy`: every one of the eighteen operations is a call to a tool of another MCP server, the *source*, made as
  the person who asks. The REST routes and the `brain` MCP server go through the same `Brain::run`, so both answer what
  the source answers, as it came.

Never both. Merging results from Governance's own index and a source's index needs one ranking across two systems that
score differently, and that is not solved; a customer who needs several sources sends their documentation into
Governance (an ingestion integration, to be evaluated) instead of using the proxy (D13).

The mapping lives in each customer's repository, never in the product. A generic example:

```yaml
dynbrain:
  mode: proxy
  proxy:
    server: brain-source        # an external MCP server declared in services/mcp/
    file_urls:                  # the URL paths the source names its files with
      - /files/{id}
    operations:
      search:
        tool: docs_search
        args: { where: filter } # our argument name: the source's; the rest keep their name
      answer:
        tool: docs_answer
        fixed: { synthesize: false }   # constants added to every call
      get:
        tool: docs_get
      # one entry per operation; an operation with no entry answers `503 unavailable`
```

- **Identity.** Nothing in the Brain signs a token. The source is called with core's MCP client and the person's
  request context; when its declaration in `services/mcp/` has `external_auth` pointing at the instance's identity
  accessor (`/api/public/identity/<server>/token`), core asks it for a short-lived RS256 token of that person
  (`iss` the instance, `aud` the source's endpoint, `sub`, `email`, `name`, `groups`, five minutes) and sends it. The
  source must therefore trust the instance's JWKS (`/.well-known/jwks.json`) and issuer, accept the token on the
  endpoint the `aud` names (with the server's slug in it), and find its user by `email`. The process that calls must
  hold the secret `mcp_credential_broker_secret`; the `brain` server needs it granted in proxy mode.
- **Errors.** A 401 or 403 of the source, or a refused token, is `403 forbidden`; a protocol "not found" is `404`; other
  protocol errors `400 invalid`, each with the source's message; a source that cannot be reached is `503
  unavailable`.
- **Permissions** are the source's: the grant and inheritance routes answer `409 managed_at_source`.
- **Files.** `request_upload`, `asset_upload`, `asset_list` and `asset_link` go to the source as the person; the upload
  URL is the source's, and the client sends the bytes there. `get` leaves the Markdown as a native `get` does: every
  reference whose URL path matches `file_urls` (`{id}` for the id, host ignored), or whose relative path the source's
  `asset_list` gives an id for, becomes `asset://<source id>`, and `asset_urls` holds the source's `asset_link` for each
  one (at most 50). `GET assets/{id}` asks the source for that link as the person and answers `307` to it (`404` when
  the source refuses). No byte is kept here, and the upload and signed-download routes of this instance find nothing.
- **Startup.** An unknown mode or key, a proxy with no `server`, an empty `tool`, or a `server` that is not a declared,
  enabled MCP server leaves the Brain unmounted in the API (logged) and stops the `brain` server from starting.

