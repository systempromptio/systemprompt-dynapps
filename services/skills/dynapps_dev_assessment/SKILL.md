---
name: dynapps-dev-assessment
description: >
  Evaluates Odoo development requests before any code is written. Determines whether the
  functionality already exists in standard Odoo, in an Dynapps Core module, or can be solved with
  Studio; only when no alternative exists does it plan custom development with a technical
  estimate. Triggers: "assess a development", "do we need development for this", "analyse a
  development request", "I need a module for X", any new-functionality request in Odoo, or a
  consultant wanting to push a change already prototyped in Studio to production.
version: 1.5.5
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion, WebSearch, WebFetch, Agent]
effort: medium
model: opus
---

# Dev Assessment — Odoo Development Request Evaluation

## Critical

1. **Output language follows the reader, not this file.** This skill is written in English; every artefact it produces (technical report, client validation document, chatter notes, tasks) is written in the **reader's** language — the client validation document in the **client's** language, which for a country deployment means that country's language. Default when unstated: the language of the request. Professional, clear, direct prose; an actionable report.
2. **Never continue without a confirmed Odoo version.** Version determines what "standard" means.
3. **"Config-only" is a hypothesis, not a verdict** for non-trivial stock / UoM / FSM / accounting / payment flows (GOTCHA 10).
4. **Read attachments before classifying** (STEP 0.2) — one can flip the verdict.

## Core Principle

Ladder **Standard > Core > Studio > Custom**, judged by **governed code**: for non-trivial logic, a reviewed and testable module outweighs ungovernable Studio. Doctrine: the
`ai-impact-studio-vs-custom.md` reference of the **odoo-classification-guide** skill.

1. **Standard Odoo** — native functionality in the client's version
2. **Dynapps Core** — an in-house module that already solves the case (Core Supremacy)
3. **Odoo Studio** — visual configuration, for the **trivial and low-risk only** (one label, one simple field, one view, a single-step automation). Studio has no quality gate.
4. **Custom development (module)** — for **non-trivial logic, validations, flows, data integrity**: with AI it is cheap and, above all, **governed** (reviewed, testable, migratable). In that band it beats a tangle of ungovernable Studio.

## Agent Routing

**sonnet** — gather requirements from the record and the user. **opus** — classification judgment
(Standard vs Core vs Studio vs Custom) and report synthesis.

## Quick Start

1. Read [references/workflow-steps.md](references/workflow-steps.md) and [references/escalation-and-modes.md](references/escalation-and-modes.md)
2. **STEP 0:** detect **mode A/B** + gather info (version, functionality, attachments) [STOP]
3. **STEP 1:** search standard Odoo (changelogs + LLM knowledge + web only on real doubt)
4. **STEP 2:** search Dynapps Core (JSON indexes)
5. **STEP 3:** classify Studio vs Custom [STOP — present the result]
6. **STEP 3.5: Studio governance check** (only with a live environment) — read [references/studio-governance-check.md](references/studio-governance-check.md): if the change lands on a zone an installed module or Studio already owns, force a module (governed) even when it "looked like" Studio.
7. **STEP 4:** technical approach (only if CUSTOM). **Scan module [SONNET SUBAGENT, optional]:** if fan-out is available, launch Agent(model: sonnet, description: "scan module") invoking `odoo-module-scanner` on {path} for an inventory of models, fields, views, security, controllers and JS. If fan-out is unavailable, run it inline and say so.
8. **STEP 5:** technical report (mandatory skeleton) → **STEP 6:** client validation (automatic when the verdict is CUSTOM). If CUSTOM, create the task via **dev-task-writer** with the complete **transfer package** (next section). It does NOT re-classify — that breaks the loop.

**At any step**, if an escalation-gate condition is met, the result is not a verdict but
`BLOCKED — needs a dev` (see `escalation-and-modes.md`).

## Prohibitions

- NEVER recommend development when **standard** or **Core** covers it
- **Studio is not a wildcard for avoiding a module**: it wins only for the trivial and low-risk. For non-trivial logic the governed module wins (`ai-impact`)
- NEVER suggest generic OCA or third-party modules. **EXCEPTION (country localizations):** fiscal localizations certified by local partners (e.g. Arxi for Portugal/Angola) ARE a valid route and beat custom development — they are not a "forbidden third party". Before proposing development for a country's tax compliance, check the attachments for an existing partner proposal and treat its cost as recurring + implementation, not as programming
- NEVER continue without a confirmed Odoo version
- NEVER use implementation jargon in the client validation document
- **NEVER declare "config-only / zero development" for non-trivial stock / UoM / FSM / accounting flows without validating the mechanism** against the target version's source (or a probe with rollback via `odoo-flow-tester`). A "standard/config" classification is a **hypothesis until executed** (AromaTrace docs 26/27: "pure config, zero Python" turned out unfeasible — UoM does not create packages, an orderpoint with Min=0 never triggers, `sale_line_id` on an asset is forbidden)

## Transfer package (mandatory when the verdict is CUSTOM)

What you read to classify, the developer also needs to implement. **Handing over the
classification is not enough.** When invoking `dev-task-writer`, pass it:

1. **The classification already resolved** — so it does NOT re-classify (GOTCHA 9).
2. **Source record**: model + ID + URL (`{base}/odoo/{model}/{id}`).
3. **Attachments**: `ir.attachment` with `id`, `name` and one line on what each contributes. Mark **DECISIVE** the one that changed the verdict: that one is **re-attached** to the task; the rest are linked.
4. **Code access** for the module to extend: the route that worked + repo + branch + version (STEP 4.0).
5. **The affected field zone** and which module owns it today, if STEP 3.5 ran.
6. **Exact identifiers** of what must be touched (xmlid, model, field). Without an id it is not verifiable.

**NEVER** close an assessment with a CUSTOM verdict without these 6 points. A developer who has
to come back and ask for the context is a failed assessment.

## GOTCHAS

1. **Standard Odoo changes every version** — a feature missing in v17 may exist in v18. Always check the target version's changelogs.
2. **Dynapps Core modules have hidden dependencies** — installing one may pull 5+. Check the dependency tree first.
3. **Studio for the trivial, not by default** — for non-trivial logic a governed module beats Studio (`ai-impact`), and Studio collisions must be checked (STEP 3.5).
4. **The client says "custom" but means "configuration"** — verify the actual gap before classifying as development.
5. **Effort estimates without risk contingency are wrong** — apply the 1.2–1.5x factor from `odoo-effort-estimator`.
6. **Attachments override the description** — a partner proposal in an attachment can turn "custom dev" into "partner cost". Read them via `index_content` before classifying (STEP 0.2).
7. **Odoo MCP efficiency** — resolve record + read attachments in ONE parallel batch; `index_content`, never binaries; grep changelogs and Core JSONs, never full `Read`. **`index_content` DISCARDS non-ASCII characters** (a perfectly valid UTF-8 CSV reads as `Preparaci n`): it serves to DECIDE, not to diagnose — any claim about encoding or accents is made against the binary. Post ONE consolidated chatter note (`message_post`, `subtype_xmlid: mail.mt_note`). **Chatter HTML handling varies by MCP version/instance** — re-read the stored `body` after posting: if it shows escaped `&lt;`/`&gt;`, repost via `post_message` or an `orm_write` on the message's Html field. Never assume HTML renders.
8. **"The whole model is touched" is NOT a collision** — in a customized client almost every model is touched by something. Collision means the **same field zone** (STEP 3.5).
9. **Loop with dev-task-writer.** It already calls dev-assessment to classify (its Step 1 gate). Hand it the classification you already made, or the two loop forever.
10. **"Config-only" is a hypothesis, not a conclusion.** Native stock / UoM / FSM / payment behaviour has silent traps (UoM ≠ packages, orderpoints trigger on strict qty<min, `package.type` without volume, `invoice_policy='delivery'`). Non-trivial flow + standard verdict → demand a probe with rollback before closing. Peripheral gotchas right ≠ central mechanism right (case B-04).
11. **`gh repo clone` goes over HTTPS with the `gh` token** and works without an SSH key. What DOES fail without a key is `git clone`/`push` against a `git@github.com:` remote (route 3): there the permission error never mentions SSH and misleads. Without `gh auth status` OK, use the ZIP route.
12. **Reason in English, deliver in the reader's language.** English reference material must not leak into a client's validation document.

For shared ecosystem patterns, see `common:references/shared/`.

## References (read on demand)

| Ref | When to read |
|-----|-------------|
| [references/workflow-steps.md](references/workflow-steps.md) | **Always** — steps 0–4: info, standard, core, studio/custom, technical approach |
| [references/escalation-and-modes.md](references/escalation-and-modes.md) | **Always** — mode A (new request) vs mode B (promotion) + the escalation gate to a dev |
| [references/studio-governance-check.md](references/studio-governance-check.md) | **When a live client environment exists** — STEP 3.5: Studio collision radar + staging→prod promotion mode |
| [references/report-and-validation.md](references/report-and-validation.md) | **Always** — STEP 5 (report) + STEP 6 (client validation). The template and writing rules are **MANDATORY**: do not improvise the report |
| [references/output-standard/README.md](references/output-standard/README.md) | When in doubt about the FORM of the report — the two deliverables that define the standard (security audit, two branches) |
| `templates/client_validation_spec.md` | When writing the client validation document |
| **odoo-classification-guide** skill | Always — golden rules, decision tree, Studio capabilities, complexity thresholds. Read its `ai-impact-studio-vs-custom.md`: governed code > Studio for non-trivial logic |
| **content-collection** skill | Always — changelogs and Core module indexes (123 + 8 + 15) |

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-classification-guide | common | Classify Standard/Studio/Core/Custom (incl. the AI-impact doctrine) |
| odoo-effort-estimator | common | Estimate development hours |
| odoo-module-scanner | common | Scan an existing module before extending it |
| content-collection | common | Odoo changelogs and Dynapps Core module indexes |
| odoo-flow-tester | common | Probe a "config-only" hypothesis with rollback before closing it |
| dev-task-writer | common | Create the task when the verdict is a module, **with the full transfer package** (it must not re-classify → breaks the dev-assessment ↔ dev-task-writer loop) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-creator | developer | When the assessment decides custom dev is needed |
| odoo-studio-migration | developer | When the assessment recommends Studio over custom |
| dev-task-writer | common | Classify a request Standard/Studio/Custom before drafting the task (Step 1 gate) |
| odoo-context-briefing | common | Classification step inside the project briefing |
| project-analysis | consultant | Classify each GAP (anti-over-customization gate) |
