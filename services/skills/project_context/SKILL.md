# Project Context (knowledge bank)

The knowledge bank is the project's memory: Confluence, Jira, meeting notes,
ADRs and specs, indexed with citations. It ranks hits by **similarity**, not by
authority or recency — a hit is a lead, not a verdict. This skill is the
contract for using it well.

## Tools

| Tool | Use |
|---|---|
| `search_knowledge(query, source_types?, top_k?)` | Find grounded excerpts. Every hit carries `source_type`, `date`, `date_source`, `uri#lines`. |
| `list_knowledge_sources()` | The categories you may read, with your document counts and last sync. Call it when results look empty or before an unfamiliar scope. |
| `knowledge_index_stats()` | Index health: totals, last build, version. |
| `sync_knowledge(source_types?, rebuild?)` | Administrators only — refresh the index. |

## When to search — and search first

Use the knowledge bank when an answer may depend on knowledge outside the
files in front of you: before explaining *why* something was designed or
decided; before planning work that may already have requirements, acceptance
criteria or proposals; whenever the user mentions Jira, Confluence, meetings,
ADRs, OpenSpec, specs, requirements, decisions or ACs; and whenever you would
otherwise infer business context from code alone.

For business and source-of-truth questions, query the knowledge bank **first**,
not filesystem grep. Use it to find the cited source and line range, then open
the file (or the page, through the Confluence/Jira tools as yourself) only
when exact wording matters. Filesystem search is for live code the bank does
not index.

## Scope every query

Pick the **smallest** set of categories that could hold the answer — tight
scope sharpens results and shrinks the pool before rerank.

| Intent | `source_types` |
|---|---|
| Why decided / rationale | `["meeting_notes", "confluence", "adrs"]` |
| Current requirements / ACs / scope | `["confluence", "jira", "specs"]` |
| Ticket state, comments, estimates | `["jira"]` |
| Current expected behaviour of a capability | `["specs", "confluence"]` |
| Durable architecture decisions | `["adrs"]` |
| What the client said in a workshop | `["meeting_notes"]` |
| Historical rationale from a COMPLETED change | `["openspec_archive"]` |

Omitting `source_types` searches every *live* category. **Archive categories
are opt-in**: `openspec_archive` is searched only when you name it, and only
for history — never for current scope, status or approvals. Do not invent
category ids; call `list_knowledge_sources` if unsure.

## Source authority and temporal ranking (binding)

Before using a retrieved fact to state scope, a decision or a status:

1. **Read the metadata first** — `source_type`, `date`, `date_source` — on
   every hit.
2. **Authority classes**, high to low: (1) signed/approved documents (SRD,
   BRD, contracts, an approved tracker requirement); (2) reconciled OpenSpec
   artifacts (`discovery`, `design`, `specs`) and dispositions; (3) ADRs;
   (4) Jira/Confluence status and ticket state; (5) `meeting_notes` and
   workshop transcripts — PROVISIONAL; (6) derived summaries — orientation
   only, not a source.
3. **Provenance rule** — a lower class never establishes scope, sign-off or a
   settled decision alone. A `meeting_notes` remark is a CANDIDATE until a
   higher class ratifies it; restating it as settled is fabrication of
   authority.
4. **Temporal rule** — within a class, a later `date` supersedes an earlier
   one (surface the supersede, do not merge). Across classes, authority wins
   over recency. Trust `date_source` in the order `frontmatter > llm > mtime`;
   an `mtime` date is a filesystem artifact and must never drive a supersede.
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
   and `knowledge_index_stats`, and tell the user the index may need a sync.

Always distinguish found context from inference. If the knowledge bank
returned no evidence for a claim, say so rather than presenting it as
documented.
