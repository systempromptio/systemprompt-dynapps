# dynbrain

The Brain's native store: a systemprompt extension that keeps documentation in the instance itself, searchable later
with the same retrieval process dynbrain runs today. Internally everything is `dynbrain`; the core's own `brain_*`
tables, `/api/v1/brain` and `brain` toolset are the core's organisational memory and are a different thing.

## What exists

The schema and the repository. The HTTP API (`/api/dynbrain/v1`), the ingestion, the search and the `brain` MCP server
come in later phases.

- `DynbrainExtension` (`dynbrain`, "Brain"), depends on `users` and `web`; registers the schema and its migrations.
- `BrainRepository` (`repository/`): spaces, articles, variants, revisions, grants and the access closure.
- `access`: levels, subjects, grants and the inheritance rule as a pure function (`closure`, `subtree`).
- `properties`: the typed property schema of a space and the check of a variant's values.
- `language`: language codes and the text-search configuration of each (`fts_config`, the map dynbrain uses).
- `content`: the content hash of a variant.

## Data model

| Table | What it holds |
| --- | --- |
| `dynbrain_space` | slug, name, default language, languages, property schema (JSON), creator |
| `dynbrain_article` | the tree: space, parent (same space), position, slug, icon, retrieval mode, trash stamp |
| `dynbrain_article_variant` | one per article and language: title, description, Markdown body, front matter, property values, content hash, revision |
| `dynbrain_article_revision` | every write of a variant, append-only (a trigger refuses updates) |
| `dynbrain_grant` | subject (user, role or group), level (read, write, manage), `inherit` |
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
- An article with grants of its own is reached by exactly those grants. An article without grants takes its parent's
  grants that inherit. A grant with `inherit = false` reaches its own article only.
- An article that no grant reaches is seen only by the space's creator and the `admin` role, who manage every article.
  Nothing is granted by default.
- `dynbrain_access` holds the result and is the only permission source retrieval reads. It is rewritten for the
  affected subtree in the same transaction as the grant change, the new article or the move that alters it, under a
  lock on the space row, never in the background.
- `BrainRepository::find_level` answers the highest level a user reaches an article at, through a row for them, one of
  their roles or one of their live groups, and only for an active account.
