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
