<!-- One file per affected capability, at `specs/<capability>/spec.md`; a real file carries only
     its own delta section (ADDED / MODIFIED / REMOVED), never all three shapes shown below.
     A capability file carries MULTIPLE requirements — a WBS's criteria are partitioned by the
     SUBJECT that owes them: the shopper-outcome seat, each applicable cross-cutting guarantee,
     and (last) a single `Management` requirement for the administering business user's criteria.
     Each requirement carries its FULL projected-downstream scenario set: every near-final proposal
     AC criterion covered by >=1 scenario (main flow + the grounded edge/error/config cases), not a
     lone happy path. The downstream FSD projects ALL of these scenarios and labels each
     requirement's rows with that requirement's NAME. The two ADDED requirements below model that
     shape; a real file repeats it per requirement. -->

## ADDED Requirements

### Requirement: <!-- WBS — behaviour name, e.g. `CAP.AREA — <behaviour name>`; the WBS code is the FSD join key and lives IN the heading -->

<!-- Trace: <RQ IDs from discovery/proposal, e.g. RQ-701> · Source: <SRD/notes/grooming-decision/baseline citation> -->

<!-- requirement text: observable behaviour in RFC 2119 language, carrying every
     approved-decision bound that shapes it (a config bound, a v1 scope limit, a
     permanent redirect) as an observable clause -->

#### Scenario: <!-- main flow; the whole heading text is the scenario name (it carries no marker, label, or other interpreted prefix) -->
- **GIVEN** <!-- precondition -->
- **WHEN** <!-- observable trigger -->
- **THEN** <!-- expected observable outcome -->

#### Scenario: <!-- a grounded EDGE case: empty result, cleared filter, boundary input -->
- **GIVEN** <!-- precondition -->
- **WHEN** <!-- observable trigger -->
- **THEN** <!-- expected observable outcome -->

#### Scenario: <!-- an error / denied case a seed names: permission denied, unavailable dependency, unresolvable input -->
- **GIVEN** <!-- precondition -->
- **WHEN** <!-- observable trigger -->
- **THEN** <!-- expected observable outcome -->
- **AND** <!-- additional assertion when the outcome has two observable parts -->

#### Scenario: <!-- a config-governed case: the shopper outcome follows an administered bound (the subject is still the shopper, so it stays here) -->
- **GIVEN** <!-- an administrator has set the bound in Business Manager -->
- **WHEN** <!-- the shopper exercises the behaviour -->
- **THEN** <!-- the observable outcome follows the configured bound -->

### Requirement: <!-- `<same WBS as above> — Management` when that WBS has criteria whose SUBJECT is the administering business user: they NEVER sit in the requirement above, they form this ONE trailing requirement. Otherwise this slot is the NEXT behaviour, or an Applicable cross-cutting guarantee as its own requirement. Several requirements MAY share one WBS (aggregated into one FSD block downstream, each labelled by its name). -->

<!-- Trace: <RQ IDs (a Management or cross-cutting requirement inherits the RQ of the behaviour it serves)> · Source: <citation> -->

<!-- its own requirement text — only the clauses its own criteria state (a Management requirement states what the business user SHALL be able to configure, curate, or enable) -->

#### Scenario: <!-- its main flow (a Management scenario: WHEN the business user sets the option, THEN the storefront behaviour follows it) -->
- **GIVEN** <!-- precondition -->
- **WHEN** <!-- observable trigger -->
- **THEN** <!-- expected observable outcome -->

<!-- ...plus its own grounded edge/error/config scenarios, same as above... -->

<!-- RENAMED (rare): only when a baseline requirement's NAME changes (same requirement,
     same WBS, new name — the WBS is the immutable join key and NEVER changes on a rename).
     Emit a `## RENAMED Requirements` section with a FROM/TO pair —
     `- FROM: \`### Requirement: <old heading, VERBATIM incl. its WBS>\`` then
     `- TO: \`### Requirement: <WBS> — <new name>\`` — never a live section unless a rename
     actually happens. No Trace/Source/Scenario on a rename pair; if behaviour also changes,
     add a MODIFIED block under the NEW heading. -->

## MODIFIED Requirements

### Requirement: <!-- baseline requirement heading, VERBATIM incl. its WBS (archive merge key) -->

<!-- Trace: <RQ IDs> · Source: <citation incl. the baseline spec path> -->

<!-- FULL updated requirement block copied from the baseline spec, then edited
     to the new behaviour — not a diff note. -->

#### Scenario: <!-- updated scenario -->
- **GIVEN** <!-- precondition, if needed -->
- **WHEN** <!-- condition -->
- **THEN** <!-- expected outcome -->

## REMOVED Requirements

### Requirement: <!-- baseline requirement heading, VERBATIM (incl. its WBS if the baseline had one) -->

**Reason:** <!-- why it is retired and what supersedes it -->
**Migration:** <!-- where the behaviour/data went: redirect, re-pointed entry, replacing capability -->

<!--
  Shape hints:
  - One spec file per affected capability; the capability's disposition from discovery picks the
    delta section (ADD -> ADDED, MODIFY -> MODIFIED, REMOVE -> REMOVED). A real file contains only
    its own delta section(s), not all three shapes above.
  - Requirement heading = Option-H: `### Requirement: <WBS> — <behaviour name>`. The WBS code is
    the single FSD join key and lives in the heading. ADDED requirements always carry it;
    MODIFIED/REMOVED headings match the baseline VERBATIM (a purely legacy requirement that never
    had a WBS keeps its behaviour-name-only heading). Several requirements may share one WBS, and
    the behaviour name must then be unique within that WBS — downstream it becomes the label of
    that requirement's group of rows in the FSD/ISD Key behaviours table.
  - Metadata = a single HTML comment right after the heading:
    `<!-- Trace: <RQ IDs> · Source: <citation> -->`. Never bold `**Trace/WBS/Source**` lines —
    any text between the heading and the first scenario is swallowed into the archived baseline
    requirement text; a comment stays invisible in the render and out of the baseline canon.
  - Requirement text uses SHALL/MUST/SHOULD/MAY and describes observable behaviour only — no
    implementation tokens (paths, components, routes, endpoints, payloads), no consumed-vendor
    names, no invented URLs or magnitudes.
  - Scenario heading = `#### Scenario: <name>` (exactly 4 hashtags). The whole heading text is the
    name: it carries NO leading label, group tag, or re-usability marker. Grouping is expressed by
    the requirement a scenario sits under — the downstream FSD/ISD renders one table per WBS block
    with a sub-header row per requirement name (flat when the block has a single requirement).
  - Every ADDED/MODIFIED requirement has the Trace/Source comment and its FULL projected scenario
    set: every near-final proposal AC criterion covered by >=1 scenario (main flow plus the
    grounded edge/error/config cases). REMOVED blocks carry Reason + Migration instead, with no
    metadata or scenarios.
-->
