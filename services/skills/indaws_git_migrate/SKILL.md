---
name: indaws-git-migrate
description: >-
  Adopta el workflow Indaws en un repo legacy: genera .gitworkflow.yml,
  .ai-memory/, configs de linting (pre-commit/ruff/pylint/editorconfig) y
  mergea .gitignore en 1-2 commits sin push. Sister of /git-new-repo (repo nuevo).
  Triggers: "migra el repo", "adopta el workflow nuevo", "bootstrap the workflow",
  "bootstrapea el repo legacy", "aplica el skeleton a este repo",
  "apply skeleton to this repo", "configura el repo con el workflow nuevo".
version: 1.0.3
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# /git-migrate — Bootstrap legacy repo into the new workflow

> **Canonical flow**: see `git-manager/SKILL.md` action `migrate`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Shortcut for `git-manager` action `migrate`. One-shot adoption of the Indaws workflow on a legacy repo (typically GitHub, pre-workflow). Generates `.gitworkflow.yml`, `.ai-memory/`, linting configs, merges `.gitignore`, and — if you accept — applies pre-commit to the full codebase in a **separate commit** so the infra commit stays clean and reviewable.

Creates 1-2 commits but **does not push**. You review before pushing.

## Usage

```
/git-migrate
```

No arguments. The action detects everything it needs from the repo: provider, branches, Odoo version, module layout, CI presence.

## What it does (summary)

1. **Preflight**: must be a git repo with a clean working tree. If `.ai-memory/` already exists, offers incremental mode (only copies what's missing).
2. **Detects** provider, branches, Odoo version, module layout, CI.
3. **Shows a summary** and proposes the naming dual when applicable (e.g. legacy GitHub `18.0` → propose creating `18.0-develop`).
4. **Asks 4 questions** (one at a time): create integration branch? copy CODEOWNERS? copy .coveragerc? apply pre-commit to all code now?
5. **Executes** the legacy migrate script + post-processes `.gitworkflow.yml` to reflect the real context (branches, `require_pipeline`, `modules.path`).
6. **Creates 1-2 commits** (infra always, pre-commit reformat optional). Never pushes.
7. **Reports** with next steps: review commits, fill CONTEXT.md, run `/memory-check sync`, then push.

## When to use

- Legacy GitHub customer repos that don't follow the new workflow yet.
- Repos with an outdated `.gitworkflow.yml` (pre-v1 schema or missing `modules:` section).
- Repos that have never had linting configs and you want to adopt the full toolkit in one shot.

## When NOT to use

- **Fresh Copier-generated repos**: they already have everything. Run `/memory-check sync` instead.
- **Partially migrated repos with conflicting state**: the action will detect `.ai-memory/` and offer incremental mode, but if the `.gitworkflow.yml` is in a weird state, prefer to fix it manually first.
- **Dirty working tree**: commit or stash your changes first. `/git-migrate` refuses to proceed with uncommitted changes.
- **CI pipelines / non-interactive shells**: this is a human-supervised action.

## Relation to the `init` action

Both `migrate` (this) and `init` coexist; they have different purposes.

| | `/git-migrate` | `init` (no slash shortcut) |
|---|---|---|
| Purpose | Full bootstrap of a legacy repo (workflow + memory + linting + optional reformat) | Only generate `.gitworkflow.yml` via the interactive Copier wizard |
| Scope | `.gitworkflow.yml` + `.ai-memory/` + linters + `.gitignore` merge + optional pre-commit | Only `.gitworkflow.yml` |
| Defaults | Context-aware (dual naming, `require_pipeline: false` when no CI, `modules.path` auto-detected) | Answered by the dev one question at a time (wizard) |
| Typical use | Legacy GitHub customer repo adopting the new workflow for the first time | Fresh repo with custom answers the wizard captures better than fixed defaults |

**For legacy GitHub repos, `/git-migrate` is the recommended entry point**. It invokes the same underlying work as `init` for the `.gitworkflow.yml` part but **adds everything else** (memory, linters, gitignore, optional reformat) and uses context-aware defaults that match the legacy scenario.

Use `init` directly only when the wizard's per-question control is needed (e.g. a fresh GitLab repo where each answer matters and a context-aware default would be wrong).

## Gotchas

- **Pre-commit reformatting is OFF by default** for a reason: in repos with thousands of Python files, running `pre-commit run --all-files` produces a massive diff. The action always asks explicitly and puts the result in a **separate commit** (`style: apply ruff auto-formatting`) so the infra commit stays clean. Recommended: say `s` and review the reformat commit before pushing.
- **Does NOT push** — the dev keeps control. After the action finishes, run `git log --oneline -3` and review both commits. Only then `git push -u origin <integration-branch>`.
- **Working tree must be clean**. The action aborts if there are uncommitted changes — no exceptions. Commit or stash first.
- **Incremental mode is safe to re-run**. If you ran `/git-migrate` previously and want to pick up newer defaults (e.g. the skeleton added a new linter config), run it again and accept the "already migrated, add what's missing" prompt. Existing files are never overwritten.
- **Legacy GitHub dual naming is a proposal, not a force**. If your repo has only `18.0` and you decline to create `18.0-develop`, the action proceeds with single-branch config (integration = production = `18.0`). This is a supported mode but limits some flows (no staging environment). You can create the dual branches later with `git-manager` action `branch`.
- **The action reads from the odoo-skeleton clone**. Set `SKELETON_PATH` env var to point to a local clone if you have it — otherwise the legacy script will clone a fresh copy to a temp dir each time (slower, needs network).

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (ninguna — atajo de usuario directo) | — | — |

## Delegates to

- `migrate-legacy-repo.sh` (ci tooling repo) — mechanical file copy, `.gitignore` merge, skeleton clone
- [../git-manager/references/environment-detection.md](../git-manager/references/environment-detection.md) — OS/shell detection for install commands
- [../git-manager/references/commit-standards.md](../git-manager/references/commit-standards.md) — commit message format for the two commits
- [../git-manager/references/providers.yml](../git-manager/references/providers.yml) — provider detection

See common:references/shared/ for shared ecosystem patterns.
