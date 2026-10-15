# Report Template & Client Validation

> **Language.** The skeleton and rules below are English. **Render the actual documents in the
> reader's language** — the client validation document in the **client's** language, which for a
> country deployment means that country's language. Translate the headings too; never deliver a
> half-English document. Professional, clear, direct prose; an actionable report.

## How the report is written

The **output standard** is the two (anonymized) security analyses in `output-standard/`. On any
question of form, imitate those.

### The 12 rules of the standard

1. **The title asserts something, it does not label the document.** "Field360 does exist here:
   what that changes in the implementation" — not "Analysis of the staging branch".
2. **Open with reality as told, not with a description of the engagement.** Measured figures
   first: "16 internal users, and all of them are administrators of almost every app".
3. **Every figure is a measurement, not an estimate.** If you have not counted it, you do not
   write it.
4. **A "declared versus actual" table**: what the request/matrix says against what the instance
   is, point by point.
5. **Say explicitly what is fine.** "Worth saying: the native multi-company rules are intact." A
   report that only alarms does not get read twice.
6. **Numbered findings (H1..Hn), with severity and the exact technical identifier.**
   `hr_timesheet.timesheet_line_rule_portal_user` is set to `active = false` — like that, not
   "some rules are disabled".
7. **Decouple the urgent from the project.** "H1 and H2 can be fixed even if everything else is
   postponed."
8. **State what each finding is NOT**, if it could be confused. "It is not a leak towards the
   portal, but it voids any per-site scope for internal users."
9. **A section of its own for what is NOT solved by this route**, with the why and what it would
   take.
10. **Explain in prose the expensive part and why it is expensive.** "Group record rules are
    unioned, not intersected" — the technical reason that drives the cost, in understandable
    language.
11. **Deviations declared, never hidden.** "Noted as a conscious deviation."
12. **Close with the decisions that are not yours to make, marking the blocking ones.** "Not a
    line of code gets written until these five are answered."

### Forbidden

- **Preamble and closing.** No "This report analyses..." and no "In conclusion, it is
  recommended...". The verdict is a datum, not a farewell.
- **Adjectives instead of figures.** "Quite a few users" does not count; "16 internal users"
  does.
- **Coverage without a verb.** "Might be covered", "it seems to exist", "in principle": either it
  is covered and you say where, or it is not.
- **Generic risk.** "Performance risk" does not count. "The viewer reads `field.issue` with a
  service user; restricting by site may leave it without data" does.
- **Alarm without bounding it** (see rule 8).
- **A paragraph longer than 4 lines.** One idea per sentence.

### Mandatory

- **The exact technical identifier** (module, model, field, xmlid, domain) WHENEVER something is
  claimed about the instance. Without an id, the claim is not verifiable.
- **The consequence behind the technical datum**, in one business sentence. The id says where to
  look; the consequence says why it matters.
- **What falls out of scope, with the reason**, in its own section.

> **This is a brief, not a document.** The whole report fits ~30-40 lines and is read top to
> bottom in under a minute. The verdict and what blocks it come first; everything else exists
> only to make them verifiable. A section that adds nothing to the verdict is dropped. The two
> long `output-standard/` documents are a *security audit* — imitate their style, not their length.

---

## STEP 5: Technical Report

Each slot declares **what kind of datum** it admits. A slot that does not demand a datum is
filled with prose.

```markdown
# {Title that asserts the conclusion}
**Verdict**: {STANDARD / CORE / STUDIO / CUSTOM / BLOCKED}. {decisive reason, one sentence.}

**Context**: {environment · Odoo version · read-only / dump / XML-RPC · date}. *Proposal for review, nothing modified.*

## What blocks
{1-3 bullets, one line each: the decisions or findings that gate everything. If none: "Nothing".}

## Declared versus actual {only if an instance was audited}
{Table, max 5 rows: Point | Request says | Instance is}

## Findings {only if audited, max 4}
{H1..Hn · severity · exact id · consequence. Two lines each. Drop "what it is NOT" unless
 genuinely ambiguous.}

## Why not the alternatives
- Standard: {covered / not — module + version, one line}
- Core: {module found, or "none", one line}
- Studio vs module: {the decisive criterion + STEP 3.5 result, one line}

## Execution {only if CUSTOM}
{3-4 gates, one line each: the command whose exit code decides progression.}

## To decide
{1-3 bullets, one line each. Mark BLOCKING. Business decisions are declared, not taken.}
```

If the verdict = CUSTOM → continue AUTOMATICALLY to STEP 6.

---

## STEP 6: Client Validation Document

**Runs AUTOMATICALLY when the verdict is CUSTOM.** Do not ask.
Template: `templates/client_validation_spec.md`.

### Writing rules

- **Technical datum + consequence, always as a pair.** The exact identifier is written down: it
  is what makes the claim verifiable. Behind it goes one business sentence saying why it matters.
  > Example from the standard: *"`hr_timesheet.timesheet_line_rule_portal_user` is set to
  > `active = false`. It is the one that limits portal users on timesheets. With 22 workers on
  > the portal, the exposure is real."*
- **Never the technical id alone** (the client does not know what to do with it) **nor the
  consequence alone** (nobody can check it). Both, or neither.
- **Examples with the client's real data**, never generic ones.
- **Implementation jargon out** — "transient", "inheritance", "prefetch" carry no consequence.
  But the NAME of what is being touched is always written.

### Delivery
Markdown by default. The consultant reviews it before sending it to the client.
For a deliverable with corporate formatting, delegate to `dynapps-brand` + `pdf-renderer`.

---

## Strict Rules

### General
- ALWAYS a confirmed Odoo version before evaluating
- NEVER recommend development if standard or Core covers it. Studio only for the trivial; for
  non-trivial logic the governed module is preferable (single criterion in
  **odoo-classification-guide**)
- NEVER suggest generic OCA or third-party modules (exception: country fiscal localizations)
- ALWAYS consult the Indaws Core indexes before recommending development
- ALWAYS apply **odoo-classification-guide** (Golden Rules first)
- Written in the reader's language (SKILL.md `## Critical`), professional, clear and direct

### Searches
- Changelogs + Core: ALWAYS consult (local files, zero cost)
- WebSearch: only on genuine doubt
- Modules with `auto_install: true` install automatically

### Estimates
- Indicative, not commitments. A min–max range, never a fixed number
- Include testing (20–30%). For complex work: `odoo-module-creator` Plan Mode

### Studio
- Studio capabilities: delegate to `odoo-classification-guide`
- NOT available in Community
- A well-written module with tests > complex Studio configuration

### Content-Collection
- Data lives in the `content-collection` skill of the `common-skills@indawsodoo` plugin
- If not found, say so and continue with LLM knowledge
