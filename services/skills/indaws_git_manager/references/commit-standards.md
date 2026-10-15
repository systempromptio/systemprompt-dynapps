# Commit Standards — Conventional Commits

[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) is a lightweight convention on top of commit messages. Each message declares the **intent** of the change (`feat`, `fix`, `refactor`, ...) so that humans and tooling can understand the history at a glance. The exact format, allowed types, and validation regex are defined in `.gitworkflow.yml` — this document covers the **judgment** the skill needs to apply them well.

## Two levels of strictness

Squash-merge workflows create two contexts with different stakes:

- **MR/PR titles** — become the permanent squash commit on the target branch. They must be precise, traceable, and follow the strict format from the config. This is the project's long-term history.
- **Branch commits** — get squashed away when the MR/PR merges. They only need to be useful during development and code review, so a lightweight format is enough. No scope or task ID required because the branch name already carries that context.

The principle: **permanent history = strict; throwaway history = lightweight.**

## Choosing the right type

The list of valid types comes from the config. What the config cannot provide is the **semantic judgment** for picking the correct one:

### feat vs fix vs refactor

| Situation | Type | Reasoning |
|-----------|------|-----------|
| Something was **broken** and now it **works correctly** | `fix` | Correcting a defect |
| Something **didn't exist** and you **added it** | `feat` | New capability |
| Something **worked** but you made it **work differently** (user-visible) | `feat` | Changed behavior = new feature |
| Something **worked** and you restructured it **without changing behavior** | `refactor` | Internal-only change |
| You improved **speed** without changing what it does | `perf` | Special case of refactor with measurable impact |

### Other common decisions

- `test` — adding or fixing tests, even if they accompany a `feat` or `fix` (use `feat`/`fix` for the MR title and `test` for the individual commit)
- `style` — formatting, whitespace, linting — zero logic change
- `docs` — documentation only (comments, READMEs, knowledge articles)
- `migrate` — version migration work (e.g., v18 to v19 API changes). Use when the primary intent is porting, not adding features
- `data` — initial data, demo data, XML/CSV data files
- `i18n` — translation files only
- `wip` — work in progress, **branch commits only** — never in MR/PR titles
- `chore`, `build`, `ci` — maintenance, dependencies, CI/CD pipeline — no business logic change

## Breaking changes

When a change breaks backward compatibility, mark it with `!` after the type/scope. This signals to reviewers and tooling that downstream code may need adapting.

In an **Odoo context**, "breaking" typically means:

- **Renaming a model or field** that other modules inherit or reference
- **Changing an XML ID** used in other modules' data files or views
- **Removing or changing a method signature** that other modules override
- **Changing a view's XML structure** that other modules target via `xpath`
- **Renaming a JS component or template** that other modules extend

Examples:
```
feat(sale_custom)!: rename x_studio_margin to x_margin_percent [P42-4550]
refactor(api)!: change partner endpoint response format [P42-4600]
```

When in doubt, mark it as breaking — a false positive is far less costly than a missed breaking change.

## Mapping from Odoo SA / OCA style

Teams coming from the traditional Odoo prefix style can use this reference:

| Odoo SA / OCA | Conventional Commits | Notes |
|---------------|---------------------|-------|
| `[FIX]` | `fix` | Direct equivalent |
| `[IMP]` | `feat` or `style` | IMP is ambiguous — split by intent: new behavior = `feat`, cosmetic = `style` |
| `[ADD]` | `feat` | Direct equivalent |
| `[REF]` | `refactor` | Direct equivalent |
| `[REM]` | `refactor` or `chore` | Removing dead code = `refactor`; removing files/tooling = `chore` |
| `[PERF]` | `perf` | Direct equivalent |
