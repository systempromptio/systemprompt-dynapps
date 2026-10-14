# `migrate` action — Legacy repo bootstrap reference

Detailed flow for the `migrate` action of the `git-manager` skill. One-shot action to bring a legacy repo (typically GitHub, pre-workflow) up to the current Indaws standard: `.gitworkflow.yml`, `.ai-memory/`, linting configs, `.gitignore` updates, optional pre-commit reformatting — all in **two separate commits** (infra + optional reformat).

Read this file when you are about to run `migrate`, or when debugging why it behaved a certain way.

## When to use

A dev working on a legacy repo that doesn't yet follow the new workflow — no `.gitworkflow.yml` (or an outdated one), no `.ai-memory/`, no pre-commit configs. Typical case: legacy GitHub customer repos.

## When NOT to use

- Fresh Copier-generated repos — they already have the full infrastructure. Run `/memory-check sync` instead.
- Repos in a CI pipeline context — this is a human-supervised action.

---

## Step 1 — Preflight checks

- **Git repo**: `git rev-parse --git-dir`. If not a git repo, abort.
- **Clean working tree**: no uncommitted changes. If dirty, ask the user to commit or stash first; do NOT proceed.
- **Script available**: the action invokes `scripts/migrate-legacy-repo.sh` (adjacent to this SKILL directory). If missing, report a clear error with the expected path — do NOT try to work around it.
- **Incremental mode check**: if `.ai-memory/` already exists in the target repo, tell the user *"Este repo parece ya migrado (ya tiene `.ai-memory/`). ¿Quieres que añada solo lo que falte (modo incremental)?"*
  - If accepted → **incremental mode**: only copy files that don't already exist; do NOT overwrite anything.
  - If rejected → abort.

## Step 2 — Context detection (silent, no prompts)

- **Provider**: parse `git remote get-url origin` → GitHub vs GitLab.
- **Branches**: `git branch -a` → identify naming pattern:
  - Legacy GitHub with a single version branch (`18.0`, `17.0`) → **dual naming proposal**: `X.Y` (production) + `X.Y-develop` (integration, to be created).
  - Already has `develop` + `master` → **modern naming**: use as-is.
  - Has `main` only → ask what to do (propose `develop` + rename `main` → `master`, or keep `main`).
- **Odoo version**: first `__manifest__.py` found → `version` field (e.g. `"18.0.1.0.0"` → Odoo 18).
- **Layout**: `parts/custom/` with manifests → standard; manifests at root → legacy `.`; otherwise fallback chain.
- **CI detected**: check for `.github/workflows/*.yml` or `.gitlab-ci.yml`.

## Step 3 — Summary + confirmation

Show the detected context to the user in one block. Include the dual-naming proposal when applicable. Example message:

> *"Detectado: repo GitHub legacy, Odoo 18, layout raíz (2 módulos), sin CI, rama única `18.0`. Propongo crear `18.0-develop` para separar integración (staging) de producción. Voy a: generar `.gitworkflow.yml`, copiar `.ai-memory/` v2, copiar configs de linting, fusionar `.gitignore` con el del skeleton. Opcional: `CODEOWNERS`, `.coveragerc`. ¿Procedo? [s/n]"*

If the user says `n` → abort. If `s` → continue to step 4.

## Step 4 — Interactive questions (one at a time)

Ask in this order, one at a time:

1. **Create integration branch** (only if applicable):
   > *"¿Creo la rama `X.Y-develop` ahora desde `X.Y`?"* (default **s**)

2. **CODEOWNERS**:
   > *"¿Copio `CODEOWNERS`?"* (default **n**)

3. **`.coveragerc`**:
   > *"¿Copio `.coveragerc`?"* (default **n**)

4. **Pre-commit on full codebase**:
   > *"¿Aplico pre-commit a todo el código existente? Reformateará imports, comillas, espacios. Generará un commit separado `style: apply ruff auto-formatting`. En repos grandes pueden ser miles de líneas. **Recomendado: sí**, pero revisa el commit antes de pushear."* (default **n**)

## Step 5 — Execution

The migration script is a **thin orchestrator around `copier`**. Copier is the Jinja renderer the skeleton was designed for, so all `.jinja` templates (`.pre-commit-config.yaml.jinja`, `.ruff.toml.jinja`, `.pylintrc.jinja`, `.gitworkflow.yml.jinja`, etc.) are rendered correctly regardless of complexity (extends, includes, namespaces, loops). The script adds only what copier does not cover: `.gitignore` merge and `.gitworkflow.yml` post-processing for legacy GitHub dual naming.

1. **Invoke the script**:

   ```bash
   SKELETON_PATH=<path>  bash <skill_dir>/scripts/migrate-legacy-repo.sh  <target_repo>
   ```

   **Skeleton source**: URL-agnostic — no URL is hardcoded. The script reads (in this order):
   1. `SKELETON_PATH` env var → local clone, no network (recommended when available).
   2. `SKELETON_REPO_URL` env var → git URL to clone (copier handles the clone internally).
   3. If neither is set → prompts the dev interactively for the URL (the dev owns the source).

   **Requirements**: `copier >= 9.1.0` installed. The script aborts with an install hint if copier is missing (`pip install --break-system-packages 'copier>=9.1.0'`).

2. **What the script does (internally)**:

   - Preflight: git repo, copier installed, resolve skeleton source.
   - Detection: Odoo version (from first `__manifest__.py`), CI system (from remote URL), module layout (by inspecting `parts/custom/` vs root manifests).
   - Backup `.gitignore` if present (copier overwrites, so we preserve the repo-specific entries).
   - Run `copier copy --defaults --overwrite` with the detected answers:
     - `odoo_version=<detected>`, `ci_system=<detected>`, `module_layout=<detected>`
     - Fixed choices: `org_name=Indaws Business Solutions S.L.` (canonical author of every Indaws module), `repo_type=customer`, `task_tracker=odoo`, `merge_method=squash`, `release_mode=develop-based`, `manifest_website_fix=no`, `readme_gen=no`
   - Excluded from the copy (not needed for legacy migration):
     - `dyn-build.yaml` (requires project-specific setup)
     - `parts/**` (legacy keeps modules at repo root, no shims/oca_pull either)
     - `_tools/**` (post-clone scripts that don't apply here)
     - `CODEOWNERS`, `.coveragerc` (asked interactively)
     - `requirements*.txt` (legacy repos usually manage deps externally)
     - `.gitlab-ci.yml` (GitLab CI setup is out of scope for legacy GitHub repos)
   - **Generates `.github/workflows/pre-commit-ci.yml`** when `ci_system=github` — a lightweight workflow that runs pre-commit on every PR. No runners or secrets needed. Aligns with the "what we ship YA in GitHub" message for the team.
   - **Generates `.copier-answers.yml`** automatically. The dev can later run `copier update` to pick up skeleton improvements without re-running `/git-migrate`.
   - Post-process `.gitignore`: append the original repo's entries that are not already in the skeleton's `.gitignore`.
   - Post-process `.gitworkflow.yml` (only for legacy GitHub):
     - Rewrite `branches:` from default (`develop` + `master`) to dual naming: `X.Y-develop` (integration) + `X.Y` (production).
     - Set `merge_requests.require_pipeline: false` (no CI yet on legacy GitHub).
   - Interactive prompts (only on TTY, skipped otherwise):
     - `¿Copiar CODEOWNERS? [y/N]`
     - `¿Copiar .coveragerc? [y/N]`

3. **If the user accepted creating `X.Y-develop`** in step 4: `git branch X.Y-develop X.Y`. Do NOT push it.

4. **If pre-commit accepted** in step 4: `pre-commit install && pre-commit run --all-files` → capture the list of modified files for the commit 2 stage.

## Step 6 — Commit separation (critical — do NOT merge the two)

**Commit 1** (`feat: add git workflow + ai-memory infrastructure`):

- Stage: `.gitworkflow.yml`, `.ai-memory/` (all of it), linting configs (`.pre-commit-config.yaml`, `.ruff.toml`, `.pylintrc`, `.pylintrc-mandatory`, `.editorconfig`, `eslint.config.cjs`, `prettier.config.cjs`), updated `.gitignore`, optional `CODEOWNERS` + `.coveragerc`.

**Commit 2** (only if pre-commit was applied):

- Title: `style: apply ruff auto-formatting (mass reformat)`
- Stage: all files modified by `pre-commit run --all-files`.
- Body: *"Run after adopting the linting stack from the skeleton. Non-functional changes only — formatting, import order, quotes. Review and ensure nothing functional leaked in."*

**Do NOT push**. The dev reviews before pushing.

## Step 7 — Final report

- Show `git log --oneline -3`.
- Suggest next steps in order:
  1. Revisa los commits, especialmente el 2º si aplicaste pre-commit.
  2. Rellena `.ai-memory/CONTEXT.md` con 3-4 líneas del proyecto.
  3. Ejecuta `/memory-check sync` para poblar el inventario de módulos.
  4. Cuando estés listo: `git push -u origin <integration-branch>` (y opcionalmente la rama de producción si se creó nueva).

---

## Incremental mode (from Step 1)

If the user accepted incremental mode, skip any file that already exists. Do NOT overwrite `.gitworkflow.yml`, do NOT rewrite `.ai-memory/` contents. Only add files that are missing.

Useful for repos partially migrated by an older script version, or repos that got some pieces manually.

## Scope boundaries

- **Does NOT push** — the dev keeps control until they review.
- **Does NOT create a PR** — that's `/git-pr`.
- **Does NOT replace `init`** — `init` generates `.gitworkflow.yml` via the interactive Copier wizard (for fresh GitLab repos with custom answers). `migrate` uses fixed defaults suitable for legacy GitHub repos and skips the wizard.
- **Does NOT run in CI** — human-supervised only.

## Why it lives inside git-manager (not a separate skill)

The action mutates the repo's git state (creates branches, commits) and reads/writes `.gitworkflow.yml` — same concerns as the rest of git-manager. Keeping it here reuses the environment detection, the provider CLI check, and the commit-standards references. The `git-migrate/` folder is a slash-command shortcut only.
