# `new-repo` action — New repo bootstrap reference

Detailed flow for the `new-repo` action of the `git-manager` skill. One-shot action to bootstrap a brand-new Odoo customer repo from the odoo-skeleton with the correct shape for the current transition period (GitHub today, GitLab tomorrow).

Read this file when you are about to run `new-repo`, or when debugging why it behaved a certain way. Sister action of `migrate` — same copier orchestration philosophy, different starting state.

## When to use

A dev creating a brand-new repo for a new customer. The directory destination is empty (or doesn't exist yet). No `.git`, no commits, no modules. Typical case: customer onboarding while we are still on GitHub.

## When NOT to use

- **Existing repos** with commits already → use `migrate` instead.
- **Repos already generated with copier** previously → re-running this overwrites everything; use `copier update` for incremental updates.
- **Repos that need a non-Indaws `org_name`** → the script hardcodes `Indaws Business Solutions S.L.`. For other orgs, run `copier copy` directly.
- **CI pipelines / non-interactive shells** — this is a human-supervised action.

---

## Step 1 — Preflight checks

- **Empty destination**:
  - If invoked with `<name>` argument → directory `<name>` must NOT exist; the script creates it.
  - If invoked without argument → current directory MUST be empty (no files, no `.git`).
- **copier installed**: aborts with install hint if missing (`pip install --break-system-packages 'copier>=9.1.0'`).
- **Skeleton resolvable**: reads in this order:
  1. `SKELETON_PATH` env var → local clone, no network (recommended).
  2. `SKELETON_REPO_URL` env var → git URL (copier handles the clone).
  3. Interactive prompt asking for the URL.
- **Platform** (`--platform=github|gitlab`): defaults to `github` while GitLab is not operational; will switch to `gitlab` default once the migration is complete.

## Step 2 — Detection (silent, no prompts)

Limited compared to `migrate` because the directory is empty:

- **Odoo version hint**: walks up the path looking for a segment matching `XX.0` (e.g., `~/Odoo/18.0/Customers/foo` → suggests `18.0`). If no hint, the user is asked.
- **`gh` availability**: detects whether `gh` is installed AND authenticated. If yes, the script offers to create the GitHub repo + push at the end. If not, it prints the manual commands.

The script does NOT detect provider from a remote (there isn't one yet), Odoo version from manifests (no manifests yet), or branches (no commits yet). All comes from the user input or the `--platform` flag.

## Step 3 — Interactive questions

Asked one at a time, with defaults:

1. **Odoo version**: defaults to the version hint if detected; otherwise asks until user enters a valid `XX.0` format.
2. **Task tracker**: `odoo` or `jira` (default `odoo`).

Hardcoded — never asked:
- `module_layout = standard` (new repos always use `parts/custom/`).
- `repo_type = customer`.
- `org_name = Indaws Business Solutions S.L.` (canonical author).
- `merge_method = squash`.
- `release_mode = develop-based`.
- `manifest_website_fix = no`.
- `readme_gen = no`.

## Step 4 — Summary + confirmation

Show the gathered context in one block. Example message:

> *"Voy a crear el repo en `~/dev/addons-acme` para Odoo 18.0, plataforma github, con ramas `18.0` (producción) + `18.0-develop` (integración). Layout standard (`parts/custom/`). Excluyo lo GitLab-only. ¿Procedo? [S/n]"*

If the user says `n` → abort. If `s` → continue.

## Step 5 — Execution

The script is a **thin orchestrator around `copier`**, identical philosophy to `migrate-legacy-repo.sh`. Copier handles all `.jinja` templates correctly (extends, includes, namespaces, loops). The script adds only what copier doesn't cover: branch creation and the optional GitHub repo creation.

1. **Run copier**:

   ```bash
   copier copy --defaults --overwrite \
     --data odoo_version=<asked> \
     --data ci_system=<platform> \
     --data task_tracker=<asked> \
     --data module_layout=standard \
     --data repo_type=customer \
     --data org_name="Indaws Business Solutions S.L." \
     --data merge_method=squash \
     --data release_mode=develop-based \
     --data manifest_website_fix=no \
     --data readme_gen=no \
     <excludes if platform=github> \
     <skeleton_src> <target_dir>
   ```

2. **Excludes when `platform=github`** (skipped in GitLab mode):
   - `dyn-build.yaml` — requires GitLab pipeline + project-specific setup.
   - `_tools/**` — post-clone scripts coupled to the GitLab pipeline.
   - `parts/dyn_shims/**` — module shims for the GitLab build system.
   - `parts/oca_pull/**` — OCA dependency manager that uses `dyn-tools` (GitLab-only today).
   - `.gitlab-ci.yml` — obvious.

   **NOT excluded** (always copied):
   - `.github/workflows/pre-commit-ci.yml` — minimal CI on GitHub from day 1.
   - `.copier-answers.yml` — enables `copier update` for future skeleton improvements.
   - `parts/custom/` — where the customer's modules will live.
   - `.ai-memory/`, `.gitworkflow.yml`, all linting configs.

3. **Sanity post-checks**: in GitHub mode, the script verifies that no excluded file leaked. If something leaks, it warns (does NOT abort) and lists the leaks. The expected state is `parts/` containing only `custom/`, no `dyn-build.yaml`, no `.gitlab-ci.yml`, etc.

4. **`git init` + first commit**:
   - `git init --initial-branch=<production-branch>` (with fallback for older git).
   - `git add -A && git commit -m "feat: initial project setup from skeleton"` (multi-line body documenting the platform, odoo version, layout, branches).
   - `git branch <integration-branch> <production-branch>`.
   - `git checkout <integration-branch>` — leaves the dev on the integration branch, ready for the first feature.

5. **Branch naming**:
   - `platform=github` → `XX.0` (production) + `XX.0-develop` (integration).
   - `platform=gitlab` → `master` (production) + `develop` (integration).

## Step 6 — Optional GitHub repo creation + push

This is the differentiator vs `migrate`. Since the repo is brand new, the script offers to handle the remote creation:

- **If `gh` is available and authenticated** (only in `platform=github` mode): asks *"¿Lo hago ahora?"* with defaults `org=indawsodoo`, `name=<basename>`, `private=yes`. If the user says yes:
  ```bash
  gh repo create indawsodoo/<name> --private --source=. --remote=origin
  git push -u origin <integration-branch>
  git push origin <production-branch>
  ```
- **If `gh` not available or user declines**: prints the manual commands clearly.
- **In `platform=gitlab` mode**: never offers — the GitLab path requires more setup (group, namespace, etc.) and is decided per-customer.

The user always has the option to decline and do it manually. The script reaches the same end-state either way: a repo on GitHub with the two branches pushed.

## Step 7 — Final report

- Show `git log --oneline -5`.
- Suggest next steps:
  1. Rellena `.ai-memory/CONTEXT.md` con 3-4 líneas del proyecto.
  2. Si vas a tener módulos OCA: vendor en `parts/oca/` manualmente (no hay `parts/oca_pull/` en modo GitHub).
  3. Cuando estés listo para tu primera feature: `/git-start P42-12345`.

---

## Differences vs `migrate`

| Aspect | `migrate` | `new-repo` |
|--------|-----------|------------|
| Starting state | Existing repo with commits, modules, sometimes a `.gitworkflow.yml` | Empty directory, no `.git`, no commits |
| Odoo version detection | From `__manifest__.py` files | From parent dir hint or asks user |
| Branches | Detected (often single `XX.0`) → proposes dual naming | None exist yet → creates dual from scratch |
| Module layout | `legacy` (root) or `standard` (`parts/custom/`) — auto-detected | Always `standard` (no legacy concern in a new repo) |
| Pre-commit reformat option | Yes (4th question, separate commit if accepted) | No — there's nothing to reformat in an empty repo |
| Pushes? | No, ever | No by default, but **optionally** offers `gh repo create + push` if `gh` is authenticated |
| Commits created | 1-2 (infra + optional reformat) | 1 (infra only) |

## Differences vs `init`

`init` only generates `.gitworkflow.yml`. `new-repo` does the full bootstrap (`.gitworkflow.yml` + `.ai-memory/` + linting + `parts/custom/` + first commit + dual branches). Use `init` only when you have a partially-set-up repo and just want the workflow config.

## Plan for GitLab migration (future)

When GitLab becomes operational:

- Default of `--platform` flips from `github` to `gitlab` (or is auto-detected from the session context).
- In `gitlab` mode, **no excludes** are applied — the full skeleton is rendered (including `dyn-build.yaml`, `parts/dyn_shims/`, `parts/oca_pull/`, `.gitlab-ci.yml`).
- Branch naming becomes `master` + `develop` instead of `XX.0` + `XX.0-develop`.
- The optional remote creation step changes from `gh repo create` to `glab repo create`. Or removed entirely if customers' GitLab projects are pre-provisioned by another flow (DynCloud España).
- The user-facing command (`/git-new-repo`) **does NOT change**. The complexity stays inside the script.

Eventually, when no more repos are created on GitHub, the GitHub branch of the script can be deleted and the script simplified. `git-new-repo` survives the migration.

## Scope boundaries

- **Does NOT push** by default — the dev keeps control. Optional `gh repo create + push` step is opt-in.
- **Does NOT create a PR** — there's nothing to PR yet, the repo is just born. First feature uses `/git-start` then `/git-pr`.
- **Does NOT replace `init`** — `init` is for partial setups.
- **Does NOT replace `migrate`** — `migrate` is for repos with commits already.
- **Does NOT run in CI** — human-supervised only.
- **Does NOT manage OCA dependencies in GitHub mode** — `parts/oca_pull/` is excluded; the dev vendors OCA modules manually in `parts/oca/`.

## Why it lives inside git-manager

Same rationale as `migrate`: the action mutates git state (creates branches, commits) and reads/writes `.gitworkflow.yml`. Sharing the parent skill reuses environment detection, the provider CLI check, commit-standards reference, etc. The `git-new-repo/` folder is a slash-command shortcut only.
