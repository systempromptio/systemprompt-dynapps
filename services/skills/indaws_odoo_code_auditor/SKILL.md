---
name: indaws-odoo-code-auditor
description: >-
  Audita código de módulos Odoo (Python/JS/XML): deuda técnica, seguridad,
  rendimiento y conformidad con convenciones. Genera informes a 3 niveles
  (high/medium/low) para desarrolladores o stakeholders. NOT diagnóstico de
  errores en runtime (→ odoo-traceback-doctor).
  Triggers: "audit this Odoo module", "analiza el código Odoo en path X",
  "evalúa la calidad del proyecto", "assess code from another partner",
  "technical debt Odoo", "security vulnerabilities Odoo code".
version: 1.2.1
effort: high
model: opus
---

# Odoo Code Auditor

Comprehensive code auditing skill for Odoo modules. Focuses on code migrability, quality assessment, identifying code duplication, security vulnerabilities, performance issues, and compliance with Odoo conventions.

## References

| File | Contents | Load when |
|------|----------|-----------|
| [scripts/find_audit_candidates.py](scripts/find_audit_candidates.py) | **Pre-localizador determinista** de candidatos por dimensión (VERIFICAR c/u contra gotchas.md). Escanea el fichero **entero**, no línea a línea: `black` parte las llamadas y una inyección escrita en dos líneas era invisible. Banco de regresión: `tools/bench-audit` (23/23) | Paso 3 — antes de analizar |
| [references/gotchas.md](references/gotchas.md) | False positive avoidance rules | **Always** (before reporting findings) |
| [references/migrability-checklist.md](references/migrability-checklist.md) | Deprecated APIs, version patterns, migration scoring | User selects Migrability |
| [references/code-quality-checklist.md](references/code-quality-checklist.md) | Duplication, complexity, maintainability, docs, organization | User selects Code Quality |
| [references/hollow-tests.md](references/hollow-tests.md) | Hollow-test detector (green but verifies nothing) + latent-bug smells: fixed-width transliteration after concat, silent numeric truncation, float-money rounding, cross-company leak, commit-in-business, noupdate-data needing migration, unjustified sudo() | Code Quality (tests present) or any dimension when a listed smell matches |
| [references/security-patterns.md](references/security-patterns.md) | SQL injection, XSS, access control, dangerous patterns | User selects Security |
| [references/mcp-exposure-security.md](references/mcp-exposure-security.md) | MCP/API/JSON-RPC/OpenAPI exposure: exposure≠ACL, metadata leaks, domain/field validation (dotted-path, measures, empty read-back), non-su reads, multi-principal, body-size cap | Security + module exposes a generic model/tool surface (mcp.server.config, runtime-chosen model/domain) |
| [references/concurrency-atomicity.md](references/concurrency-atomicity.md) | TOCTOU / check-then-write, single-use tokens & quotas, commit-per-task batches, idempotency of side effects | Security or Code Quality + module has tokens/quotas/counters, batch crons, or external side effects |
| [references/performance-patterns.md](references/performance-patterns.md) | N+1 queries, ORM misuse, computed fields, memory | User selects Performance |
| [references/odoo-conventions.md](references/odoo-conventions.md) | Module structure, naming, model attributes, view conventions | User selects Conventions |
| [references/standard-features.md](references/standard-features.md) | Core features reimplemented as custom code | User selects Standard Features |
| [references/report-generation.md](references/report-generation.md) | Report lengths, technical levels, structure, scoring, severity | When generating the report |
| [references/analysis-techniques.md](references/analysis-techniques.md) | Search tools, batch processing, context management, example workflow | Planning analysis approach |
| [assets/report_template.md](assets/report_template.md) | Markdown report template | When generating the report |

## Quick Start Workflow

### 1. Gather Information
Ask the user (skip anything found in `desarrollos/PROJECT_REGISTRY.md`):
- **Module path(s)** and **current Odoo version**
- **Target Odoo version** (optional)
- **Focus areas** (at least one): Migrability, Code Quality, Security, Performance, Conventions, Standard Features
- **Technical level**: High / Medium / Low
- **Report length**: Extensive / Standard / Summary

If no focus areas specified, recommend Migrability + Code Quality.

### 1.1 Git freshness preflight (MANDATORY if path is a git repo)
Before analyzing, verify the local checkout is NOT stale — auditing old code yields confidently wrong findings.
```bash
git -C <repo> fetch --quiet && git -C <repo> rev-list --left-right --count HEAD...@{u}
```
If the repo is **behind upstream** (right-hand count > 0): STOP and warn the user with the branch, the N commits behind, and what those commits touch (`git log HEAD..@{u} --stat`). Recommend `git pull` before auditing. Never present an analysis of stale code as current state. (Real case: an inventory run on a 4-week-stale checkout missed a whole new module and mismarked a completed phase.)

### 1.5 Component Discovery
**Scan module [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "scan module") — invoke odoo-module-scanner on {path}. Output: models, fields, views, security, controllers, JS inventory.

### 2. Module Discovery
Run structure analyzer:
```bash
python3 scripts/analyze_structure.py <path> [--recursive]
```

### 2.5 Load Version API (if Migrability selected)
**Load API ref [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "load API ref v{version}") — read odoo-version-api-reference v18-base/{topics} + diffs/v{version}-diff.md if != v18. Output: filtered API summary with deprecated patterns.

### 3. Analyze by Category
**Pre-localiza candidatos [determinista, ~0 tokens LLM]:** `python3 scripts/find_audit_candidates.py --module <path> --dimensions <sel> --version <N> --md`
→ señales de anti-patrón por dimensión (fichero:línea). **SON CANDIDATOS, no hallazgos**: adjudica CADA uno
contra `references/gotchas.md` (el `%s` es seguro, `sudo()` puede estar justificado, lo deprecado depende de la
versión). El grep localiza; el veredicto y la severidad son juicio del LLM. Fallback: buscar con las references si falla.

Perform ONLY the analysis for user-selected categories. For each, load the corresponding reference file:
- **Migrability** -> `references/migrability-checklist.md`
- **Code Quality** -> `references/code-quality-checklist.md` (+ `references/hollow-tests.md` for hollow-test detection and latent-bug smells: fixed-width transliteration after concat, silent numeric truncation, float-money rounding, computed-field stale deps). Si el módulo trae tests y alguien te dice que pasan, **eso no es un hallazgo de calidad: es una afirmación sin verificar** → `common-skills:odoo-verify-green` (¿ejecutó?, ¿base limpia?, ¿se ha visto en rojo?). Un módulo cuyos tests nunca se han visto caer se audita como «sin cobertura demostrada», no como «cubierto».
- **Security** -> `references/security-patterns.md` (+ `references/mcp-exposure-security.md` if the module exposes a generic model/tool surface: MCP server, JSON-RPC/OpenAPI bridge, runtime-chosen model/domain; + `references/concurrency-atomicity.md` if it has tokens/quotas/counters, batch crons, or external side effects)
- **Performance** -> `references/performance-patterns.md`
- **Conventions** -> `references/odoo-conventions.md`
- **Standard Features** -> `references/standard-features.md`

### 4. Generate Report
Load `references/report-generation.md` for structure, scoring, severity levels, and adaptation rules. Use `assets/report_template.md` as the base template.

For shared ecosystem patterns, see common:references/shared/.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-verify-green | common | Dimensión Code Quality: cuando hay tests que «pasan» — comprobar que el verde es real antes de contarlo como cobertura |
| odoo-version-api-reference | common | Detect deprecated APIs |
| odoo-module-scanner | common | Component discovery for audit |
| odoo-deep-knowledge | odoo-knowledge | Consultar gotchas reales conocidos al auditar (no reinventarlos) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-flow-tester | common | Audit code issues found during testing |
| quality-gate | common | Deep code quality gate |

## Related Skills
- **odoo-module-scanner**: Pre-audit component inventory
- **odoo-module-upgrader**: Apply fixes found during audit
- **odoo-database-analyzer**: Legacy code scanning for migration planning

## Prohibitions
- Do NOT analyze categories the user did not select
- Do NOT load all reference files at once -- load on demand per category
- Pregunta solo lo que falte (path, versión, foco, nivel, longitud) tras mirar PROJECT_REGISTRY.md.

## Gotchas
- `%s` in `cr.execute` is safe (psycopg2 parameter binding) -- do NOT flag as SQL injection
- `sudo()` is normal in many contexts (mail, cron, portal) -- assess intent before flagging
- Small modules (<500 LOC) should not be penalized on architecture scores
- Always check the module's Odoo version before flagging deprecated APIs -- what's deprecated in 17+ may be correct for 14.0

### Idioma del codigo: ingles SIEMPRE (finding)

Marca como hallazgo cualquier codigo fuente que NO este en ingles: identificadores, nombres tecnicos de modelos/campos, comentarios, docstrings, external IDs o mensajes de commit en otro idioma. Aplica aunque el cliente lo pidiera en su idioma. Excepcion: los strings de cara al usuario (`string=`, labels, informes, help) pueden ir traducidos. Severidad segun origen: codigo generado/modificado en este trabajo -> bloqueante (critico); legacy preexistente no tocado -> advisory (warning, para limpieza gradual, sin bloquear). Ver `references/shared/coding-standards.md`.

### Ortografia es_ES obligatoria

Todo contenido generado en espanol DEBE usar acentos, enes y dieresis correctos (a, e, i, o, u, n, u). NUNCA generar texto sin tildes.
