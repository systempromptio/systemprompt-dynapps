# Project Context (knowledge bank)

The knowledge bank is the project's memory: the Confluence pages and Jira
issues you can read, indexed with citations and shared with everyone else who
can read them. It ranks hits by **similarity**, not by authority or recency —
a hit is a lead, not a verdict. This skill is the contract for using it well.

## Connect

The knowledge bank is a remote MCP server on the Astound instance; nothing
runs on your machine. Add it to your project's `.mcp.json` (or the equivalent
in Cowork) with three headers:

```json
{
  "mcpServers": {
    "knowledge": {
      "type": "http",
      "url": "https://astound.systemprompt.io/api/v1/mcp/knowledge-bank",
      "headers": {
        "Authorization": "Bearer ${SYSTEMPROMPT_PAT}",
        "x-atlassian-email": "${ATLASSIAN_EMAIL}",
        "x-atlassian-api-token": "${ATLASSIAN_API_TOKEN}"
      }
    }
  }
}
```

- `Authorization` is your systemprompt personal access token — who you are.
- `x-atlassian-email` and `x-atlassian-api-token` are **your own** Atlassian
  account and API token (create one at id.atlassian.com → Security → API
  tokens). They are used only by `sync_knowledge`, only to read Confluence and
  Jira as you, and are never logged or stored.
- `x-atlassian-base-url` is optional; it overrides the instance's configured
  site when you need another one.

`sync_knowledge` indexes what **your** Atlassian account can read in the
configured spaces and projects and grants each document to you. Search only
ever returns documents you have synced. A page a colleague has also synced is
one shared row — embedded once, granted to both of you — so the index is the
union of what its users could read, never more than any one of them could.
If `sync_knowledge` answers `missing header x-atlassian-api-token`, the
headers above are not reaching the server.

## Tools

| Tool | Use |
|---|---|
| `search_knowledge(query, source_types?, top_k?)` | Find grounded excerpts. Every hit carries `source_type`, `date`, `date_source`, `uri#lines`. |
| `list_knowledge_sources()` | The categories, with **your** document counts and **your** last sync per category. Call it when results look empty or before an unfamiliar scope. |
| `sync_knowledge(source_types?, rebuild?)` | Index what your Atlassian account can read; runs in the background. `rebuild` ignores your sync cursor and re-chunks everything you fetch. Needs the Atlassian headers. |
| `knowledge_index_stats()` | Administrators only — index health: totals, syncing users, grants, cache rows, database size. |
| `prune_knowledge(stale_days?, dry_run?)` | Administrators only — drop grants of users who have not synced in `stale_days` (default 90) or whose account is gone, delete documents nobody holds, prune caches. `dry_run` reports without deleting. |

## When to search — and search first

Use the knowledge bank when an answer may depend on knowledge outside the
files in front of you: before explaining *why* something was designed or
decided; before planning work that may already have requirements, acceptance
criteria or proposals; whenever the user mentions Jira, Confluence, meetings,
decisions or ACs; and whenever you would otherwise infer business context
from code alone.

For business and source-of-truth questions, query the knowledge bank
**first**, not filesystem grep. Use it to find the cited page or issue and
line range, then open it (the hit's `uri`) only when exact wording matters.
Filesystem search is for live code the bank does not index.

If a search returns nothing and `list_knowledge_sources` shows zero documents
or no sync for you, run `sync_knowledge` first — the index holds only what
its users have synced.

## Scope every query

Pick the **smallest** set of categories that could hold the answer — tight
scope sharpens results and shrinks the pool before rerank.

| Intent | `source_types` |
|---|---|
| Why decided / rationale, workshop outcomes, designs | `["confluence"]` |
| Current requirements / ACs / scope | `["confluence", "jira"]` |
| Ticket state, comments, estimates | `["jira"]` |
| Current expected behaviour of a capability | `["confluence"]` |

Omitting `source_types` searches every *live* category. Archive categories
(`archived: true` in the configuration) are searched only when you name them,
and only for history — never for current scope, status or approvals. Do not
invent category ids; call `list_knowledge_sources` if unsure.

## Source authority and temporal ranking (binding)

Before using a retrieved fact to state scope, a decision or a status:

1. **Read the metadata first** — `source_type`, `date`, `date_source` — on
   every hit.
2. **Authority classes**, high to low: (1) signed/approved documents (SRD,
   BRD, contracts, an approved tracker requirement); (2) reconciled design
   and specification pages and dispositions; (3) decision records; (4)
   Jira/Confluence status and ticket state; (5) meeting notes and workshop
   transcripts — PROVISIONAL; (6) derived summaries — orientation only, not
   a source.
3. **Provenance rule** — a lower class never establishes scope, sign-off or a
   settled decision alone. A meeting-notes remark is a CANDIDATE until a
   higher class ratifies it; restating it as settled is fabrication of
   authority.
4. **Temporal rule** — within a class, a later `date` supersedes an earlier
   one (surface the supersede, do not merge). Across classes, authority wins
   over recency. `date` is the page or issue's own version date
   (`date_source: frontmatter`) or, when the text names a better one, the
   date the model extracted (`date_source: llm`); trust `frontmatter` over
   `llm`.
5. **Downstream rule** — a reconciled upstream artifact's disposition governs
   an item's STATUS; a raw lower-class source may add DETAIL but may not
   re-open, re-scope or resurrect what upstream deferred or left unsigned.
6. **Unrankable conflict** — when neither authority nor date breaks a tie,
   record an open question. Never silently pick one.

Documents marked `LEGACY REFERENCE` describe the previous system. They are
never a source of truth; label them as legacy whenever you cite them.

## Usage pattern

1. One specific `search_knowledge`, scoped to the smallest category set.
2. Read excerpts **and dates**.
3. Open the cited `uri#lines` when the answer drives implementation, scope
   or a user-facing claim, and cite that anchor.
4. For synthesis, run two or three focused searches varying both query and
   categories rather than one broad search.
5. If results are empty or suspiciously stale, call `list_knowledge_sources`
   and tell the user their index may need a `sync_knowledge`.

Always distinguish found context from inference. If the knowledge bank
returned no evidence for a claim, say so rather than presenting it as
documented.
