---
name: project-context
description: >-
  Retrieve local project memory through the project-context MCP server before
  answering or planning. Use for past decisions, requirements, acceptance
  criteria, Jira/Confluence context, meeting outcomes, ADRs, OpenSpec specs, or
  implementation plans that may depend on prior project context.
---

# Project Context

Use this skill when an answer may depend on knowledge outside the current source
files: project history, requirements, acceptance criteria, decisions, meetings,
Confluence, Jira, ADRs, or OpenSpec.

`project-context` is the project's local memory. It indexes normalized project
documents locally and exposes excerpts through MCP. It is for grounding answers
and plans in existing project context, not for live source-code search or live
Atlassian state.

## When to use

- Before explaining why something was designed or decided.
- Before planning work that may already have requirements, ACs, proposals, or
  prior discussion.
- When the user mentions Jira, Confluence, meetings, ADRs, OpenSpec, specs,
  requirements, decisions, or acceptance criteria.
- When you would otherwise infer business/project context from code alone.

## Source Lookup Contract

For business/source-of-truth questions, use project-context first. This includes:
- what the client said in workshops or meeting notes;
- SRD/FSD/ISD/Jira/Confluence scope and acceptance criteria;
- why a requirement exists, whether something was discussed, and what decision was made.

Do NOT start with filesystem grep for those questions. Use project-context to find the cited source and
line range, then open the cited file only when the exact wording matters.

Filesystem search is appropriate after project-context has identified a source, or when the question is
about live source code that project-context does not index (implementation files, routes, components,
cartridges, tests).

If live infrastructure is unavailable (for example missing `b2c` CLI, no sandbox, no installed
dependencies), do not spend multiple rounds probing equivalent dead ends. Record the gap as an
implementation spike/blocker and continue with the best documented evidence.

It complements the live `openspec/` tree and Jira. Use it to find relevant
context, then open the cited file when details matter.

## Source Authority & Temporal Ranking

Retrieval ranks by semantic similarity, not by authority or recency. A result is
a lead, not a verdict. Before you use any retrieved fact to state scope, a
decision, or a status, apply this contract. It is binding, not advisory.

1. Read the metadata first. Every result carries `source_type`, `date`, and
   `date_source`. Never use a fact to set scope/decision/status without first
   weighing all three.
2. Authority classes, highest to lowest, for any claim about scope, sign-off, or
   a decision's status:
   1. Signed / approved documents (SRD, BRD, contracts; an approved tracker
      requirement).
   2. Reconciled OpenSpec artifacts (`discovery`, `design`, `specs`) and their
      recorded dispositions.
   3. ADRs (durable architecture decisions).
   4. Jira/Confluence status fields and ticket state.
   5. `meeting_notes` and workshop transcripts — PROVISIONAL: raw talk, not a
      signed decision.
   6. Derived summaries / indexes — orientation only; NOT a source.
3. Provenance rule: a lower class never establishes scope, sign-off, or a settled
   decision on its own. A remark in `meeting_notes` is a CANDIDATE (a new ask, an
   open question, a constraint to confirm) until a higher class ratifies it.
   Restating it in the artifact as settled is fabrication of authority.
4. Temporal rule: within one authority class, a later `date` supersedes an
   earlier one; surface the supersede, do not merge both. Across classes,
   authority wins over recency — a newer low-class remark does not override an
   older signed fact; raise the conflict instead. Trust `date_source` in the
   order `frontmatter` > `llm` > `mtime`; an `mtime` date is a filesystem
   artifact, not an authored date, so do not let it drive a supersede.
5. Downstream rule: when a reconciled upstream artifact (e.g. `discovery`) has
   already classified an item, that disposition governs the item's STATUS
   downstream. A raw lower-class source may add DETAIL to an item the upstream
   carries as in-scope; it may not re-open, re-scope, or resurrect an item the
   upstream deferred, left unsigned, or omitted.
6. Unrankable conflict: when two sources genuinely conflict and neither authority
   nor date can break the tie, record it as an open question — never silently
   pick one.

## Sources (what each bucket actually holds)

Each `source_type` indexes a specific corpus with a specific origin and coverage.
Knowing the essence is what lets you scope a query correctly.

- `meeting_notes` — hand-authored transcripts and notes from client workshops,
  discovery sessions, and tech calls, dropped into the index and shared via git.
  A CURATED subset (only what the team saved), not a mirror of any system.
  PROVISIONAL: raw talk, not a signed decision (lowest authority — see below).
- `confluence` — a FULL MIRROR of the configured Confluence space(s): every
  current page, plus footer comments (on by default). Refreshed from Atlassian on
  sync; deleted pages drop out. This is the reconciled project documentation:
  requirements, designs, playbooks, and FSD/ISD pages live here. May include
  **legacy reference** docs (see below) — treat those as reference only.
- `jira` — issues from the configured project(s) (a subset defined by project keys
  and an optional JQL filter, not all of Jira), including comments and status.
  Refreshed incrementally on sync. Use for ticket scope, acceptance criteria,
  status, and issue history.
- `specs` — the CURRENT OpenSpec capability specs only (`openspec/specs/**/spec.md`),
  mirrored read-only on each index. This is "what the system should do now."
- `openspec_archive` — historical rationale from COMPLETED (archived) OpenSpec
  changes: `proposal.md`, `design.md` (solution shape), and `technical-design.md`
  (concrete file/route/loader wiring) from `openspec/changes/archive`. Delta
  `spec.md` is deliberately excluded (current behavior comes from `specs`).
  Opt-in: use for history only, never for active FSD/ISD publish metadata, Page ID
  lookup, approval state, or delta/conflict review.
- `adrs` — hand-authored, git-shared Architecture Decision Records: the durable
  record of WHY an architecture choice was made.

Actual `source_type` values come from the MCP schema/config. If unsure, call
`list_sources`.

## Choosing categories (`source_type` is REQUIRED)

`search` takes a REQUIRED `source_type` ARRAY — you must consciously scope every
query. Pick the SMALLEST set of categories that could hold the answer; a tight
scope both sharpens results and shrinks the candidate pool before reranking. Map
the intent of the question to categories:

- Why something was decided / rationale → `["meeting_notes", "confluence", "adrs"]`
- Current requirements / acceptance criteria / scope → `["confluence", "jira", "specs"]`
- Ticket state, comments, estimates → `["jira"]`
- Current expected behavior of a capability → `["specs", "confluence"]`
- Durable architecture decisions → `["adrs"]`
- What the client said in a workshop / raw discussion → `["meeting_notes"]`
- Historical rationale from a COMPLETED change → `["openspec_archive"]`

Rules:
- Broad / "I don't know which bucket": do NOT omit the filter (you can't) and do
  NOT guess one type — pass ALL live categories explicitly:
  `["meeting_notes", "confluence", "jira", "specs", "adrs"]`, then narrow on the
  next call.
- Archive is opt-in: `openspec_archive` is NEVER included unless you list it. Add
  it only for historical rationale, never for current scope, status, or approval
  state (see Source Authority below).
- Do not invent values; if unsure which exist, call `list_sources`.

## Legacy documentation (reference only — never source of truth)

Project-context contains **legacy reference documentation** describing the *previous* Charles
Tyrwhitt SFCC (SiteGenesis/SFRA) storefront, imported to explain how the old platform behaved. It is
historical background for the current Storefront Next build, not a specification of it.

- **Never a source of truth.** Legacy docs never establish scope, requirements, acceptance criteria,
  or a settled decision for the current platform. They sit at the **lowest authority** ("orientation
  only; NOT a source" in the ranking above) and can be outdated or contradict current behavior.
- **Always double-marked** so retrieval and readers cannot confuse them with current docs:
  1. **Filename** — `legacy-<name>.gitkeep.md` (the `legacy-` prefix; `.gitkeep` keeps them committed).
  2. **In-document banner** — a `⚠️ LEGACY REFERENCE — NOT A SOURCE OF TRUTH` block at the very top.
- **How to use them.** Only as reference: to understand prior behavior, integrations, or intent when
  planning the new implementation. For anything authoritative, defer to approved requirements, OpenSpec
  specs, ADRs, and Jira, and verify against the live codebase. When citing legacy content, label it
  explicitly as legacy/reference, never as current scope or a decision.

## Tools

```
search(query, source_type[], top_k?)
```

- `query`: natural-language project-context query. Prefer English and project
  terms/IDs from the documents, for example `"SSO OIDC configuration decision"`
  instead of `"auth"`.
- `source_type`: **REQUIRED array** of one or more categories (see *Choosing
  categories* above). To search broadly, pass all live categories explicitly;
  archive is only searched when you list it. Do not invent values; use
  `list_sources` or the tool schema.
- `top_k`: optional. The server uses the configured default when omitted; use
  10-15 for synthesis or broad planning questions.

Examples:

```
search("SSO OIDC configuration decision", ["meeting_notes", "confluence", "adrs"])
search("store locator acceptance criteria", ["confluence", "jira", "specs"])
search("consent manager", ["meeting_notes", "confluence", "jira", "specs", "adrs"])  # broad
```

Results include `text`, `title`, `source_type`, `file`, `lines`, `date`,
`date_source`, `score`, and chunk position. `file` is workspace-relative and
normally points under `.cursor/.project/project-context/inbox/<source_type>/`.
When citing a result in OpenSpec artifacts, use the precise anchor
`<file>:<lines>` from the result, for example
`.cursor/.project/project-context/inbox/meeting_notes/discovery-workshop.md:12-28`.
Treat excerpts as leads; open `file` at `lines` for requirements, decisions, or
anything you will rely on.

```
list_sources()
```

Use before filtering, when deciding which source type to search, or when search
results look unexpectedly empty. It returns indexed source types, document
counts, and last sync time.

```
index_stats()
```

Use as a health check. It returns total chunks, per-source counts, and last sync
time.

## Usage pattern

1. Start with one specific `search` query, scoped to the smallest `source_type`
   set that could hold the answer (see *Choosing categories*).
2. Read the excerpts and dates.
3. Open cited files when the answer will drive implementation, scope, or a
   user-facing claim.
4. For synthesis, run 2-3 focused searches rather than one broad search — vary
   BOTH the query and the categories: decision/rationale
   (`["meeting_notes", "confluence", "adrs"]`), requirements/ACs
   (`["confluence", "jira", "specs"]`), constraints/status (`["jira", "specs"]`).
5. If results are empty or suspiciously stale, call `list_sources` or
   `index_stats` and tell the user the index may need a sync.

When answering, distinguish found context from your inference. If the MCP did
not return evidence for a claim, say so instead of presenting it as documented.

## Keeping it current

If results look stale or empty, run `/project-context:sync` to pull fresh Confluence + Jira
and rebuild the index.
