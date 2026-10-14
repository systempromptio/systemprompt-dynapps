---
name: indaws-odoo-database-analyzer
version: 1.2.6
effort: high
model: opus
description: >-
  Analiza codigo Odoo legacy para estimar esfuerzo de migracion con contingencia de riesgo y generar
  presupuestos tecnicos/economicos (estrategias 1to1, Refactoring+Core, Refactoring sin Studio).
  NOT flujo de presupuestacion Indaws→Dynapps → migration-estimator.
  Triggers: "analizar modulos del cliente", "presupuesto migracion", "escanear codigo legacy",
  "complexity assessment", "migration budget", "alternativas custom a core", "informe migracion".
---

# Odoo Database Migration Analyzer

Analyze legacy Odoo codebases to estimate migration effort, applying risk-aware contingency and the appropriate migration strategy.

### Fuente de verdad para clasificación
Las reglas de clasificación (🟢 Standard, 🟡 Studio, 🟣 Core, 🔴 Custom), Golden Rules y Core Supremacy se definen en el skill **odoo-classification-guide**. Los archivos `references/migration-*.md` aplican esas reglas al contexto específico de cada estrategia de migración.

---

See `references/shared/field-mapping-syntax.md` for field types and command tuples.

## Quick Start

1. **Activate** → display prompt from `references/activation-prompt.md`
2. **Gather params** → validate against `references/required-parameters.md`
3. **Execute** → follow `references/execution-protocol.md` (STEPs 0-8)
4. **Parallelize** → use agent teams per `references/execution-mode.md`
5. **Generate report** → write to `OUTPUT_PATH` using `templates/migration-report.md`

---

## Execution Steps (summary)

| Step | What | Key Reference |
|------|------|---------------|
| 0 | Data gathering & validation | `references/activation-prompt.md`, `references/required-parameters.md` |
| 1 | Module scanning | `odoo-module-scanner` skill |
| 2 | Classification by migration type | `references/migration-1to1.md` / `migration-refactor.md` / `migration-refactor-no-studio.md` |
| 3 | Version analysis (native in target?) | `references/odoo-versions/` + `odoo-version-api-reference` skill |

### Step 0.5 — Git freshness preflight (if repo)
Antes de escanear: `git -C <repo> fetch -q && git -C <repo> rev-list --left-right --count HEAD...@{u}`. Si va por detrás del upstream → STOP, avisar (rama + N commits + `git log HEAD..@{u} --stat`), recomendar `git pull`. No analizar código desfasado (estimaciones confiadamente erróneas).

### Step 1 — Module scanning
**Scan modules [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "scan {N} modules for migration analysis"):
- **prompt:** "Scan these Odoo {source_version} modules and return a JSON inventory per module. Modules to scan: {list of module paths with tech names}. For each module return: models (name, _inherit, field count, compute methods, SQL queries), views (count by type, inherited vs new), security (ACL count, ir.rule count), controllers (route count), JS components (count). Also flag: raw SQL usage, deprecated API calls, and module complexity classification (SIMPLE/STANDARD/COMPLEX/ADVANCED)."

### Step 3 — Version analysis
**Load API diffs [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "load API diffs v{source}→v{target}"):
- **prompt:** "Load the Odoo API differences between version {source} and version {target}. Focus on: breaking changes that affect migration (removed methods, renamed fields, changed signatures), deprecated APIs that still work but should be updated, new engine patterns (precompute, aggregator, check_company changes), and OWL/JS framework changes if modules have JS components. Return a structured summary grouped by: Python API changes, XML/view changes, JS/OWL changes, and migration-critical gotchas."
| 4 | Hour calculation + risk contingency | `odoo-effort-estimator` skill, `references/ai-assistance-and-risk.md` |
| 5 | Database migration (optional) | `references/database-migration.md` |
| 6 | Functional testing (optional) | `odoo-effort-estimator` skill → `FUNCTIONAL_TESTING` |
| 7 | Unit testing (optional, 🔴 Custom only) | `references/ai-assistance-and-risk.md`, `references/execution-protocol.md` |
| 8 | Generate report | `templates/migration-report.md` |

For full step-by-step details: **Read `references/execution-protocol.md`**

---

## Deep Code Analysis

When `DEEP_ANALYSIS` is enabled, scan for deprecated APIs, security risks, and performance issues to adjust estimates. Full patterns and grep commands in `references/deep-code-analysis.md`.

**Automation:** Use **odoo-module-scanner** skill. For deprecated API patterns by version, see **odoo-version-api-reference** skill.

---

## Output Format (MANDATORY)

Write report to `OUTPUT_PATH` using the template defined in `templates/migration-report.md`. Render all content in `LANGUAGE`.

**Coste/urgencia de no migrar:** si el hosting es Odoo.sh o el cliente es Enterprise en una versión fuera de las 3 más recientes, incluye en el Executive Summary la nota de coste/urgencia (fecha de retirada de Odoo.sh + posible recargo +25% en renovación). Política y calendario en `references/shared/odoo-lifecycle-licensing.md` (no hardcodear fechas ni la regla; la fecha de renovación se confirma por cliente).

---

## Safety Rules

1. **Complete Scanner:** If N `__manifest__.py` files found, table must have N rows
2. **Strategy Consistency:** Only use solutions allowed by migration type
3. **Anti-Hallucination:** If not in Core, classify as Studio (if allowed) or Custom
4. **Mathematics:** Always round UP to integers, apply correct risk %
5. **Persistence:** Always write file to `OUTPUT_PATH` and confirm completion
6. **Validation:** Verify code findings are used to adjust estimates
7. **Emoji Mandatory:** Every row in Solution column MUST include the emoji prefix (🟢/🟡/🟣/🔴)
8. **Unit Testing Scope:** ONLY estimate unit tests for `🔴 Custom` modules
9. **AI Factor:** ALWAYS apply AI assistance factors to unit testing hours

### Ortografía es_ES obligatoria

Todo contenido generado en español (comentarios, documentación, descripciones, informes) DEBE usar acentos, eñes y diéresis correctos (á, é, í, ó, ú, ñ, ü). NUNCA generar texto sin tildes. Palabras que fallan con frecuencia: análisis, valoración, información, configuración, sección, descripción, módulo, método, categoría, período, diagnóstico, fabricación, producción, corrección, inversión, antigüedad, rotación, conciliación, tránsito, automático, específico, también, según, más, además, reunión, gestión, solución, aplicación.

---

## GOTCHAS
1. **Line count != complexity** — a 50-line module with raw SQL is harder to migrate than a 500-line CRUD module.
2. **Deprecated APIs in source don't always fail** — some work with deprecation warnings. Check target version.
3. **OCA modules may have been patched locally** — compare installed code vs OCA repo to detect drift.
4. **Studio fields are invisible in code analysis** — they live in ir.model.fields, not in .py files.
5. **Risk contingency is multiplicative** — high-risk module (1.5x) with version jump (1.3x) = 1.95x, not 1.8x.

## Agent Routing
| Phase | Model | Why |
|-------|-------|-----|
| Module scanning | sonnet | Delegate to odoo-module-scanner |
| Code analysis and risk scoring | **opus** | Deep reasoning |
| Effort estimation | sonnet | Delegate to odoo-effort-estimator |
| Report generation | **opus** | Synthesis of findings |

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-version-api-reference | common | Version diffs for API change detection |
| odoo-module-scanner | common | Component inventory per module |
| odoo-effort-estimator | common | Hours estimation and contingency |
| odoo-classification-guide | common | Reglas de clasificacion Standard/Studio/Core/Custom, Golden Rules, umbrales |
| odoo-deep-knowledge | odoo-knowledge | Cross-reference comportamientos/gotchas al analizar código legacy |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-creator | developer | Pre-creation analysis of existing code |
| odoo-code-auditor | developer | Migration complexity context |
| odoo-migration-pitch | sales | Technical migration budget |
| odoo-rescue-pack | developer | Gap de versión en el diagnóstico de rescate |
| migration-estimator | migration-lab | Code/effort analysis engine (1-to-1 profile) for the migration estimate |

## Reference Files

| File | Purpose |
|------|---------|
| `references/activation-prompt.md` | Full activation output message to display |
| `references/required-parameters.md` | All parameters with types, options, defaults |
| `references/execution-mode.md` | Agent teams parallel distribution + sequential fallback |
| `references/execution-protocol.md` | Complete STEP 0-8 execution details |
| `references/ai-assistance-and-risk.md` | AI factors for unit testing + risk contingency matrix |
| `references/deep-code-analysis.md` | Grep patterns for deprecated APIs, security, performance |
| `odoo-effort-estimator` skill | Hour calculation base (includes FUNCTIONAL_TESTING and UNIT_TESTING categories) |
| `references/migration-1to1.md` | 1:1 migration strategy |
| `references/migration-refactor.md` | Full refactoring strategy |
| `references/migration-refactor-no-studio.md` | No-Studio strategy |
| `references/database-migration.md` | DB upgrade calculation |
| `references/odoo-versions/version-impact-summary.md` | Effort multipliers by module type |
| `odoo-version-api-reference` → `diffs/v*-diff.md` | Detailed API changes, breaking changes per version (external skill) |
| `templates/migration-report.md` | Mandatory report output template |
| `references/shared/odoo-lifecycle-licensing.md` | Calendario de retirada de Odoo.sh + política del +25% en renovación Enterprise (coste/urgencia de no migrar) |
