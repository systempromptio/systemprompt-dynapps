---
name: indaws-git-review-pr
description: >-
  Revisión AI de una MR/PR abierta (GitLab/GitHub): analiza el diff para
  antipatrones Odoo, seguridad, tests faltantes, convenciones y drift
  .ai-memory/. Delega en quality-gate, odoo-code-auditor y memory-check.
  Usar ANTES de aprobar. Para revisión pre-PR local usa /git-precheck.
  Triggers: "revisa el PR 42", "revisa la merge request", "haz review del PR",
  "antes de aprobar el PR/MR", "AI review del PR", "review the PR",
  "code review of PR".
version: 2.1.1
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion, Agent]
---

# /git-review-pr — AI Code Review

> **Canonical flow**: see `git-manager/SKILL.md` action `review`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Analyzes a MR/PR like a CodeRabbit: reads the diff, detects issues using Odoo-specific knowledge from existing skills, and proposes concrete improvements. Does NOT duplicate what the reviewer already sees in GitLab/GitHub (file list, line count, etc.) — only adds value the platform can't provide.

## Usage

```
/git-review-pr              (reviews MR/PR for current branch)
/git-review-pr 42           (reviews MR/PR #42)
/git-review-pr <url>        (reviews from URL)
```

## Workflow

### 1. Find and fetch the MR/PR
- Detect provider (GitLab/GitHub) from remote URL
- Fetch MR/PR metadata (title, author, target branch, pipeline status)
- Fetch the full diff

### 2. Quick gate check
Before deep analysis, check blockers:
- **Pipeline failed?** → Report which job failed and why (use git-manager pipeline-diagnosis.md). No point reviewing code that doesn't pass CI.
- **¿Pipeline en verde?** → tampoco te la creas de entrada: un job puede salir con exit 0 **sin haber ejecutado un solo test**. Mira en el log del job cuántos se ejecutaron y si aparecen todos los módulos esperados (`odoo.tests.stats`); criterios y casos medidos en `common-skills:odoo-verify-green`. Un verde sin recuento se reporta como `[WARNING]`, no como gate superado.
- **Unresolved discussions?** → List them. The author should address these first.
- If either blocker exists, report and ask: "Do you still want the full code review?"

### 3. Deep code analysis (delegate to existing skills)
This is where the value is. Invoke the analysis skills on the changed files:

**Delegate to `quality-gate`** which in turn invokes `odoo-code-auditor`:
- **Security**: SQL injection in `cr.execute`, `sudo()` without justification, overly permissive `ir.model.access.csv`, sensitive data exposure
- **Odoo antipatterns**: `force_create=True`, compute fields without `@api.depends`, `_inherit` without `_name` on new models, manual `write` in `create` override
- **Performance**: N+1 queries in loops, `search` + `browse` instead of `search_read`, unbounded `search` without limit
- **Conventions**: naming (fields, models, XML IDs), version bump present, module structure, conventional commit in title
- **Tests**: new model/feature without tests, changed logic without updated tests. Y un test que **ya existe** no se juzga por si pasa, sino por si **discrimina**: por cada assert, *¿qué pasa si borro el código que vigila?* Delega ese juicio en `common-skills:odoo-verify-green` — no solo el verde del pipeline
- **Standard features reimplemented**: custom code that duplicates Odoo standard functionality
- **Lo que el diff NO enseña**: los **defaults y los `data/*.xml` y ACL** que le caen al código nuevo; **ejecutar la acción una segunda vez** (regenerar, reenviar, repetir); un cambio de **núcleo revisado desde sus consumidores**; y **quién es "el primero"** (`records[0]`, `[:1]`, `ensure_one()` que falta). Casos en `git-manager/references/actions.md` acción `review`
- **i18n es código visible**: PR con cadenas nuevas o cambiadas y **cero ficheros bajo `i18n/`** = hallazgo. Reescribir un `msgid` ya traducido deja la traducción huérfana y **ningún test lo caza** (mecánica en `odoo-i18n-manager`)
- **Afirmaciones del PR**: contrasta cada claim del título/cuerpo (qué bloquea o avisa, qué declara no afectado, qué dice cubrir con tests, qué dice haber actualizado) contra el código y reporta aparte las que no se sostienen — flujo y casos medidos en `git-manager/references/actions.md` acción `review`

**Check ai-memory drift** (if `.ai-memory/` exists): invoke the `memory-check` skill (delegated) with action `check --json`. This is a standard skill delegation — do NOT call the script directly via shell, and do NOT hardcode the plugin cache path (it changes with the plugin version). The `memory-check` skill is installed in the same plugin (`developer-skills`) and Claude resolves it by name.

Parse the JSON list returned by the skill and fold findings into the review output. Drift types to surface:
- `module_added` — new module in the PR, not yet in `inventory/modules.yml`
- `module_removed` — module deleted but still listed
- `missing_adr` — new/changed module without an ADR in `decisions/`
- `snapshot_stale` — `SNAPSHOT.md` hash doesn't match current YAML (manual edit or sync pending)
- `missing_schema_version` — `.ai-memory/.schema-version` missing or invalid

**Exit code handling**:
- `0` → clean, no drift
- `1` → drift detected (expected; findings go in the report)
- `2` → error de memory-check: repórtalo como CRITICAL (contrato en `memory-check/SKILL.md` § Exit code contract) y sigue con el análisis de código
- `3` → no `.ai-memory/` in the repo → skip silently; not every repo is adopted yet

### 4. Report findings
**Dos veredictos, separados y los dos escritos**: *"¿está bien el PR?"* y *"¿queda arreglado el problema del cliente?"*. Pueden no coincidir (p.ej. el cliente ejecuta un módulo que el PR no toca), y darlos juntos hace que se cierre la incidencia al mergear. Di siempre **lo que no has podido verificar y por qué**.

Present findings grouped by severity:

```
[CRITICAL] 1 finding
  security: sudo() in compute field without access check
  → acme_sale/models/sale_order.py:23
  → Suggestion: remove sudo() or add explicit access rule

[WARNING] 2 findings
  antipattern: Float field without digits parameter
  → acme_sale/models/sale_order.py:15
  → Suggestion: add digits=(16, 2)  [Float ONLY, NUNCA en Monetary: ahí es inerte — `odoo-code-auditor`]

  convention: missing test for new compute field
  → No test covers x_acme_margin computation
  → Suggestion: add test in tests/test_sale_order.py

[INFO] 1 finding
  ai-memory: new field added, inventory not updated
  → Consider running memory-check before merge
```

### 5. Offer to fix
For each finding that has an automatic fix:
- "Do you want me to fix {n} issues automatically?"
- If yes: apply fixes, commit, push. The MR updates automatically.

## What this skill does NOT do

- List files changed (the reviewer sees this in GitLab/GitHub)
- Count lines added/removed (same)
- Show a generic "Ready to merge" verdict (the reviewer decides)
- Replace the human review (this is an assistant, not a gatekeeper)

## RULES

- **NEVER** approve or merge the PR — `/git-review-pr` is an assistant; the merge/approve decision stays with the human reviewer.
- **NEVER** review code blindly on a red pipeline — surface the failure first and ask before doing the full analysis.
- **ALWAYS** group findings by severity (CRITICAL / WARNING / INFO) and skip what the platform already shows (file list, line counts).

## Gotchas

- **Pipeline rojo → preguntar antes de review** — si el pipeline de la PR está fallando, el skill avisa primero. No tiene sentido revisar código que no compila. Te pregunta si quieres continuar con el análisis de todos modos.
- **Discusiones sin resolver → listar primero** — si hay comentarios abiertos, el skill los muestra. El autor tiene que resolverlos antes de un review limpio.
- **Auto-fix pushea automáticamente** — a diferencia de `/git-precheck`, cuando aceptas *"¿corrijo los hallazgos?"* en `/git-review-pr` el skill sí commitea y pushea los fixes (la PR se actualiza sola). Es aceptable porque el reviewer ya está decidiendo.
- **No sustituye al reviewer humano** — es un asistente que pre-filtra lo mecánico. La decisión de aprobar / rechazar sigue siendo del reviewer.
- **Repo sin `.ai-memory/`** — el drift check de memoria se salta silenciosamente (exit 3 de memory-check). El análisis de código sigue igual. No es un error.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (atajo de usuario directo) | — | — |
| odoo-pr-reviewer | developer | Ejecuta la revisión de PR en la fase git tras el análisis Odoo |

## Delegates to

- `quality-gate` (ai-claude-common) — 5-gate structured review
- `odoo-verify-green` (ai-claude-common) — auditar el verde del pipeline (¿ejecutó?, ¿qué módulos?) **y si un test del diff discrimina** antes de contarlo como cubierto
- `odoo-i18n-manager` (ai-claude-developer) — catálogos: cadena nueva o `msgid` reescrito
- `odoo-code-auditor` (ai-claude-developer) — deep Odoo code analysis (invoked by quality-gate)
- `odoo-module-scanner` (ai-claude-common) — component inventory (invoked by code-auditor)
- `odoo-version-api-reference` (ai-claude-developer) — deprecated APIs, secure coding rules (invoked by code-auditor)
- [../git-manager/references/providers.yml](../git-manager/references/providers.yml) — provider CLI commands
- [../git-manager/references/pipeline-diagnosis.md](../git-manager/references/pipeline-diagnosis.md) — CI failure classification

See common:references/shared/ for shared ecosystem patterns.
