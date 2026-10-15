# ADR template usage

When `sync` detects a new module without an associated ADR **and runs in interactive mode**, it offers to create a stub from [templates/new_adr.md](../templates/new_adr.md). The template is pre-filled with metadata — the user writes the content later.

**ADRs are always human-authored.** The skill never creates ADRs under `--yes` or in non-interactive environments (no TTY, CI, `/loop`, batch scripts). In those modes a missing ADR produces a `[WARNING]` log line and a summary entry — never a stub. Rationale belongs in a human's head; a blank "proposed" ADR in the repo is worse than no ADR at all.

## File naming

`.ai-memory/decisions/YYYY-MM-DD-<slug>.md` where:

- `YYYY-MM-DD` is today's date in UTC.
- `<slug>` is the module's `technical_name` with underscores converted to hyphens (kebab-case).

Example:
- Module: `acme_sale_approval`
- Today: 2026-04-17
- File: `.ai-memory/decisions/2026-04-17-acme-sale-approval.md`

If a file already exists at that path (unlikely but possible if the dev accepts twice on the same day), append `-2`, `-3`, etc. to the slug.

## Pre-filled metadata

The skill writes these fields into the frontmatter automatically:

| Field | Value |
|-------|-------|
| `date` | Today (UTC, `YYYY-MM-DD`) |
| `author` | `memory-check` |
| `status` | `proposed` (the dev changes to `accepted` once they fill the content) |
| `tags` | `[<technical_name>]` — the slug, unquoted |
| `supersedes` | `null` |

The title line is pre-filled as `# Decision: new module <technical_name>` — placeholder for the user to edit.

The body sections (`## Context`, `## Decision`, `## Alternatives considered`, `## Consequences`) are empty with a `[TODO]` marker. The user fills them.

## Template engine

Placeholders `{{date}}`, `{{technical_name}}`, `{{slug}}` are replaced with `str.replace()` — no Jinja, no templating engine. Keep placeholders simple: no conditionals, no loops. See `create_adr()` in `scripts/reconcile.py` for the exact render path.

## What the skill does NOT do

- Does NOT write the Context / Decision / Alternatives / Consequences body.
  The module's technical details are in the manifest; the **rationale** is only in the dev's head.
- Does NOT open an editor.
  The skill prints the path and lets the dev open it in their editor of choice.
- Does NOT block the `sync` action.
  If the dev declines the ADR prompt ("no"), the sync continues and the drift stays as `missing_adr` for the next run.
- Does NOT modify existing ADRs. Ever.
- Does NOT create ADRs under `--yes` or in non-interactive environments.
  Even if the operator wants "no prompts, just do it", the skill will only apply YAML changes automatically — ADR authoring always needs a human in the loop. Missing ADRs surface as warnings for later follow-up.

## Interactive prompt wording

When prompting, use Spanish (matches team language). Sample:

> El módulo `acme_sale_approval` no tiene ADR asociado. ¿Quieres crear uno ahora? [s/n]

If yes:

> Creado: `.ai-memory/decisions/2026-04-17-acme-sale-approval.md` (status: proposed).
> Abre el fichero cuando tengas un momento y rellena Context / Decision / Alternatives / Consequences.

If no:

> De acuerdo. El módulo quedará sin ADR hasta el próximo `memory-check`.

## Why proposed status

The ADR starts as `proposed` (not `accepted`) because the dev has NOT yet written the content. Marking it `accepted` upfront would let skeleton ADRs slip through review. The dev must change it to `accepted` when they finish writing — this is the explicit signal that the decision is documented.

Layer 2 (git-manager review / quality-gate) can optionally flag `proposed` ADRs older than 7 days as drift — but that belongs to the code-reviewer rules, not to this skill.
