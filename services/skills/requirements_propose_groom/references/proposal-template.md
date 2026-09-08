# <Change> — Proposal

<!-- Comments here are local section-filling hints only. The change name/title is the
     discovery-resolved owned change, not the raw feature name. -->

## Executive Summary

<!-- Reader map (not a recap), derived LAST from the finished body: a reader-first opening
     (problem/change context + one business-value sentence), then what this offer SETTLES, what it
     HANDS DOWNSTREAM, and what STAYS OPEN — by subject and count, no S#/RQ/CR/Q inventories, no
     pending item implied approved, readiness mirroring the Proposal Gate. See the schema EXECUTIVE
     SUMMARY rule. -->

## Document Consumption Guide

<!-- Map of THIS document's current sections: one row per ## section in document order (excludes
     Executive Summary, this guide, and Proposal Gate). `Key data` is identifiers/sets, `None` for
     an empty set — re-derived from the body, never carried over. See the schema DOCUMENT
     CONSUMPTION GUIDE ROWS rule. -->

| Section | What it settles & why it matters downstream | Key data |
|---|---|---|
| <section name> | <what it settles & why it matters downstream> | <identifiers/sets, or `None`> |

## Tracker

<!-- Tracker reference from the sources (e.g. JIRA `PROJ-1234` — "<title>"). If the owned change has
     no dedicated ticket, say which consumer/feature ticket provides context; never invent one. -->

## Proposal Source Inventory

<!-- Point at the upstream artifacts and the key sources behind the lock; do not restate them. -->

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/<name>/discovery.md` | <trace and dispositions, dependencies, CRs, open questions, targets> |
| Design (this change) | `openspec/changes/<name>/design.md` | <solution shape, operational prerequisites, risks> |
| <BRD / SRD / notes> | <project-context anchor> | <what it grounds> |

## Requirements Lock

<!-- The BA/SA scope lock downstream artifacts consume. One row per discovery Requirements Trace row,
     grouped by WBS, disposition inherited verbatim. Do not add scope absent from discovery; new asks
     become CR candidates there first. Notes carry inherited cross-links (Q#/CR#/dependency/owner)
     or `—`. -->

| WBS | RQ ID | Requirement summary | Disposition | Notes |
|---|---|---|---|---|
| <WBS> | <RQ-ID> | <short faithful summary> | in scope / out of scope / deferred / CR candidate | <inherited cross-links or `—`> |

## Change Request Dispositions

<!-- One row per discovery CR candidate with the disposition PROPOSED for grooming; proposing is not
     approving. ALWAYS a table; if discovery carries none, emit `| None | — | — | — | <why> |`. -->

| CR# | Description | Impact | Proposed disposition | Status |
|---|---|---|---|---|
| CR1 | <what changes vs the signed scope> | <scope/effort impact> | confirm with client / absorb / descope | pending grooming approval |

## Re-usability Legend

<!-- PRINT IN FULL, DON'T AUTHOR. This is a fixed KEY the client reads the same way on every
     document, not a selection: keep EVERY bullet below, in this order, wording VERBATIM — never
     drop a member because no slice carries its marker. The gloss defines the marker, never what
     this change does with it (that delivery detail belongs in the criteria), so it is never
     re-authored into case-specific prose. The six are the closed set and are never reworded or
     renamed; a project MAY add its own vendor member in the same terse shape (e.g. `[CMP OOTB]`
     — delivered by the consent platform as licensed, consumed as published), which then prints
     on every document too. Selecting applies to the Acceptance Criteria groups below, never here. -->

- `[SF Next OOTB]` — base SF Next functionality, used as delivered
- `[3rd party OOTB]` — delivered by the third-party service as licensed, consumed as published
- `[SF Next Modified]` — base SF Next functionality that must be changed / adapted
- `[Legacy AS IS]` — ported from the legacy codebase to SF Next without behavioural change
- `[Legacy Modified]` — ported from the legacy codebase but must be changed / adapted
- `[New]` — brand-new functionality, built from scratch

## Delivery Breakdown

<!-- The offer's core: the in-scope lock rows sliced into per-slice BLOCKS grooming approves and
     stories/specs/FSD consume. Coverage law is a BIJECTION: one block per in-scope requirement, each
     covering EXACTLY ONE clean WBS (the SAME leaf code the FSD block is keyed by, e.g. `CHK.PAY`, NOT
     an RQ ID) — no grouping (never fold two requirements into one block), no splitting (never spread one
     across blocks), nothing outside the lock's in-scope set. Block count = in-scope requirement count =
     FSD in-scope block count (the scaffold script errors otherwise). Each block is a
     `### S# — <WBS> — <name>` heading, a machine-readable metadata list, a `#### User Story`, and a
     `#### Acceptance Criteria`. Authored NEAR-FINAL — the proposal is the single source of truth the FSD
     PROJECTS read-only. Story (STORY-LOOKUP LAW — a copy, not authoring): join this slice's
     `Covers (WBS)` to the discovery Requirements Trace row with that WBS and copy its `Requirement` cell
     VERBATIM (persona/wording intact; only a trailing `[Q#]` may be added). Do NOT re-author, paraphrase,
     shorten, or re-persona (keep "As a user" as "As a user"). Identical for functional AND integration —
     an integration slice copies the SAME cell; the framing lives ONLY in the slice name and Acceptance
     Criteria, NEVER in a rewritten "As a <consuming developer>…" story. Acceptance
     Criteria: numbered SHALL lists grouped under `#####` RE-USABILITY MARKER headings from the closed
     legend (`[SF Next OOTB]` base SF Next used as is · `[3rd party OOTB]` covered vendor-side, nothing
     for us to build · `[SF Next Modified]` base SF Next adapted · `[Legacy AS IS]` ported from legacy,
     behaviour unchanged · `[Legacy Modified]` ported from legacy, behaviour changed · `[New]` from
     scratch). Take ONLY the markers that apply — a slice whose criteria share one footing carries ONE
     group; `## Re-usability Legend` above prints the whole key either way. The marker
     groups are the ONLY heading axis here: there is NO device / surface / configuration axis, so device
     context sits inside the criterion ("On mobile and tablet, <the surface> SHALL …") and the
     administering actor is the criterion's subject ("Business users SHALL be able to …"). Where this slice HOSTS a
     surface whose content a sibling change or a vendor OWNS, open the block with a one-line boundary
     note naming that owner and what this slice delivers instead; a sibling change's deliverable gets the
     note, never a marker. PROJECT DESIGN CROSS-CUTTING:
     for each design `Base Configuration & Cross-Cutting` row whose `Applies to` names this slice's WBS, emit
     the matching AC obligation — a merchant/admin-config row becomes a criterion whose subject is the
     administering business user; a
     privacy/security/accessibility/i18n/performance row becomes a cross-cutting criterion in the owning
     slice;      an applicable row that reaches draft with no AC is a DROPPED obligation. The seed is the discovery
     trace `Requirement Details` + `Notes` for this WBS (NOT the raw SRD) — ELABORATE them into this
     slice's AC, never copy a Details line as a finished criterion. ELABORATE = BASELINE-DELTA against
     design (`Gap Analysis` gap class / `Base Configuration` / `Architecture Decisions` / integration
     `Integration Specification`): per criterion state EITHER a NET-NEW obligation (gap `rebuild`, no
     platform primitive) as the capability's own contract, OR a DELTA (gap `config`/`adapt`) naming the
     platform primitive it builds on + what this capability adds; a criterion that only re-states a
     Details line (no net-new obligation, no baseline delta) is PARROTING.
     PHRASING: open each criterion with a verb naming what THIS capability provides or guarantees; a seed
     clause may appear only as that verb's object, never as the criterion's leading verb and never with its
     mechanism token. One obligation per criterion, observable rather than subjective (no "fast", "easy",
     "seamless"), with a concrete subject. Before drafting, label each intended criterion
     net-new | delta | seed-echo and rewrite or drop every seed-echo.
     On an INTEGRATION slice additionally: drop the shopper-visible UI outcome, and
     lift a named vendor SDK/API up to the capability surface it provides, leaving the concrete mechanism
     to the design's Integration Specification. No Given/When/Then blocks (specs own those), no
     consumed-vendor names, no code tokens; undecided items keep their Q# reference. On an INTEGRATION
     change replace the `- **FSD:**` pointer with `- **ISD:**`. -->

### S1 — <WBS> — <requirement name>

- **Covers (WBS):** <one clean WBS code, e.g. CHK.PAY>
- **Capability:** <capability from discovery>
- **FSD:** fsd/<feature>.md
- **Depends on:** <External Dependencies & Access row(s) or `—`>

#### User Story

<the discovery Requirements Trace `Requirement` cell for this slice's Covers (WBS), copied VERBATIM — same for functional AND integration slices>

#### Acceptance Criteria

<!-- Optional first line, ONLY where this slice hosts a surface someone else owns:
<one-line boundary note — what the owning change/vendor owns, and what THIS slice delivers instead>
-->

##### [<marker>]

1. <The capability / the named surface / the business user> SHALL <near-final, testable,
   plain-business obligation on that footing>.

<!-- Repeat one `##### [<marker>]` group per footing this slice actually has; a slice whose criteria
     share one footing carries a single group. -->

<!-- Repeat one block per in-scope requirement. -->

## Out of Scope & Boundaries

<!-- Deferred lock rows, unsigned CR topics, and consumed capabilities owned by other changes — name
     the owning change/capability. Nothing listed here may appear in a slice. -->

## External Dependencies & Access

<!-- Build-prerequisite register grooming turns into prerequisites. Functional scope: consumed
     upstream deliveries / existing services (no vendor credentials — the owning integration changes
     carry those). Integration scope: operational prerequisites (account / credentials / billing /
     sandbox / environment) with factual statuses. Never the change's own work items. Lead "Needed
     for" with the S# slice(s) the row blocks. Every row verified against the upstream
     artifacts/sources or marked `UNVERIFIED — confirm before proposal merge`. ALWAYS a table; if
     genuinely none, emit `| None | — | — | <why> |`. -->

| Dependency / Access | Needed for | Owner / Source | Status |
|---|---|---|---|
| <delivery, service, account, credential, environment, or access> | <S#(s) + RQ(s) / build need> | <owning change, team, or vendor> | have / requested / not requested / <factual unresolved status> |

## Approach

<!-- SHORT compression (1-2 paragraphs): how the offer lands per the design — one grounded sentence of
     the solution shape, an explicit pointer that decisions/rationale live in design.md (integration:
     the service contract lives in the design's Integration Specification), the FSD/ISD working copies
     from discovery's targets, and the testing obligation. No implementation detail; undecided items
     stay undecided (reference their Q#). -->

## Open Questions

<!-- ONLY questions newly raised at proposal time; do not re-list or close discovery/design Q#s.
     HUNT first: walk the lock / breakdown slicing / dependency verification for genuine new
     ambiguities before defaulting to None. ALWAYS a table; with no new questions emit
     `| None | <why> | — | — |`, where <why> names what you checked (never a bare "all inherited"). -->

| Q# | Question | Owner | Status |
|---|---|---|---|
| <Q#> | <question>? | <BA / SA> | Raised for grooming |

## Proposal Gate

<!-- State AFTER the body. Mark each item checked ([x]) as the passed self-check; leave a box
     unchecked only if the item genuinely does not hold. -->

- [x] Requirements Lock mirrors every discovery trace row exactly once with inherited dispositions
- [x] Delivery Breakdown is 1:1 with the in-scope requirements — each in-scope requirement delivered by exactly one slice and each slice delivers exactly one in-scope requirement (no grouping, no splitting); every slice traces to the lock
- [x] No scope absent from discovery was introduced (new asks are CR candidates in discovery first)
- [x] Every Change Request disposition is proposed for grooming, none approved here
- [x] Every External Dependencies & Access row is verified against the upstream artifacts or explicitly marked UNVERIFIED
- [x] Slices and Approach inherit the design without re-deciding architecture or adding implementation detail

## Review & Approval

<!-- Human sign-off gate for this phase. After generating this artifact the workflow command STOPS
     here and runs the Per-Artifact Review Gate (see rules/workflow/openspec-workflow/openspec-workflow.mdc): a short
     summary + open questions, then Approve / Request changes — plus Walk the questions
     (recommended) ONLY when this artifact still has open questions in this pass (with none,
     drop Walk; never re-walk already-settled decisions). The agent NEVER checks this box and NEVER names itself as
     approver; only a named human approves after review. The next phase stays blocked until Approved
     is checked with a real human in "Approved by". This is separate from the Proposal Gate above,
     which is the agent's self-check. Note: the full requirements/scope grooming still happens in the
     grooming artifact — this box is the reviewer's sign-off that the proposal is ready for grooming. -->

- [ ] **Approved** — a human reviewed this artifact and approves proceeding to the next phase
- **Approved by:** —   <!-- real human name or role; never Agent / LLM / Cursor / blank -->
- **Date:** —
- **Review notes:** —   <!-- optional: what was reviewed, deferrals, follow-ups -->
