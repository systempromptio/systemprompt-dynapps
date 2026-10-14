---
name: indaws-git-precheck
description: >-
  Revisa el codigo de tu rama antes de abrir el PR (diff base..HEAD), con los mismos detectores que la review de PR. Úsala para revisar el diff local antes de abrir el PR ("revisa mi rama", "precheck", "revisa antes de subir", "review before push").
version: 1.0.6
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion, Agent]
---

# /git-precheck — Pre-MR Code Review

> **Canonical flow**: see `git-manager/SKILL.md` action `precheck`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Same detectors as `/git-review-pr`, but runs on the **local branch diff** so the developer finds problems **before** opening the PR. Catches antipatterns, security issues, missing tests, convention violations, and `.ai-memory/` drift early — when fixing them is cheap.

**Rule of thumb**: run `/git-precheck` right before `/git-pr`. If it surfaces critical findings, fix them first.

## Usage

```
/git-precheck          (reviews current branch against its base)
```

The base branch is read from `.gitworkflow.yml` → `branches.integration`. If no config, the skill infers it from repo signals and confirms once.

## Step-by-step flow

### 1. Resolve the diff target
- Detect the current branch name. If on a protected branch (integration or production), stop: precheck only makes sense on feature/hotfix branches.
- Read `.gitworkflow.yml` → `branches.integration`. Fallback: infer (`develop`, `18.0-develop`, `main`).
- Compute `git fetch origin <base>` and `git diff origin/<base>..HEAD`.
- Count commits ahead. If zero, tell the user: *"No hay commits nuevos sobre `<base>`. Nada que revisar."* and exit.

### 2. List changed files
- Get `git diff --name-status origin/<base>..HEAD`
- Classify: Python, XML, JS/OWL, YAML, manifests, security, other.
- If the diff is empty or only touches docs, tell the user: *"Solo hay cambios en documentación. Nada que analizar técnicamente."* and skip step 3.

### 3. Delegate code analysis (same detectors as `/git-review-pr`)

**Delegate to `quality-gate`** which in turn invokes `odoo-code-auditor`:
- **Security**: SQL injection in `cr.execute`, `sudo()` without justification, overly permissive `ir.model.access.csv`, sensitive data exposure.
- **Odoo antipatterns**: `force_create=True`, compute fields without `@api.depends`, `_inherit` without `_name` on new models, manual `write` in `create` override.
- **Performance**: N+1 queries in loops, `search` + `browse` instead of `search_read`, unbounded `search` without limit.
- **Conventions**: field/model/XML ID naming, version bump present, module structure, conventional commit in the local commits.
- **Tests**: new model/feature without tests, changed logic without updated tests. Y si el dev dice que «pasan», el precheck no corre la suite pero **sí cuestiona la afirmación** con `common-skills:odoo-verify-green`: ¿ese verde ejecutó algo, salió de una base limpia, y se vio caer al neutralizar el arreglo? Caso concreto que se ve solo en el diff: tocar `static/src/` sin haber corrido los tours (`-standard` no entra en el pase por defecto) → `[WARNING]`.
- **Standard features reimplemented**: custom code duplicating Odoo standard behavior.

### 4. Check `.ai-memory/` drift

Invoke the `memory-check` skill (delegated) with action `check --json`. This is a standard skill delegation — do NOT call the script directly via shell, and do NOT hardcode the plugin cache path (it changes with the plugin version). The `memory-check` skill is installed in the same plugin (`developer-skills`) and Claude resolves it by name.

Parse the JSON list returned by the skill. Drift types to surface:
- `module_added` — new module present in `parts/custom/` (or wherever `modules.path` points) but not in `inventory/modules.yml`
- `module_removed` — module deleted but still listed
- `missing_adr` — new/changed module without an ADR in `decisions/`
- `snapshot_stale` — `SNAPSHOT.md` hash doesn't match current YAML (manual edit or sync pending)
- `missing_schema_version` — `.ai-memory/.schema-version` missing or invalid

**Exit code handling**:
- `0` → clean, no drift
- `1` → drift detected (expected; findings go in the report)
- `2` → error de memory-check: repórtalo como CRITICAL (contrato en `memory-check/SKILL.md` § Exit code contract) y sigue con el análisis de código
- `3` → no `.ai-memory/` in the repo → skip silently; not every repo is adopted yet

### 5. Consolidate and report

Group findings by severity — header line (branch → base, commits ahead, files changed), then `[CRITICAL]` / `[WARNING]` / `[INFO]` blocks, each finding showing type, `file:line`, and a one-line suggestion. For `memory-drift` findings suggest `/memory-check sync`.

### 6. Offer auto-fix (optional)

For straightforward fixes, ask *"Hay X hallazgos auto-fixables. ¿Los aplico?"* — if yes, apply and leave them staged (NOT committed; the dev owns the commit).

### 7. Verdict (soft)

- Any `critical` → *"Hay X hallazgos críticos. Te recomiendo arreglarlos antes de `/git-pr`."*
- Only `warning`/`info` → *"Cambios viables. Puedes abrir el PR; considera revisar los N warnings."*
- No findings → *"Sin hallazgos. Listo para `/git-pr`."*

## What this skill does NOT do

- Open the PR (use `/git-pr` for that)
- Commit the fixes automatically (the dev owns the commit)
- Replace the human reviewer (this is a pre-filter, the PR still needs review)
- Run tests or pipelines (that's the CI's job)

**Relation to `/git-review-pr`**: same detectors and soft verdict, two entry points — `/git-precheck` runs on the local `<base>..HEAD` diff for the developer before `/git-pr`; `/git-review-pr` runs on the open PR diff for the reviewer.

## RULES

- **NEVER** open the PR — `/git-precheck` is local analysis only; the dev runs `/git-pr` afterwards.
- **NEVER** commit the auto-fixes — leave them staged so the dev owns the commit.
- **ALWAYS** treat the verdict as a recommendation, not a gate — the dev decides whether to open the PR.

## Gotchas

- **Sin commits sobre la base** — si no hay diferencias con la rama de integración, el skill te lo dice (*"Nada que revisar"*) y sale sin invocar los detectores. Ejecutarlo en una rama recién creada no rompe nada.
- **Repo sin `.ai-memory/`** — el drift check de memoria se salta silenciosamente (exit 3 de memory-check). El code review sigue funcionando igual. No es un error.
- **Auto-fix deja cambios staged pero NO commitea** — el dev controla el commit. Si aceptas *"¿aplico los fixes?"*, el skill modifica los archivos, los añade al staging, y para ahí. Tienes que commitear tú.
- **`/git-precheck` NO abre el PR** — es puramente análisis local. Tras el verdict, sigues con `/git-pr` manualmente.
- **En rama protegida no corre** — si estás en la rama de integración o producción, el skill se detiene: el precheck solo tiene sentido en ramas feature/hotfix con diff sobre la base.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (ninguna — atajo de usuario directo) | — | — |

## Delegates to

- `quality-gate` (ai-claude-common) — 5-gate structured review
- `odoo-verify-green` (ai-claude-common) — cuestionar el «los tests pasan» del dev sin correr la suite (qué exige un verde para valer, y el aviso de tours no ejecutados)
- `odoo-code-auditor` (ai-claude-developer) — deep Odoo code analysis (invoked by quality-gate)
- `odoo-module-scanner` (ai-claude-common) — component inventory (invoked by code-auditor)
- `odoo-version-api-reference` (ai-claude-developer) — deprecated APIs, secure coding rules (invoked by code-auditor)
- `memory-check` (ai-claude-developer) — `.ai-memory/` drift detection (action `check --json`)
- [../git-manager/references/providers.yml](../git-manager/references/providers.yml) — base branch detection fallbacks

See common:references/shared/ for shared ecosystem patterns.
