# <Change> — Discovery & Requirements

<!-- Comments here are local table-filling hints only. -->

## Executive Summary

<!-- Answer, from the sections below as they stand NOW: what has this file settled, what is still
     open, and is the change ready? Record the answer as a reader map that orients someone before the
     detailed sections — not a retelling. Open with the problem/change context and the business value
     in one short paragraph, then cover WHAT THIS FILE SETTLES (delivery scope, the shape of the
     requirement set, which capabilities move, and which section holds each), WHAT STAYS OPEN (the
     subjects still undecided and the register each sits in), and READINESS stated consistently with
     the Discovery Gate below.
     Every statement must already be readable in a section below; one those sections no longer
     support is wrong however well it is written.
     Name SUBJECTS, never identifiers: "the redirect for the retired /stores URL", not "Q2". Counts
     are fine ("three refinements", "nine in scope"); re-listing a table's rows is not. Identifiers
     and sets live in the Document Consumption Guide — this block and that table split the same facts
     along the prose/identifier line and never overlap. -->

## Document Consumption Guide

<!-- The interface a downstream reader (an LLM first, a human second) uses to consume this file
     without re-reading the whole body. It sits directly after the Executive Summary — the two-layer
     interface (prose overview + this navigation map) before the detail sections it projects. ONE row
     per consumed section: "What it settles & why it matters downstream" (navigation — a short
     human-readable phrase) plus "Key data" (the identifiers and sets that section exports). A
     PROJECTION, never a new source of truth — every value must already be readable in the section the
     row names, and one that section no longer supports is wrong however well it is written.
     FIXED INTERFACE: keep one row per consumed section below, in this order, every time. An empty
     set is written `None`; a row is never dropped, merged, renamed, or reordered. Key data carries
     identifiers and sets only (WBS codes, capability names with direction, Q#/CR# with status,
     paths) — no restated requirement titles. Write it after the body is final and re-derive it
     whenever the body changes. Each row's authority is defined in the schema's DOCUMENT CONSUMPTION
     GUIDE ROWS. -->

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| Requirements Trace | <the signed scope boundary this change builds, defers, or leaves to another owner> | In scope: `<WBS of every in-scope row>`; Deferred: `<WBS deferred, or None>`; Out of scope: `<WBS → owning change/team, or None>` |
| Affected Capabilities | <which capabilities move and how> | `<capability ADD / MODIFY / REMOVE, comma-separated>` |
| Dependencies | <external services consumed but not owned> | `<dependencies this change consumes, or None>` |
| Open Questions | <what stays open and gates readiness> | `<Q# — severity, status; only questions still Open, or None>` |
| Change Request Candidates | <asks against signed scope> | `<CR# — status, or None>` |
| Cross-cutting Requirements | <which cross-cutting areas apply> | `<areas marked yes; omit areas marked N/A, or None>` |
| Delivery Scope & Targets | <the resolved scope and what is published where> | `<functional / integration; FSD/ISD DISPOSITION → working-copy path>` |

## Discovery Source Inventory

<!-- Replace example rows with the actual inputs/evidence used; mark missing inputs explicitly.
     Use project-context inbox anchors as returned by search, e.g.
     `.cursor/.project/project-context/inbox/<source_type>/<file>.md:<lines>`.
     Keep NEW/EXTEND publication decisions in "Delivery Scope & Targets", not here. -->

| Source | Reference / Anchor | Purpose |
|---|---|---|
| BRD / SRD | `<Confluence URL/Page ID>` or `<exported file>:<lines>` | Canonical signed WHAT and official RQ/WBS trace |
| Existing OpenSpec baseline specs | `openspec/specs/<capability>/spec.md:<lines>` | Current behaviour and capability ownership |
| Legacy implementation / current storefront | `openspec/specs/<capability>/spec.md:<lines>`, `<live route/URL>`, or `— (legacy implementation not found via project-context; baseline spec + live URL serve as current-state evidence)`. Cite a `<code path>:<lines>` ONLY when project-context actually returns one; never invent a workspace path/filename/line range | Existing behaviour being modified, retired, or replaced |
| Legacy URL / live storefront page | `<URL or route>` | Current user journey, SEO exposure, redirect/retirement evidence |
| Meeting notes | `.cursor/.project/project-context/inbox/meeting_notes/<file>.md:<lines>` | Workshop decisions, CR candidates, open questions, constraints |
| Jira | project-context `jira`: `<issue key>` or `<Jira URL>` | Tracker context, comments, acceptance criteria |
| Design assets / Figma / committed screenshots | `<assets path>:<lines>`, `<Figma URL>`, or `—` | Visual source used, or explicit note that none was provided |
| FSD/ISD/TSD/API/config/source lookup | `<location or no matching source found>` | Supporting source or availability check; publication decision recorded later |

## Requirements Trace

<!-- One row per source requirement. Use source wording and the allowed dispositions.
     Requirement: the verbatim source requirement (e.g. the user story). Requirement Details:
     the verbatim detail list from the source (preserve its numbering/structure); use `—` when
     the source has no separate details column. Mirror the source columns; do not paraphrase or
     merge the two into one cell.
     WBS: ONE clean/leaf code per row (e.g. `FEAT.SUB`), never a coarse parent-only grouping
     (`FEAT`).
     Disposition and Delivery mode: the values the owner-based SCOPE DISPOSITION law and the
     DELIVERY MODE axis assign (DECISION KERNEL). Notes: material findings, or `—`.
     Which rows belong here at all is the trace lock; for integration scope, apply
     SOURCE-PARTITION FIRST before dispositioning consumer-worded rows. -->

| RQ ID | WBS | Title | Source requirement | Requirement | Requirement Details | Disposition | Delivery mode | Affected capability | Notes |
|---|---|---|---|---|---|---|---|---|---|
| RQ-001 | FEAT.SUB | <title> | <BRD/SRD/Jira/meeting citation> | <verbatim source requirement> | <verbatim requirement details or `—`> | in scope / out of scope / deferred / CR candidate | build / consume / configure / `—` | <capability> | <material finding or `—`> |

## Affected Capabilities

<!-- One row per capability THIS change owns. Functional changes may list modified/retired
     feature capabilities; integration-only changes should list only agnostic service
     integration capabilities and must not list consumer/legacy feature capabilities. For a
     functional change, include existing legacy capabilities/pages this change retires or
     supersedes as REMOVE, even when no signed RQ row maps directly to that legacy capability. -->

| Capability | Spec path | Disposition |
|---|---|---|
| <capability> | `openspec/specs/<capability>/spec.md` | MODIFY |
| <new-capability> | — (does not exist yet) | ADD (new) |
| <legacy-capability> | `openspec/specs/<legacy-capability>/spec.md` | REMOVE (retired) |

## Dependencies

<!-- External capabilities/services an in-scope requirement RELIES ON but this change does NOT
     own (owned by another change/team; consumed here) — apply the consume-vs-own test. This is
     not a scope-addition (that is a Change Request) and not a
     HOW-question (that is an Open Question). Write "None." when this change consumes nothing
     external — e.g. an agnostic integration that others consume but that depends on nothing.
     Do not list services used only by unapproved CRs, or browser/native platform APIs, here. -->

| Dependency | Type | Status | Used by |
|---|---|---|---|
| <service/capability> | external integration / data service | separate integration change / already integrated | RQ-... |

## Change Request Candidates

<!-- Remove example rows. Add CR rows as needed, or write "None." when there are no CR candidates.
     CRs are new/changed WHAT versus signed scope. Deferred signed requirements stay deferred in
     Requirements Trace; HOW/refinement decisions belong in Open Questions. -->

| CR# | Trigger | Description | Impact | Proposed disposition | Status |
|---|---|---|---|---|---|
| CR1 | <source> | <what changes vs the BRD> | scope+ / effort | confirm with client / absorb / descope | pending |

## Open Questions

<!-- Remove example rows. Add answerable questions as needed, or write "None." when no questions remain. -->

| Q# | Severity | Question | Why unclear | Tech note (if constrained) | Owner | Status |
|---|---|---|---|---|---|---|
| Q1 | CRITICAL / NICE-TO-HAVE | <question>? | <what is ambiguous> | <e.g. SCAPI basket has no hierarchical price adjustments> | client / BA | Open |

## Cross-cutting Requirements

<!-- Keep the four rows; fill each with yes or N/A plus a grounded one-line requirement or reason.
     For agnostic integration scope, Accessibility/SEO/i18n are usually N/A unless the integration
     itself produces UI, an indexable page, or user-facing copy; Consent/privacy is applicable when
     the service handles geolocation, PII, browser keys, tracking, or similar sensitive data. -->

| Area | Applicable? | Requirement summary (if applicable) |
|---|---|---|
| Accessibility (WCAG 2.2) | yes / N/A | <e.g. keyboard operable, focus order, SR labels> |
| SEO (meta, canonical, structured data, sitemap, noindex) | yes / N/A | <e.g. PDP needs product structured data> |
| Consent / privacy (tracking gating, cookies, DNT) | yes / N/A | <e.g. analytics events gated on consent> |
| i18n (user-facing strings) | yes / N/A | <e.g. all copy via translation keys> |

## Delivery Scope & Targets

<!-- Declare the scope first. The delivery scope is resolved up front and only recorded here.
     Use `functional` when this change produces
     an FSD and only references integrations; use `integration` when this change produces a
     service-oriented ISD for the one selected service. If the gate resolved to ambiguous
     (bundled functional + integration work with no chosen scope), the workflow stops before
     discovery with a decomposition instead of filling this file. Add only produced/touched
     documents; do not add N/A rows. For `integration` scope the ISD working copy lives under
     the service-owned change folder (`openspec/changes/<service-change>/...`), never under
     the consumer feature's / this command's change folder. -->

Scope: functional / integration — <short reason>

| Type | Disposition | Confluence target | Working copy |
|---|---|---|---|
| FSD | NEW / EXTEND | `<Confluence URL or TBD>` | `openspec/changes/<name>/fsd/<feature>.md` |
| ISD | NEW / EXTEND | `<Confluence URL or TBD>` | `openspec/changes/<service-change>/isd/<service-integration>.md` |

## Discovery Gate

<!-- Set checkboxes from the final Open Questions table. -->

- [ ] No CRITICAL open questions remain Open (each is Resolved with user/BA/SA sign-off, or consciously carried forward with user sign-off)
- [ ] Resolved answers captured back into the Open Questions table (Status + decision + signer in Owner/Status note)
- [ ] Executive Summary reflects discovery readiness without replacing the detailed sections
- [ ] Document Consumption Guide matches the sections it projects (scope, WBS sets, capabilities, dependencies, open items)

## Review & Approval

<!-- Human sign-off gate for this phase. After generating this artifact the workflow command STOPS
     here and runs the Per-Artifact Review Gate (see rules/workflow/openspec-workflow/openspec-workflow.mdc): a short
     summary + open questions, then Approve / Request changes — plus Walk the questions
     (recommended) ONLY when this artifact still has open questions in this pass (with none,
     drop Walk; never re-walk already-settled decisions). The agent NEVER checks this box and NEVER names itself as
     approver; only a named human approves after review. The next phase stays blocked until Approved
     is checked with a real human in "Approved by". This is separate from the Discovery Gate above,
     which is the agent's self-check. -->

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —   <!-- real human name or role; never Agent / LLM / Cursor / blank -->
- **Date:** —
- **Review notes:** —   <!-- optional: what was reviewed, deferrals, follow-ups -->
