# FSD/ISD Upstream Actualization (design / proposal / discovery / specs ← FSD/ISD)

A UNIQUE pass that runs in the OPPOSITE direction to the normal flow. Normally the upstream artifacts are
canonical and the FSD/ISD project DOWN from them. But the FSD/ISD are the client-SIGNED WHAT — the primary
downstream source of truth for technical-design and build — and in practice a team tailors or hardens the
real contract ON the FSD/ISD at the review / approval gate: resolving a call placement, pinning an auth
mechanism or endpoint, tightening a client-visible bound, correcting a mapping. At that point the upstream
`design.md` / `proposal.md` / `discovery.md` / delta specs no longer match the contract the client actually
signed. This pass propagates those FSD/ISD-driven changes back UP so every upstream artifact stays true to
the signed WHAT, instead of leaving them silently stale and fixed by hand later.

Scope and boundary:

- It actualizes each upstream artifact WITHIN ITS OWNERSHIP — design (architecture + integration
  contract), proposal (Requirements Lock / Delivery Breakdown), discovery (Delivery Scope & Targets /
  obligations), delta specs (behaviour canon). It never invents scope; it only reconciles what the FSD/ISD
  hardened.
- It is **outside the eval contour** — no LLM-as-judge coverage, by conscious decision, because
  reconciling a signed contract back into upstream artifacts is too open-ended to score reliably. Treat it
  as a human-supervised consistency pass, not an evaluated artifact.
- The FSD/ISD scaffold sub-sections (`#### User Story` / `#### Acceptance Criteria` / `#### Key
  behaviours`) are DETERMINISTIC projections from the proposal slice + specs. NEVER hand-edit them and
  NEVER reconcile a behaviour change by editing them here: a hardened behaviour that belongs to a scaffold
  section is fixed UPSTREAM (the covering proposal slice or spec requirement) and the scaffold script is
  re-run. This pass reconciles the ENRICHED / hardened non-scaffold content (integration contract,
  governance, constraints, dispositions, rationale).

## When it runs (trigger)

At the **FSD/ISD review gate** — after the FSD/ISD has been reviewed and its contract is settled (approved,
or changed during review) — and again re-verified at **`/opsx:propose-submit`** BEFORE the artifacts are
committed and published. Check whether the settled FSD/ISD tailored or hardened anything the upstream
asserts. If it did, run this pass BEFORE the artifacts leave the gate / are committed. If the FSD/ISD merely
projected the upstream verbatim with no tailoring, there is nothing to actualize and the pass is skipped.

## What to reconcile (each upstream against the settled FSD/ISD)

Walk the settled FSD/ISD against each upstream artifact and, for each item, KEEP (upstream still matches) or
FIX (the FSD/ISD moved it):

1. **Contract / behaviour drift.**
   - ISD: endpoints, fields, auth, status/fault codes, quota/TTL, or the SDK interface the ISD finalized
     that the design's `## Integration Specification` now contradicts, lacks, or states more vaguely —
     update the design to agree with the ISD's settled wire/interface at architecture level (no
     developer-implementation detail leaks upward).
   - FSD: a client-visible behaviour bound the FSD tightened or corrected that the delta specs' behaviour
     canon or design no longer matches — reconcile via the OWNING artifact (a behaviour change routes to
     the covering spec requirement / proposal slice, then re-run the scaffold; a rationale change updates
     the design).
2. **Decisions hardened downstream.** A choice the FSD/ISD effectively made concrete (a call-placement /
   mechanism the design left as a `Q#` or a proposed-direction `D#`) becomes a settled decision in the
   design with rationale / alternatives / trade-offs — not left as an open question the team has in fact
   answered.
3. **Scope / lock drift.** A scope or coverage fact the FSD/ISD settled that the proposal Requirements Lock
   / Delivery Breakdown or discovery Delivery Scope & Targets no longer matches — reconcile the proposal /
   discovery disposition. NEVER widen scope; only reflect what was signed.
4. **Open Questions / Planning Deferrals closure.** An upstream `Q#` the FSD/ISD resolved is recorded as
   decided in the owning artifact (not lingering as Open); a `PD#` the FSD/ISD consumed is reflected in its
   status.
5. **Cross-cutting constraints and RFC 2119 strength.** A constraint the FSD/ISD tightened (e.g. a MAY
   fallback that became the SHALL primary path) has its requirement strength updated in the design's Base
   Configuration & Cross-Cutting and decisions, and in the covering spec where it is behaviour-bearing.
6. **Re-derive the upstream derived blocks.** After the edits above, rebuild each touched upstream
   artifact's derived blocks — design / proposal / discovery `## Solution Overview` or `## Executive
   Summary` + `## Document Consumption Guide` (and, for an integration design, the Integration
   Specification `### Overview`) — from the CHANGED body, per the block re-derivation procedure in
   [derived-block-rederivation](mdc:.cursor/rules/workflow/openspec-workflow/references/derived-block-rederivation.md).

## Mechanism

Delegate this pass to a fresh `generalPurpose` subagent, exactly as the post-grooming consistency pass
does: the primary agent that authored the FSD/ISD carries "what I already wrote" bias and misses stale
upstream wordings a fresh reader catches. Prompt it with the settled FSD/ISD and the current upstream
artifacts, have it audit each upstream artifact against the six goals above, edit each `*.md` directly with
minimal `StrReplace` changes (no git), re-derive the touched artifacts' derived blocks, and return a
verdict: **CLEAN | CLEAN AFTER FIXES | NEEDS REVIEW**.

Do not treat an upstream artifact as approved-and-frozen once the FSD/ISD changes its contract: an FSD/ISD
that hardened the solution but left design / proposal / discovery / specs describing a superseded contract
is a defect a downstream reader (technical-design) will inherit. Present the actualized upstream for a human
re-approval when the pass reports CLEAN AFTER FIXES with material changes.
