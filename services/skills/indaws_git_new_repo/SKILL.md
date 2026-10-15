---
name: indaws-git-new-repo
description: >-
  Crea un nuevo repo de cliente Odoo desde el odoo-skeleton: copier, git init,
  primer commit y ramas production/integration. Crea el repo GitHub si gh está
  autenticado. Sister of /git-migrate (repo legacy ya existente).
  Triggers: "crea repo nuevo", "nuevo repositorio", "new repo", "bootstrap a new
  repo", "arranca un cliente nuevo", "nuevo cliente", "scaffold a customer repo",
  "set up a new customer repo", "genera un repo desde el skeleton".
version: 1.0.4
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# /git-new-repo — Bootstrap a brand-new Odoo customer repo

> **Canonical flow**: see `git-manager/SKILL.md` action `new-repo`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Shortcut for `git-manager` action `new-repo`. Generates a brand-new customer repo from the odoo-skeleton with the correct excludes and structure for the target platform (default `github`, set in `new-repo.sh`; `--platform=gitlab` for GitLab projects).

Creates 1 commit and the dual branches. Optionally creates the GitHub repo and pushes if you let it. The dev decides.

## Usage

```
/git-new-repo                     # works in cwd; cwd must be empty
/git-new-repo addons-acme         # creates ./addons-acme and works inside
/git-new-repo --platform=gitlab   # forces GitLab mode (no excludes)
```

The script auto-detects an Odoo version hint from the parent directory (e.g., if you're in `~/Odoo/18.0/Customers/`, it suggests `18.0`).

## What it does (summary)

1. **Preflight**: target dir must be empty (or be created from `<name>` argument); copier installed; skeleton resolvable via `SKELETON_PATH`, `SKELETON_REPO_URL`, or interactive prompt.
2. **Asks** Odoo version (with hint) and task tracker (odoo — Odoo project tasks via `dynapps-odoo`). The rest are hardcoded (`module_layout=standard`, `repo_type=customer`, `org_name=Indaws Business Solutions S.L.`, etc.).
3. **Renders** the skeleton with `copier copy --defaults` excluding GitLab-only files when targeting GitHub.
4. **Sanity-checks** that no excluded file leaked.
5. **`git init`** + first commit + creates the dual branches (`X.Y` + `X.Y-develop` on GitHub; `master` + `develop` on GitLab).
6. **Offers** to create the GitHub repo and push (`gh repo create` + `git push`), if `gh` is authenticated. Otherwise prints the manual commands.
7. **Reports** with next steps: fill `CONTEXT.md`, vendor OCA in `parts/oca/`, kick off first feature with `/git-start`.

## When to use

- New customer onboarding.
- Any new repo where you want the Indaws workflow from day 1 with `parts/custom/` already set up.

> ⚠️ Antes de proponer el layout `parts/`, avisa al usuario de que ningún proyecto existente lo usa y
> ofrece el layout real (`desarrollos/` + symlinks, ver `common-skills:indaws-project-setup`).
- Repos that will move to GitLab eventually — the `--copier-answers.yml` lets you re-render later with `copier update`.

## When NOT to use

- **Existing repos with commits already** → use `/git-migrate` instead.
- **Repos already generated with copier** → re-running overwrites; use `copier update` for incremental updates.
- **Repos that need a non-Indaws `org_name`** → the script hardcodes `Indaws Business Solutions S.L.`. For other orgs, run `copier copy` directly.
- **CI pipelines / non-interactive shells** — this is a human-supervised action.

## Relation to the `migrate` action and `init`

| | `/git-new-repo` | `/git-migrate` | `init` (no shortcut) |
|---|---|---|---|
| Starting state | Empty dir, no `.git` | Existing repo with commits | Existing repo (any state) |
| Output | Full skeleton + git init + first commit + 2 branches | `.gitworkflow.yml` + `.ai-memory/` + linters merged into existing repo | Only `.gitworkflow.yml` |
| Pushes? | Optional (`gh repo create + push`) | Never | Never |
| Commits | 1 (infra) | 1-2 (infra + optional reformat) | 0 (writes file, you commit) |
| Typical use | New customer | Adopting workflow on legacy GitHub repo | Partial setup / GitLab repo with custom answers |

**Decision tree**:
- Empty directory, want a new customer repo → `/git-new-repo`
- Existing GitHub repo, want to adopt the workflow → `/git-migrate`
- Existing repo with workflow partially configured, just need `.gitworkflow.yml` → `init`

## Gotchas

- **Empty directory required**. Without an argument, `cwd` must be empty. With an argument, `<name>` must NOT exist (the script creates it). No exceptions — there's no "merge into existing files" mode for new-repo (use `migrate` for that).
- **No automatic OCA dependency management on GitHub**. `parts/oca_pull/` is excluded because it depends on `dyn-tools` (GitLab-only). If you need OCA modules, vendor them manually in `parts/oca/` (clones or submodules). With `--platform=gitlab`, `parts/oca_pull/` is available.
- **Defaults to GitHub mode** (set in `new-repo.sh`). Use `--platform=gitlab` for GitLab projects.
- **`gh repo create` is OPT-IN**. The script never creates a remote repo without your explicit `s/y` answer. If `gh` is not authenticated, it skips the question and prints the manual commands.
- **Hardcoded `org_name`**. `Indaws Business Solutions S.L.` is the canonical author for every Indaws module. If you need a different org (rare — only for non-Indaws customers), use `copier copy` directly.
- **`SKELETON_PATH` recommended**. The fastest path. Without it, copier clones the skeleton each time (slow + needs network). Add `export SKELETON_PATH=~/odoo-skeleton` to your shell rc.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (ninguna — atajo de usuario directo) | — | — |

## Delegates to

- `new-repo.sh` (skill scripts dir) — copier orchestration, git init, branch setup, optional gh repo create
- [../git-manager/references/new-repo-action.md](../git-manager/references/new-repo-action.md) — full flow documentation
- [../git-manager/references/environment-detection.md](../git-manager/references/environment-detection.md) — OS/shell detection for install commands
- [../git-manager/references/commit-standards.md](../git-manager/references/commit-standards.md) — commit message format
- [../git-manager/references/providers.yml](../git-manager/references/providers.yml) — provider detection (used by parent skill, not this script directly)

See common:references/shared/ for shared ecosystem patterns.
