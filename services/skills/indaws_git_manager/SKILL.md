---
name: indaws-git-manager
description: >-
  Ejecuta cualquier operación git (commit, push, rama, PR/MR, cherry-pick,
  hotfix, release, tag, sync, cleanup, review, CI, recuperación). Provider-agnostic
  GitLab/GitHub; lee .gitworkflow.yml o infiere convenciones. Shortcut skills:
  /git-start /git-sync /git-precheck /git-pr /git-review-pr /git-status
  /git-migrate /git-new-repo.
  Triggers: "commit", "haz commit", "push", "crea rama", "abre PR/MR",
  "merge / rebase / cherry-pick", "diagnostica el pipeline",
  "deshaz el commit", "hotfix / release / tag", "migra el repo",
  "crea repo nuevo", "create branch", "open pull request".
version: 4.5.1
model: opus
effort: high
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# Git Manager

Generic git workflow interpreter. Knows git and provider CLIs. Reads `.gitworkflow.yml` to learn how each repo works. Without it, infers conventions from the repo and asks when unsure.

**The skill knows git. The config tells it how to apply that knowledge here.**

## Agent routing

| Model | Actions |
|-------|---------|
| **sonnet** (subagent) | status, branch, commit, push, mr-status, sync, cleanup |
| **sonnet** (atajo, `model:` de su SKILL.md) | review, precheck, migrate, new-repo |
| **opus** (principal) | guide, pipeline, cherry-pick, hotfix, release, recovery, init |

## Startup sequence (condensed)

Full detail in [actions.md](references/actions.md) → "Startup sequence (full)".

1. **Detect environment** — OS/shell/terminal ([environment-detection.md](references/environment-detection.md)); picks install/syntax. Store for the session.
2. **Verify git repo** — `git rev-parse --git-dir`; if not a repo, stop.
3. **Read workflow config** — `.gitworkflow.yml`: config mode (found → interpret fully) or inference mode (absent → infer from branches/remote/commits, confirm before ambiguous actions).
4. **Detect provider + verify CLI** — config field → CLI availability → remote URL. Check `gh`/`glab` installed + authenticated; if missing, ask before installing. Same pattern for every external tool (`pre-commit`, etc.).
5. **Repo snapshot + drift check** — note `.gitworkflow.yml`/CI/`.pre-commit-config.yaml`/`CODEOWNERS`/`dyn-build.yaml`. In config mode, compare against platform API and offer auto-fix (platform truth > config for shared settings).

## Actions index

Full descriptions in [actions.md](references/actions.md). Each action's model in brackets.

| # | Action | Model | # | Action | Model |
|---|--------|-------|---|--------|-------|
| 1 | status | sonnet | 11 | tag | opus |
| 2 | guide | opus | 12 | recovery | opus |
| 3 | branch | sonnet | 13 | init | opus |
| 4 | commit | sonnet | 14 | sync | sonnet |
| 5 | push | sonnet | 15 | cleanup | sonnet |
| 6 | mr-status | sonnet | 16 | review | opus |
| 7 | pipeline | opus | 17 | precheck | opus |
| 8 | cherry-pick | opus | 18 | migrate | opus |
| 9 | hotfix | opus | 19 | new-repo | opus |
| 10 | release | opus | | | |

## GOTCHAS

1. **Never `push --force` to protected branches** (develop, master, version branches) — and never push directly to them; always via MR/PR. These are non-negotiable guardrails.
2. **Never `git flow finish`** — it bypasses MR / pipeline / squash / branch protection in one shot. Always go through the explicit branch + MR flow.
3. **Pre-commit gate is mandatory before push** (action 5): if `.pre-commit-config.yaml` exists, run `pre-commit run --all-files` first. Auto-fixable failures → fix, amend, proceed; lint errors → stop. Never skip with `--no-verify`.
4. **`memory-check` is delegated and non-blocking** — invoke the skill by name (do NOT call its script via shell). Exit code 3 (`.ai-memory/` absent) → skip silently; non-adopted repos are out of scope.
5. **Never `--remove-source-branch` on permanent branches** — only feature/hotfix branches are deleted after merge. Protected branches are never deleted.
6. **hotfix order is production FIRST, then integration** — branch from production, MR to production, tag, then sync down. Getting the order wrong loses the fix on the next release.
7. **Cherry-pick / tag are production operations → require explicit confirmation.** Preview cherry-pick with `--no-commit`; tags are always annotated and may trigger deployments.
8. **Default platform is `github`** (actions 18/19, set in `new-repo.sh`); `--platform=gitlab` for GitLab projects.

## RULES

Non-negotiable guardrails — never bypass, explain why instead:

### NEVER
- Never `push --force` to protected branches.
- Never push directly to protected branches — always via MR/PR.
- Never `git flow finish` (bypasses MR/pipeline/squash/protection).
- Never stage sensitive files (.env, credentials, keys).
- Never skip pre-commit (`--no-verify`).
- Never delete protected/permanent branches.
- Never call the `memory-check` script directly via shell — delegate the skill by name.

### ALWAYS
- Always confirm before cherry-pick to production and before creating/deleting tags.
- Always run the pre-commit gate before push when `.pre-commit-config.yaml` exists.
- Always verify the provider CLI is installed + authenticated before remote operations.
- Always edit the canonical action in `references/actions.md`, not the shortcut wrappers.

## Branch cleanup policy

Feature/hotfix branches are deleted by default after merge (traceability stays in the squashed commit, the archived MR/PR, and platform audit events). Protected/permanent branches are never deleted. Full rationale in [actions.md](references/actions.md).

## Dry-run mode

When the user asks "what would you do?" / "dry-run" / "show commands": show the exact commands in order without executing, with explanations, then ask "Execute? [y/n]".

## References

| Ref | When to read |
|-----|-------------|
| [actions.md](references/actions.md) | Full action descriptions, full startup, slash shortcuts, policies |
| [providers.yml](references/providers.yml) | Startup — provider detection, CLI commands, install, platform validation |
| [git-concepts.md](references/git-concepts.md) | Merge methods, cherry-pick, sync, fork model, tags |
| [gotchas.md](references/gotchas.md) | Before any risky operation (cherry-pick, tag, hotfix, release) |
| [commit-standards.md](references/commit-standards.md) | Before committing — choosing the right type |
| [mr-template.md](references/mr-template.md) | When creating a Merge/Pull Request |
| [pipeline-diagnosis.md](references/pipeline-diagnosis.md) | When a pipeline/CI fails |
| [recovery.md](references/recovery.md) | When something goes wrong |
| [environment-detection.md](references/environment-detection.md) | Startup — detect OS, shell, terminal |
| [init-wizard.md](references/init-wizard.md) | Before action `init` |
| [migrate-action.md](references/migrate-action.md) | Before action `migrate` |
| [new-repo-action.md](references/new-repo-action.md) | Before action `new-repo` |
| `common:references/shared/` | Cross-suite shared conventions |

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| `quality-gate` | common | Deep code analysis on review/precheck |
| `odoo-code-auditor` | developer | Odoo-specific checks (via quality-gate) |
| `memory-check` | developer | `.ai-memory/` drift check on review/precheck/commit |
| `odoo-module-scanner` | common | Module inventory when scoping changes |

## Consumed by

| Skill | Suite | Context |
|-------|-------|---------|
| `git-start` | developer | Shortcut → action `branch` |
| `git-sync` | developer | Shortcut → action `sync` |
| `git-precheck` | developer | Shortcut → action `precheck` |
| `git-pr` | developer | Shortcut → action `push` |
| `git-review-pr` | developer | Shortcut → action `review` |
| `git-status` | developer | Shortcut → action `status` |
| `git-migrate` | developer | Shortcut → action `migrate` |
| `git-new-repo` | developer | Shortcut → action `new-repo` |
| `dynapps-code-migration` | developer | Commits, MRs, pipeline diagnosis |
