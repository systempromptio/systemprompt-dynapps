# Generator Planning Pass (shared within schema)

This partial defines the planning pass the generator performs before it writes
the final artifact. It is injected into the generation system message for this
schema and is artifact-agnostic on purpose: artifact-specific section rules,
classification rules, severity labels, and quality bars live in the SCHEMA
ARTIFACT block.

Perform the planning pass as a compact checklist, not prose:

- Name the requested artifact and the OPENSPEC INSTRUCTIONS `<template>` sections
  that must be filled.
- List the CONTEXT source groups that contain ground truth for this run — such as
  initial requirements, meeting notes, tracker data, baseline specs, or upstream
  artifacts — naming each supplied block by its header verbatim. This list is the
  ONLY evidence of what exists for this change: a block that appears in it EXISTS,
  and an input is absent only when its block is missing or marked "NOT FOUND".
  Never record an artifact as missing, skipped, or "does not exist" while its
  block is present.
- Then RESOLVE the declared dependencies against that list as a CLOSED SET: take
  every id in the OPENSPEC INSTRUCTIONS `<dependencies>` block and write one line
  per id — `<id> -> supplied by "<verbatim CONTEXT block header>"` or `<id> -> NOT
  FOUND`. Resolve each id from the CONTEXT headers alone; a compile-time
  `status="missing"` stamp or unmet-dependencies warning describes the empty
  scaffold the instructions were compiled against, not this run. Only an id you
  resolved NOT FOUND is absent, and only an absent id can satisfy a stop condition
  or an "artifacts must exist" entry gate. When any later rule turns on whether an
  input exists, re-read these lines instead of recalling the answer.
- Extract the requirement-bearing identifiers and capability facts you will carry
  into the artifact. Keep identifiers verbatim.
- Where the OPENSPEC INSTRUCTIONS define pre-draft enumerations or ledgers of
  their own, build EVERY one of them here, in full, under the instructions' own
  names and resolution vocabulary. They are the same kind of binding commitment
  as the placement ledger below, so an instruction-defined ledger left unbuilt —
  or built by recalling a few members instead of walking the sources — is a
  failed pass, not a shortcut.
- Build a placement ledger before drafting: one line per extracted item as
  `item -> disposition -> single destination section -> owner/target`, using the
  OPENSPEC INSTRUCTIONS' own rules for dispositions, open questions, severities,
  gates, and handoff targets. The `owner/target` slot is REQUIRED whenever the
  destination's rows name an owner, capability, or target (e.g. an affected
  capability, a dependency owner, a publication path) — commit that value in the
  ledger, per the instructions' own assignment rules, instead of deferring it to
  emit time; omit the slot only for destinations without one. Assign each item
  EXACTLY ONE destination and do not repeat it in another section. When the
  instruction states an item type does NOT belong in a section, record its
  allowed home instead and keep it out of the forbidden one.
- For any section whose instruction defines a CLOSED membership (only listed item
  types belong and named others are barred), commit two explicit lines before
  drafting: `<SECTION> ROWS = [<verbatim members, or None>]` and
  `<SECTION> OUT = [<barred item -> its real home section>]`. This IS the binding
  row-set; do not defer the membership decision to emit time.
- Beyond WHICH rows belong, commit WHAT each row exports: for any committed item
  whose destination cell is a SET or carries qualifiers (a status, date, owner,
  blocking slice, approval state, or a projected sub-set of another section), write
  `<row> KEY DATA = [<every member, each with its qualifier, verbatim from the
  source>]`. Walk the source cell to enumerate it; never recall a few members. This
  binds exactly like the ROWS line: the emitted cell is a VERBATIM COPY of this set,
  so dropping a member or stripping a member's qualifier at emit time is a placement
  error — restore it. A single-value cell commits its one value the same way.
- State any unavailable live dependencies and how their LIVE-REFERENCE
  SUBSTITUTIONS are being applied.
- Treat the ledger as binding, not advisory: the final artifact is a VERBATIM COPY
  of it, never a re-derivation from CONTEXT. A closed-membership section emits
  EXACTLY its `ROWS` set — add no row at emit time — and nothing on its `OUT` line
  may appear anywhere in that section. As the LAST step before output, diff every
  emitted row against `ROWS`/`OUT` AND every emitted set-cell's members against its
  `KEY DATA` line: a row with no ledger line, or one the ledger assigned elsewhere
  (or listed in `OUT`), is a placement error — delete it; a cell that drops a member
  or a member's qualifier its `KEY DATA` line commits is a placement error — restore
  it; do not keep an unplanned row or annotate it "for completeness" / "for
  transparency".
- End with one self-check line confirming the reconciliation is done and that the
  final artifact will preserve source identifiers, avoid fabrication, follow the
  OPENSPEC INSTRUCTIONS `<template>` order, and output only Markdown after the
  plan block closes.

Do not solve artifact-specific semantics in this partial. Use it only to commit
to extraction, placement, substitution handling, and no-fabrication before the
artifact is written.
