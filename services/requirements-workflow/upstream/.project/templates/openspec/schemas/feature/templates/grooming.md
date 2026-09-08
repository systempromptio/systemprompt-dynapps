# <Change> — Grooming

Status: **DRAFT — human approval required.** The agent prepared this ledger; no item is decided until a named human approves it in the grooming session.

<!-- Comments here are local section-filling hints only. The agent prepares; humans decide: every Decision / Approved by
     cell ships `—`, every Status OPEN (or grounded BLOCKED), every gate box unchecked. Valid statuses:
     OPEN / BLOCKED / APPROVED / DEFERRED-APPROVED / ROUTED-TO-PLANNING / OWNED-BY-OTHER-CHANGE. "Approved by:
     Agent/LLM/Cursor" or blank is invalid. The one draft exception is a resolved-upstream item (an upstream already
     closed it): it arrives already decided, carrying its upstream decision, named approver, and Origin back-link. -->

## Executive Summary

<!-- Written last, reader-first: what upstream settled (one sentence), what this session must settle
     (pending items by type, blockers named, planning routes to acknowledge, any resolved-upstream item
     migrated for the record), and the explicit statement that nothing is approved yet and the pipeline
     stays blocked until the Grooming Approval Gate passes. No Q#/CR#/P#/PD# inventories. -->

## Grooming Source Inventory

<!-- Keep the header row verbatim: `Input | Reference | Used for` — do NOT relabel `Used for` as `Purpose`/`Use`/`Notes`. -->

| Input | Reference | Used for |
|---|---|---|
| Discovery (this change) | `openspec/changes/<name>/discovery.md` | <Q# origins and severities, CR origins, source anchors> |
| Design (this change) | `openspec/changes/<name>/design.md` | <settled decisions; risk context; Planning Deferrals PD# (or None)> |
| Proposal (this change) | `openspec/changes/<name>/proposal.md` | <lock, breakdown, CR dispositions, access register> |
| Tracker | <tracker ref from the sources, verbatim> | <session reference> |

## Prerequisites & Access

<!-- One P# row per proposal "External Dependencies & Access" row. Origin back-links to the upstream section that
     raised it. Inherit the factual status verbatim into "Current status"; the grooming Status starts OPEN (BLOCKED
     only where a source records a blocker). A prerequisite gated by an open Q# references that Q# — the question
     lives once, in its own register. -->

| P# | Prerequisite / access | Origin | Needed for | Owner / provider | Current status (from sources) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|
| P1 | <what is needed> | <upstream artifact / section that raised it> | <S#(s) / RQ(s) it gates> | <owning change, team, or vendor> | <inherited factual status> | — | — | OPEN |

## BA / Requirements Questions

<!-- Business/requirements ambiguity migrated from upstream Open Questions (dedupe by Q#; severity verbatim). Origin
     back-links to the upstream artifact/section (or the grounding fact for a session-raised item). 2-3 option
     one-liners with implications from the upstream analysis; recommendation labeled as the agent's. ALWAYS a table;
     with no items emit `| None | — | — | <where the open items actually live> | — | — | — | — | — | — |`. -->

| Q# | Origin | Severity | Question | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| <Q#> | <discovery/design section that raised it> | CRITICAL / NICE-TO-HAVE | <question>? | <which slice/requirement/prerequisite it gates> | (a) <option — implication>; (b) <option — implication> | <labeled recommendation or —> | — | — | OPEN |

## Architecture / Tech Decisions

<!-- Technical/operational decisions: migrated tech-natured Q#s and design-raised undecided decisions. Origin
     back-links to the upstream artifact/section/decision. NEW items must pass the arch filter (multiple viable
     options; material scope/risk/timeline impact; rework if deferred) — no boilerplate. ALWAYS a table; with no
     items emit the explicit `None` row. -->

| Q# | Origin | Severity | Decision needed | Context (why it matters) | Options (implications) | Agent recommendation | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|---|---|
| <Q#> | <design decision / section that raised it> | CRITICAL / NICE-TO-HAVE | <decision>? | <what it determines> | (a) <option — implication>; (b) <option — implication> | <labeled recommendation or —> | — | — | OPEN |

## Planning Routes

<!-- One PD# row per design "Planning Deferrals" row — developer-HOW items design routed to technical-design planning.
     Grooming acknowledges the ROUTE (a named human confirms ROUTED-TO-PLANNING); it does NOT decide the developer-HOW.
     Origin back-links to the design section/decision. Draft: Route acknowledgement / Approved by `—`, Status OPEN.
     ALWAYS a table; when design defers nothing emit `| None | design Planning Deferrals | <design defers nothing> | — | — | — | — | — |`. -->

| PD# | Origin (design section/decision) | Deferred item | Target phase | Owner | Route acknowledgement | Approved by | Status |
|---|---|---|---|---|---|---|---|
| <PD#> | <design Planning Deferrals — section/decision> | <the developer-HOW item, verbatim from design> | planning | <owner from design> | — | — | OPEN |

## Change Request Decisions

<!-- One row per proposal Change Request row, proposed disposition inherited verbatim; the session decides. Origin
     back-links to the discovery Change Request Candidate (and the review where it was decided, for a resolved-upstream
     CR). A resolved-upstream CR arrives already decided — record its upstream decision, named approver, and Origin;
     do not re-open it. ALWAYS a table; when the proposal carries the explicit `None` row, carry it with its reason. -->

| CR# | Origin | Description | Impact | Proposed disposition (from proposal) | Decision | Approved by | Status |
|---|---|---|---|---|---|---|---|
| <CR#> | <discovery Change Request Candidates CR#; + review if already decided> | <what changes vs the signed scope> | <scope/effort impact> | <inherited proposed disposition> | — | — | OPEN |

## Scope & Offer Approval

<!-- Exactly two rows. Origin is the proposal section the offer projects. Summarize the offer as proposal.md states it —
     changing the lock boundary or slice set here is re-deciding scope. -->

| O# | Origin | Item | What the human approves | Decision | Approved by | Status |
|---|---|---|---|---|---|---|
| O1 | proposal Requirements Lock | Requirements Lock | <the lock boundary as proposed: in-scope set, deferred rows, nothing else introduced> | — | — | OPEN |
| O2 | proposal Delivery Breakdown S1–Sn | Delivery Breakdown | <the slice set S1–Sn with acceptance-criteria seeds as the approved slicing for specs and stories> | — | — | OPEN |

## Grooming Approval Gate

All items above must be APPROVED or DEFERRED-APPROVED by a named human before specs, FSD/ISD, or stories are generated.

- [ ] Every prerequisite (P#) is APPROVED, or DEFERRED-APPROVED with a named human approver and an explicit constraint
- [ ] Every BA / requirements question (Q#) is APPROVED or DEFERRED-APPROVED by a named human
- [ ] Every architecture / tech decision is APPROVED or DEFERRED-APPROVED by a named human, or a named human confirmed none is required
- [ ] Every planning route (PD#) carried from design is acknowledged as ROUTED-TO-PLANNING by a named human, or the register is empty
- [ ] Every Change Request (CR#) has a disposition decided by a named human
- [ ] The Requirements Lock and Delivery Breakdown (O1, O2) are approved by a named human
- [ ] Every migrated or raised item back-links to the upstream artifact section or decision that raised it
- [ ] Approved decisions are written back into proposal.md / design.md where they change scope or approach
- [ ] No approver is "Agent", "LLM", "Cursor", blank, or implied
