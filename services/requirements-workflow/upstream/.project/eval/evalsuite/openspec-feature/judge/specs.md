---
schema: openspec-feature
artifact: specs
context:
  - .context/initial-context
  - discovery/final
  - design/final
  - proposal/final
  - grooming/final
  - specs/global-navigation/spec
  - specs/store-directory/spec
sources:
  - ../../../../templates/openspec/schemas/feature/templates/spec.md
scoring:
  pass_threshold: 0.7
---

# Specs Standard — Contract and Rubric

This is the canonical definition of good delta specifications (`specs/**/*.md`)
in the OpenSpec `feature` workflow. The judge uses it to score a candidate
specs bundle. It is intentionally explicit and self-contained: do not assume
the judge has any ambient project knowledge beyond this standard, the inlined
SOURCES, and the CONTEXT (the source packet, the upstream discovery / design /
proposal artifacts, the APPROVED grooming ledger, and the baseline capability
specs).

## What specs are (essence)

- Specs are the **behaviour canon** of the pipeline: the formal, testable
  contract of WHAT the system does after this change. They live as working
  copies at `openspec/changes/<name>/specs/<capability>/spec.md` — one file
  per affected capability — and they are the ONLY artifact that survives into
  the `openspec/specs/` archive when the change completes. Everything
  downstream (FSD/ISD, stories, tasks, tests) derives from them.
- Specs are **deltas against the baseline capability specs**: `## ADDED
  Requirements` for new behaviour, `## MODIFIED Requirements` for changed
  baseline requirements, `## REMOVED Requirements` for retired baseline
  requirements. The archive merges these blocks into the capability canon by
  requirement heading, so heading fidelity is a merge key, not cosmetics.
- Specs **formalize the approved offer**: the proposal's Delivery Breakdown
  slices (S1..Sn) with their acceptance-criteria seeds are the human-approved
  skeleton; specs turn each seed into precise requirements and deterministic
  scenarios and own the exhaustive edge/error-case canon the seeds only hint
  at.
- Specs are generated **only after the Grooming Approval Gate passed**: the
  grooming ledger in CONTEXT is APPROVED, and its decisions are LAW. A
  decision a named human approved is encoded as behaviour; a spec that
  contradicts an approved decision, or encodes an alternative the session
  rejected, is wrong.
- Specs describe **observable behaviour only** — primarily consumed by LLMs
  for planning and execution, but readable by a human QA. RFC 2119 keywords
  (SHALL/MUST/SHOULD/MAY) state the requirement; GIVEN/WHEN/THEN scenarios
  make it testable. Implementation (file paths, components, routes, hooks,
  endpoints, payload fields) lives in design/technical-design, never here.

## Invariants the candidate MUST satisfy

1. **File inventory matches the affected capabilities.** Exactly one spec
   file per capability row in THIS flow's discovery Affected Capabilities
   table, named `specs/<capability>/spec.md`. The capability's disposition
   determines the delta section: ADD -> `## ADDED Requirements`, MODIFY ->
   `## MODIFIED Requirements`, REMOVE -> `## REMOVED Requirements`. No file
   for capabilities the change does not own; no missing file.
2. **Coverage of the approved offer.** Every in-scope requirement of the
   proposal's Requirements Lock appears in at least one requirement's
   `Trace:`; every Delivery Breakdown slice's acceptance-criteria seeds are
   formalized by at least one requirement; every cross-cutting area marked
   Applicable in discovery (accessibility, SEO, consent/privacy, i18n) is
   captured as behaviour. Deferred and out-of-scope requirements produce NO
   behaviour.
3. **Approved decisions are law.** Every grooming decision that shapes
   behaviour is ENCODED: its bound appears in the behaviour text of the
   requirement covering the decided area (e.g. an approved "regular hours
   only" bounds the detail behaviour; an approved "permanent redirect" is a
   permanent redirect; an approved "server proxy" means the vendor key never
   appears browser-side). A covering requirement whose text carries no trace
   of its approved bound violates this invariant — encoding lives in the
   behaviour text; whether `Source:` also cites the grooming decision is
   metadata hygiene, a warning at most. Nothing contradicts an approved
   decision, encodes a rejected option, or re-opens a settled question
   (conditional pre-decision phrasing like "if the API exposes them" after
   the human already decided is re-opening).
4. **Delta discipline against the baseline.** A MODIFIED requirement's
   heading matches the baseline requirement heading VERBATIM (the archive
   merge key) and carries the FULL updated requirement block (text +
   scenarios), not a diff note. A REMOVED requirement names a baseline
   requirement verbatim and carries **Reason:** and **Migration:** (where
   the behaviour went), with no scenarios required. ADDED is only for
   requirements with no baseline counterpart.
5. **Behaviour-only contract, inside the obligation boundary.** Requirement
   text uses RFC 2119 keywords and states observable behaviour. No
   implementation: no file paths, component or hook names, route
   implementations, loaders, helper names, endpoint URLs, payload fields,
   SDK initialization narratives, or task prose. Service contracts
   (endpoints, data mapping, payload shapes) live in the design's
   Integration Specification — an integration spec states the capability's
   observable obligations at its interface, not the wire format or vendor
   SDK mechanics. Each spec stays on ITS side of a shared requirement: an
   integration capability never specs the consumer's UI experience (how the
   consumer presents, synchronizes, or displays results), and a functional
   spec never names the vendor or service behind a consumed dependency
   ("address suggestions appear", not "suggestions sourced from the
   Addressly service") — only an integration capability's OWN vendor may be
   named, at its interface boundary.
6. **Deterministic, grounded scenarios (full projected set).** Every
   ADDED/MODIFIED requirement has at least one `#### Scenario:` in
   GIVEN/WHEN/THEN(/AND) form, specific enough to test (one observable
   condition or outcome per line, a concrete subject, no subjective
   wording), covering the main flow plus grounded
   edge/error cases (permission denied, empty results, unavailable
   dependency). Because the downstream FSD projects ALL scenarios (no
   curation), the candidate authors the COMPLETE grounded set: every near-final
   proposal acceptance-criteria criterion is covered by >=1 scenario
   (`specs_from_ac`). Scenario COUNT stays open-world — more or fewer than the
   golden is never itself a finding, only a criterion left with no covering
   scenario is. The scenario heading is `#### Scenario: <name>` — the whole text
   after the colon is the name, carrying no bracketed label, grouping tag, or
   re-usability marker; grouping is carried by the REQUIREMENT a scenario sits
   under, whose name becomes that group's label in the projected FSD/ISD table
   (see the requirement-partition law). No vague outcomes ("works correctly"), no invented
   magnitudes: a threshold, radius, distance, time, count, or limit
   appears only when a cited source or approved decision states it.
7. **Trace metadata (Option-H heading + comment).** Every ADDED/MODIFIED
   requirement carries its WBS code IN the heading — `### Requirement: <WBS> —
   <name>` (the FSD join key) — and, on the line immediately after the heading,
   a single HTML comment holding Trace + Source:
   `<!-- Trace: <RQ IDs> · Source: <citation> -->`. RQ and WBS codes are
   VERBATIM from the upstream trace, and Source cites grounding that exists in
   CONTEXT (SRD, technical/meeting notes, grooming decisions, baseline specs).
   The WBS belongs in the heading; requiring a bold `**WBS:**`/`**Trace:**`
   line, or flagging the WBS-in-heading as a code-in-heading defect, is a false
   finding. Several requirements MAY share one WBS (aggregated into one FSD
   block downstream) — a repeated WBS across requirements is never a finding. A
   cross-cutting requirement (privacy, accessibility, i18n) inherits the codes
   of the traced requirement(s) it constrains — a minted code (`PRIV`,
   `CC.PRIV`) or an `N/A` code is a `trace_metadata` critical.
8. **No fabrication.** No invented requirements, behaviour, codes, sources,
   capabilities, or decisions. Behaviour absent from the sources, seeds, and
   approved decisions does not appear, no matter how plausible.

## Forbidden (each is a critical finding when present)

- A missing or extra capability file, or a capability under the wrong delta
  section for its disposition (e.g. a REMOVE capability with ADDED
  behaviour).
- A dropped in-scope lock requirement (in no `Trace:`), an unformalized
  slice, or an Applicable cross-cutting area with no behaviour.
- Behaviour for a deferred requirement (signed later-phase), a declined or
  pending CR, or an out-of-scope requirement owned by another change.
- A contradiction of an approved grooming decision, an encoded rejected
  option, a covering requirement missing its approved bound, or behaviour
  built on an option the session left undecided (none exist here: the
  CONTEXT grooming ledger is fully approved).
- A MODIFIED heading that matches no baseline requirement, a MODIFIED block
  that is a diff note instead of the full updated block, or a REMOVED
  requirement without **Reason:** and **Migration:**.
- Implementation leakage: file paths, components, hooks, routes-as-code,
  helper names, endpoint URLs, payload field mappings, or vendor SDK
  mechanics in requirement text or scenarios.
- An obligation-boundary breach: an integration spec authoring the
  consumer's UI behaviour (map/list synchronization, display rules), or a
  functional spec naming a consumed dependency's vendor in behaviour text.
- An invented magnitude (radius, distance, latency, character count, limit)
  or an invented URL/route no cited source, approved decision, or upstream
  artifact (e.g. a design IA path decision) states.
- A fabricated RQ/WBS code (including minted cross-cutting codes and `N/A`
  cells), source citation, capability, or requirement.

## Judging method (apply before scoring)

Judge each claim by what is actually wrong with it, and record it in ONE
place.

- Artifact presence (run FIRST): if the candidate is not a specs bundle at
  all — a refusal or STOP message, a status report, planning notes, or an
  empty document — record the critical finding `no_artifact`, score EVERY
  dimension 0.0, and stop. A missing-upstream or unapproved-grooming
  complaint is never a valid candidate: the upstream artifacts and the
  APPROVED grooming ledger are given in CONTEXT.
- GOLDEN is a non-exhaustive positive EXAMPLE, not an answer key. A
  difference from GOLDEN — different requirement grouping, more or fewer
  scenarios, different wording — is NEVER by itself a finding. Every finding
  MUST quote the candidate text AND the STANDARD rule or CONTEXT fact it
  violates.
- Closed-world vs open-world. The capability set, delta sections, RQ/WBS
  codes, lock dispositions, slice set, approved decisions, and baseline
  requirement headings are CLOSED-world: they must trace to CONTEXT.
  Requirement naming, requirement grouping, scenario phrasing, and scenario
  count are OPEN-world synthesis: judge them for grounded, testable
  precision, not for matching the golden.
- File inventory law (closed rule — run BEFORE scoring `file_inventory`):
  list THIS flow's discovery Affected Capabilities rows with dispositions.
  Diff the candidate's file markers against that list. A missing capability
  file or a wrong delta section for a disposition is a critical; a file for
  a capability the flow does not own is a critical (`scope`), unless it is
  the golden's own routing of a new requirement (see allowed patterns).
- Coverage law (closed rule — run BEFORE scoring `requirement_coverage`):
  list the proposal lock's in-scope RQ set and the slice set S1..Sn. Every
  in-scope RQ must appear in >=1 `Trace:`; every slice's seeds must be
  formalized by >=1 requirement. HOW requirements group RQs or slices is
  free: one requirement may serve two slices, one slice may split into
  several requirements. Do not demand one-requirement-per-slice or flag
  grouping choices. A cross-cutting area counts as CAPTURED when its
  bounds appear in ANY requirement's text or in a dedicated scenario —
  a standalone requirement is never required for it; firing "not
  formalized as a standalone requirement" when the bound is present in
  requirement text is a false finding.
- Requirement-partition law (run BEFORE scoring `structure_conformance`): for
  each WBS the candidate uses, list its requirements in order and check THREE
  things, nothing more. (i) No requirement mixes an ADMINISTERING-business-user
  criterion ("Business users SHALL be able to configure / enable / curate …")
  with a shopper-outcome criterion: a WBS's administering criteria sit in ONE
  trailing requirement, canonically named `Management`. (ii) The requirement
  names within that WBS are DISTINCT — each name becomes a group label in the
  projected FSD/ISD table. (iii) When one capability's observer seat is split
  into several requirements on the same WBS, each keeps >=2 scenarios (a
  `Management` or cross-cutting requirement is exempt, and requirements the WBS
  collects from DIFFERENT capabilities are separate seats, never a split).
  A scenario that merely PRESUPPOSES an administered record (an admin GIVEN
  under a shopper-outcome requirement) is NOT a mixed subject. Everything else
  about grouping stays OPEN-world: how many requirements a WBS carries, how they
  are named, and any difference from GOLDEN's partition are never findings. A
  partition defect is a `structure_conformance` deduction and a WARNING — never a
  critical, and never re-filed as a coverage critical while every criterion is
  still covered by some requirement.
- Approved-decision law (closed rule — run BEFORE scoring
  `approved_decisions`): from the CONTEXT grooming ledger, list every
  APPROVED / DEFERRED-APPROVED Decision cell. For each, locate the candidate
  requirement(s) covering the decided area and check the behaviour TEXT
  carries the approved bound and nowhere contradicts it. Encoding is judged
  on the behaviour text; a `Source:` cell that does not cite the grooming
  decision is a warning, never the trigger for this law. A
  DEFERRED-APPROVED prerequisite (build proceeds against a stable interface)
  does NOT block or reduce spec behaviour — specs spec the full approved
  behaviour; flagging "dependency not delivered" against a spec is wrong.
- Implementation-leak test: name the offending token class (path, component,
  hook, endpoint URL, payload field, SDK mechanics). Approved-decision
  language is NOT a leak, in requirement text or scenarios: naming the
  approved MECHANISM as the decision's bound — "served server-side",
  "through the capability's server proxy", "the key never appears in
  browser-visible requests", "permanently redirected", "a domain-restricted
  key", "a lazy-loadable entrypoint", "billing on the client's account" —
  states the bound a human approved. A leak is: naming a vendor SDK or
  product as the implementation ("renders using the Cartora Maps SDK"), a
  scenario NARRATING internal steps ("the capability forwards the request
  to Cartora", "the entrypoint initializes the SDK", "returns a JSON
  payload containing latitude"), or endpoint/payload/internal-path tokens.
  A user-facing URL an upstream design fixed (the surface's path, a
  navigation or redirect target, e.g. `/store-locator`) is an observable
  IA outcome, NOT a path leak; the path leak is only an INTERNAL route
  module / loader / action of the build. The
  test: does the line state a bound or observable outcome (fine), or does
  it walk through internals a consumer cannot observe (leak)? A heading
  that names the capability's own vendor or an approved mechanism ("Cartora
  geocoding via server proxy") is style, not a leak — a warning at most;
  the leak test applies to requirement text and scenario lines.
- Redirect / cross-cutting placement: the coverage law requires each
  Applicable cross-cutting behaviour (e.g. the retired URL's permanent
  redirect) to be CAPTURED as behaviour: its bounds in some requirement's
  text or in a dedicated scenario, in any capability file. A REMOVED
  block's Migration note alone is NOT capture and IS a coverage critical;
  everything else about placement is the candidate's choice — never fire
  on which file carries it, and never demand a standalone requirement when
  the bound sits inside another requirement's text or scenario (this holds
  for the redirect exactly as for privacy).
- Minted routes are never downgraded: a literal path is allowed only when a
  cited source OR an upstream artifact in CONTEXT states it — the sourced
  retired `/stores`, and any user-facing URL the upstream design fixed as an
  information-architecture decision (the new surface's path, a navigation or
  redirect target; e.g. design deciding the locator is served at
  `/store-locator`). Before firing, scan design/discovery for the exact path:
  if an upstream artifact states it, the candidate inherits it and there is NO
  finding. A literal path NEITHER sourced nor stated by any upstream artifact —
  in requirement text, a scenario, or a Migration note — is a `no_fabrication`
  CRITICAL, never a warning: with no upstream path the named form ("the store
  locator page") is required, and minting a DIFFERENT slug the upstream does not
  carry is still critical. Likewise consumed-vendor naming in a functional
  spec's requirement text or scenario lines is ALWAYS a critical (in a
  heading or a Source cell it is a warning). Do not let an otherwise
  polished candidate soften these into warnings.
- One defect, one primary critical. Classify by the actual fault:
  - a missing/extra file or wrong section -> `file_inventory`;
  - a dropped RQ / slice / cross-cutting area -> `requirement_coverage`;
  - out-of-scope or deferred behaviour present, or an obligation-boundary
    breach (the other side of a shared requirement) ->
    `requirement_coverage` (scope side);
  - a contradicted / re-opened / rejected / unencoded decision ->
    `approved_decisions`;
  - a broken merge key, diff-note MODIFIED, or ABSENT Reason/Migration ->
    `delta_discipline`;
  - implementation tokens, mechanics narration, consumed-vendor naming, or
    vague/invented scenario content -> `behaviour_contract`;
  - a wrong, minted, or `N/A` RQ/WBS/source -> `no_fabrication` if invented
    out of thin air, `trace_metadata` if mis-copied or minted for a
    cross-cutting block.
  Do not raise the same underlying claim under several dimensions.
- Allowed patterns are never findings — under ANY dimension: (a) file
  markers with or without a flow-suffixed prefix (`specs.ba/...` in GOLDEN
  vs `specs/...` in the candidate) — a harness naming artifact; (b) the
  Trace/Source comment on MODIFIED blocks even though baselines carry none, a
  WBS in the heading, and several requirements sharing one WBS;
  (c) negative scope statements grounded in an approved decision ("holiday
  hours are not displayed in v1"); (d) scenario count and grouping choices
  that differ from GOLDEN while staying grounded — the requirement-partition
  law is the ONLY closed constraint on grouping, and it is warning-level;
  (e) routing a NEW
  requirement (no baseline counterpart) to the new surface's capability —
  e.g. the retired-URL redirect living with the locator capability rather
  than navigation — when discovery marks the source capability REMOVE and
  the behaviour survives on the new surface; (f) minor change-name slips
  between the fixture folder and the change identity in anchors — a warning
  at most; (g) citing grooming decisions (e.g. "grooming Q1") as a Source —
  approved decisions are legitimate grounding; (h) a cross-cutting
  behaviour (e.g. coordinate privacy) folded into the requirement it
  constrains — its SHALL NOT bounds in that requirement's text or a
  dedicated scenario — instead of the GOLDEN's standalone requirement:
  never demand the golden's grouping.
- Omission is not a critical — under ANY dimension — EXCEPT the closed laws:
  a missing capability file, a dropped in-scope RQ, an unformalized slice, a
  missing Applicable cross-cutting behaviour, a REMOVED block without
  Reason/Migration, and a covering requirement missing its approved bound
  ARE criticals, because inventory, coverage, decision encoding, and delta
  integrity are this artifact's core contract. Everything else missing is
  `missing_evidence` or a warning — including: a thinner edge-case set than
  the golden, a missing optional AND line, a `Source:` that cites one valid
  grounding but omits another the golden also cites, and a Reason/Migration
  that IS present and names where the behaviour went but is thinner than the
  golden's.
- Completeness does not offset a violation, and fluency is not correctness.
  Inspect each file, requirement, Trace, and scenario on its own merits
  regardless of how polished the bundle is.

## Scoring rubric

Score each dimension from 0.0 to 1.0 and put it in `rubric_scores` keyed by
the dimension id below. For each dimension, 1.0 = fully satisfied with
evidence, ~0.5 = partially satisfied or minor gaps, 0.0 = violated or absent.

- `file_inventory` — the closed inventory law holds: one file per affected
  capability, correct delta section per disposition, no missing or extra
  capabilities.
- `requirement_coverage` — the closed coverage law holds: every in-scope
  lock RQ traced, every slice formalized, every Applicable cross-cutting
  area captured as behaviour; no behaviour for deferred / declined /
  out-of-scope items and no obligation-boundary breach (the other side of a
  shared requirement).
- `approved_decisions` — every approved grooming decision that shapes
  behaviour has its bound in the covering requirement's text; nothing
  contradicts, re-opens, or swaps a decision for a rejected option.
- `behaviour_contract` — requirement text is RFC 2119 observable behaviour;
  scenarios are deterministic GIVEN/WHEN/THEN, specific enough to test,
  covering main flow plus grounded edge/error cases; no implementation
  tokens, mechanics narration, or consumed-vendor naming; no invented
  magnitudes or URLs.
- `delta_discipline` — MODIFIED headings match baseline verbatim with full
  updated blocks; REMOVED blocks carry Reason + Migration naming where the
  behaviour went; ADDED only for requirements without a baseline
  counterpart.
- `trace_metadata` — every ADDED/MODIFIED requirement carries its WBS in the
  Option-H heading and a `<!-- Trace: … · Source: … -->` comment after the
  heading, with verbatim codes and CONTEXT-grounded citations; the WBS in the
  heading is expected (not a code-in-heading defect) and a shared WBS is fine.
- `no_fabrication` — every requirement, behaviour, code, capability, source,
  and magnitude traces to CONTEXT or an approved decision; nothing invented.
- `structure_conformance` — Option-H `### Requirement: <WBS> — <name>` and
  `#### Scenario: <name>` shapes (exactly 3 and 4 hashes),
  bold GIVEN/WHEN/THEN lines, a `<!-- Trace: … · Source: … -->` comment after
  each ADDED/MODIFIED heading (no bold metadata lines), one `## <DELTA>
  Requirements` section per file matching the template, and the
  requirement-partition law's three checks (no mixed subject, distinct names
  within a WBS, no observer split below two scenarios).

### Overall score and pass

- Compute `score` as the mean of the rubric dimension scores, then cap it:
  if any item in `critical_findings` is present, `score` MUST NOT exceed
  0.5.
- `pass` is true only when `score >= threshold` (given in the user message)
  AND `critical_findings` is empty.

### Finding severity

- **critical_findings** — violations of any Forbidden item or invariants
  1-5 / 8 (inventory, coverage, decision, delta, leak, or fabrication
  defects). These block acceptance.
- **warnings** — quality issues that should be fixed but do not make the
  bundle wrong (e.g. a thin scenario, a borderline-vague outcome, a naming
  slip in an anchor, an awkward requirement grouping).
- **missing_evidence** — expected content the CONTEXT implies but the
  candidate omits without breaking a closed law (e.g. an edge case the
  seeds suggest, an AND assertion the source states). Record it ONLY here:
  it MUST NOT appear in `critical_findings` and MUST NOT cap the score.

Always ground findings in concrete evidence. Every critical finding MUST
quote a string that appears VERBATIM in the candidate. Before recording a
critical, locate that exact string (the marker line, the heading, the Trace
line, the scenario line) in the candidate text; if it is not literally
present, DROP the finding — do not infer, paraphrase, or assume a file or
requirement exists. For a missing-item finding, name the specific CONTEXT
fact (the capability row, the RQ id, the slice, the decision) the candidate
omits. A finding whose evidence you cannot copy verbatim from the candidate
is a hallucination, not a violation.
