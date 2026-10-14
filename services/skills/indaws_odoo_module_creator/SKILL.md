---
name: indaws-odoo-module-creator
description: Use this skill when the user needs to create new Odoo modules (versions 14.0-19.0) following Dynapps coding standards. Activates for module scaffolding, models, views, security, multi-company, and version-specific best practices. Supports plan mode for collaborative design.
version: 4.4.3
effort: medium
model: opus
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion, EnterPlanMode]
---

# Odoo Module Creator v4.1

Create Odoo modules (14.0-19.0) with Dynapps standards, engine-aware patterns, and real subagent delegation.

## References (read BEFORE generating code)

| Ref | When |
|-----|------|
| `references/dynapps-guidelines.md` | Always — Dynapps naming, manifest format, directory structure, README/CHANGELOG conventions |
| `references/multi-company.md` | Always — company_id, ir.rule, check_company |
| `references/workflow-details.md` | First use — full step-by-step with sub-items |
| `references/readme-changelog.md` | Always — README.rst and CHANGELOG.md templates |
| `references/qweb-reports/` | Only if PDF reports — anatomy, layout, CSS, widgets. For deep QWeb/wkhtmltopdf work delegate to `odoo-qweb-report-builder` |
| `references/examples.md` | First time — 3 usage examples |
| `references/dynapps_icon.png` · `references/indaws_core_icon.png` | Always — module icon per brand profile: Dynapps logo (`dynapps_icon.png`, MD5 `9d388265…`) for standard `dynapps_*`; Indaws V2 "W" (`indaws_core_icon.png`, MD5 `5ee05477…`) for legacy Indaws Core `indaws_*`. Copy the right one to `<module>/static/description/icon.png` in STEP 2. |
| `odoo-version-api-reference` (subagent) | Always — version-specific API changes, deprecated APIs, gotchas, performance patterns, security gates. Invoked as subagent in STEP 1b (do NOT read locally). |
| `odoo-orm-modeler/references/` (read when modeling) | STEP 1/2 al modelar — `inheritance-and-relations.md` (árbol `_inherit`/`_inherits`/M2o/mixin, clases base, modelo vista SQL), `fields-and-attributes.md` (atributos por-tipo: `inverse_name`, `relation/column1/column2`, `currency_field`), `version-diffs.md` (`company_dependent`→jsonb v18, `<tree>`→`<list>`, `name_get`→`_compute_display_name`). Verificado vs fuente — no modeles de memoria. |
| Source code (symlinks) | Before inheriting → `base/{module}/models/*.py` |
| `references/shared/organic-module-design.md` | STEP 1c — 3 ejes: reusar antes de crear → montar sobre motores estándar → enchufar a flujos ERP |
| `references/shared/field-mapping-syntax.md` | STEP 2 al declarar campos — tipos y `Command.*` |
| `references/shared/skill-provenance-marker.md` | STEP 2 — regla y sintaxis del marcador de procedencia |
| `references/shared/base-domain-delegation.md` | Antes de enrutar a una skill de dominio — quién orquesta y el anti-loop guard |

**MANDATORY pre-delivery gate:** self-audit against `references/shared/coding-standards.md` §*"Anti-laxity antipatterns — fix or justify"* (hardcoded data/secrets, hollow tests, float-money, cross-company leak, commit-in-business, noupdate needing migration, incomplete `@depends`…). Each hit: **fix, or keep ONLY with inline `# JUSTIFICADO: <reason>`**; never ship an unjustified one.

## Workflow

### STEP 0 — Gather requirements [STOP]

Pre-fill from `PROJECT_REGISTRY.md` if exists. Use `AskUserQuestion` for missing: version, tech name (`dynapps_<area>_<feature>`; an `indaws_*` name selects the legacy **Indaws Core** brand profile — see `dynapps-guidelines.md`), description, approach (A=plan ready / B=plan mode / C=scaffold), paths, multi-company (default YES → classify parent/child/shared). **Wait.**

### STEP 0.4 — Scope routing (FAST PATH)

**Edit pequeño** de módulo existente (1-2 campos/métodos, una vista, una ACL, un dato) →
FAST PATH: salta STEP 0.5–3 + plan mode + README/icon/CHANGELOG; **delega directo a la capa**
(resuelve su versión sola) y valida solo lo tocado — `orm-modeler` (modelo/campo) · `view-designer`
(vista) · `security-designer` (ACL) · `data-files` (dato/cron) · `mail-integration` · `wizard-builder`
· `qweb-report-builder` · `integration-builder` (job async `with_delay`/queue_job). **Módulo nuevo o extensión sustancial → ruta completa STEP 1–3.** Detalle: `references/workflow-details.md`.

### STEP 0.5 — Plan Mode (Option B only)

`EnterPlanMode` → search refs, Q&A, document architecture, exit with approval.

### STEP 1 — Analyze & Plan [STOP]

**NEVER trabajes de memoria** — ni la API de la versión ni el contenido de un módulo: o lo trae un subagente, o lo lees/scripteas tú, pero nunca lo inventas. El **subagente es CONDICIONAL** (pre-flight verde **y** trabajo no mecánico); si no, inline y dilo. Datos y motivo: `references/workflow-details.md` §«Delegación condicional».

**1a. Scan del módulo** (solo si extiendes uno) → `odoo-module-scanner`; guarda `SCAN_RESULT`.
**1b. API de la versión** → `odoo-version-api-reference`; guarda `API_RESULT`. Los prompts
literales de las dos, en `references/workflow-details.md` §«1a / 1b». Sin esos dos resultados
no se diseña ni se escribe código: subagente, o leído por ti, nunca de memoria.

**1c. Design architecture [OPUS — this agent]**

Using `SCAN_RESULT` + `API_RESULT` + local references (dynapps-guidelines, multi-company, organic-module-design — run its 3 axes) — y si hay una **spec/esqueleto de `odoo-mockup-generator`** (mockup validado con cliente), úsala como **diseño de partida** (modelos+campos+vistas ya definidos) y valida/complétala en vez de partir de cero:
- Propose: structure, models+fields, views, security (ACL + ir.rules), deps
- Multi-company classification per model (parent/child/shared)
- Performance notes (precompute, indexes from API_RESULT)
- **Wait for user confirmation.**

### STEP 2 — Generate code [OPUS — this agent]

**Brand profile:** a tech name starting with `indaws_` is an **Indaws Core** module → legacy Indaws identity (header comment, `inDAWS:` name, Indaws author/website, `# inDAWS` deps header, Core icon). Anything else → standard **Dynapps** identity. Copy the matching manifest variant (standard vs Indaws Core) from `dynapps-guidelines.md` "Brand profile" / "__manifest__.py Format" **verbatim** — don't hand-diff.

Generate in order: manifest → `__init__` → models (with engine patterns from API_RESULT) → security (ACL + ir.rules for every company_id model) → views → menus → data → reports (if needed) → README.rst → CHANGELOG.md → **`static/description/icon.png`** → **provenance marker on every file written** (`Made with odoo-module-creator · v<this skill's frontmatter version>`, idempotent; `.csv`/`.json` exempt; a domain skill layered on top stamps ITS files with ITS own name/version — mechanics + verification command in `references/workflow-details.md` §"Provenance marker").

Icon: copy the icon for the module's brand profile (see References table) — `dynapps_icon.png` for `dynapps_*`, `indaws_core_icon.png` for Core `indaws_*`. Skip **only** if the module deliberately uses a domain-specific icon (rare — e.g. `indaws_data_import`, `indaws_rappel`).

```bash
mkdir -p {module_path}/static/description
cp {skill_path}/references/dynapps_icon.png {module_path}/static/description/icon.png   # dynapps_*
# Core indaws_* → cp {skill_path}/references/indaws_core_icon.png {module_path}/static/description/icon.png
```

For the manifest's `depends` section, follow the **convention** (`# Odoo` → `# OCA` → `# Dynapps`, plus `# inDAWS` for legacy Core deps) defined in `references/dynapps-guidelines.md`.

### STEP 3 — Control de calidad (piedra angular)

**3a. Scan created module [SUBAGENTE SI PROCEDE — si no, inline]**

Launch a subagent:
- **model:** sonnet · **subagent_type:** general-purpose
- **description:** "validate created module"
- **prompt:** "Invoke the skill 'odoo-module-scanner' on the module at {created_path}. Return JSON inventory. Check: all models in __init__.py, all company_id models have ir.rules, views use correct tag for version."

Store result as `VALIDATION_RESULT`. Review for completeness.

**3b. Quality gate [OPTIONAL SUBAGENT]** — suggest to user. Launch sonnet subagent (subagent_type: general-purpose): invoke `quality-gate` on `{created_path}`, run 5 gates, return PASS/WARN/FAIL per gate.

**3c. La prueba de flujo = el control de calidad del módulo [piedra angular · Desarrollador 2.0]** — el módulo no está terminado hasta **demostrar que hace lo que dice en el flujo real de negocio** (no que "da verde"). Subagente sonnet (subagent_type: general-purpose): invoca `odoo-test-generator` sobre `{created_path}` con el SCAN → **unit** (por componente) **+ E2E de flujo** (patrón en `odoo-test-generator/references/e2e-flow-tests.md`: enruta la cadena con `odoo-flow-coverage-planner/scripts/map_module_to_flows.py` y **aserta la contribución del módulo, no que la cadena corrió** → **Gate 4**). Escribe en `<módulo>/tests/`. Si el mapper da 0 cadenas (módulo técnico), solo unit.

Present summary to user. **Maintainer self-test:** `evals/run.sh` installs a fixture built to these standards in a throwaway Odoo (binary oracle: model+computed+ir.rule in DB) — run after changing the skill's conventions. See `evals/README.md`.

## Delegates to

Esta skill es la **BASE** del patrón base⇄dominio — contrato en `references/shared/base-domain-delegation.md`. **Orquesta quien fue invocado:** si te llamaron a ti, montas el esqueleto y **enrutas el dominio** (que NO te vuelve a pedir scaffold); si te llamó una skill de dominio, eres subrutina — entregas esqueleto y devuelves el control.

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-module-scanner | common | Before extending + after creating | Subagent **si procede**, si no inline |
| odoo-version-api-reference | common | Before writing code | Subagent **si procede**, si no inline |
| quality-gate | common | After creating | Subagent (optional) |
| odoo-test-generator | developer | After creating | Subagent (optional) |
| odoo-qweb-report-builder | developer | Non-trivial PDF reports (QWeb/wkhtmltopdf) | Subagent (optional) |
| odoo-view-designer | developer | Deep view arch (widgets, searchpanel, kanban v19, inheritance) | Subagent (optional) |
| odoo-orm-modeler | developer | Deep model layer (computes, constraints, inheritance, relations) | Subagent (optional) |
| odoo-data-files | developer | Data/demo XML, external IDs, noupdate, cron/sequence/templates | Subagent (optional) |
| odoo-mail-integration | developer | mail.thread/activity, tracking, message_post, templates | Subagent (optional) |
| odoo-wizard-builder | developer | Asistentes TransientModel (default_get, multi-paso, acciones) | Subagent (optional) |
| odoo-integration-builder | developer | Jobs asíncronos (`with_delay`/queue_job), llamadas a servicios externos | Subagent (optional) |
| carrier/pos/barcode-developer | developer | **Domain routing** — al extender delivery / POS / barcode (`depends` `stock_delivery` / `point_of_sale` / `stock_barcode`·`mrp_workorder`): scaffold aquí, dominio allí. POS: que el módulo declare su glob en `point_of_sale._assets_pos` vía `assets`. | Subagent (optional) |
| odoo-accounting-developer | developer | **Domain routing** — al extender contabilidad (`depends` `account`/`account_accountant`/`account_reports`): scaffold aquí, el dominio contable (overrides `_post`/`compute_all`, engines `account.report`, widget de conciliación, EDI) allí. | Subagent (optional) |
| odoo-deep-knowledge | odoo-knowledge | Comportamientos/gotchas que informan diseño (incl. `ecosystem/queue_job` para jobs async, `cross-module/cron` para cron) | Subagent (optional) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| dev-assessment | common | When assessment decides custom dev is needed |

## GOTCHAS

1. **Multi-company model without ir.rule** → data leak (medido) · **`check_company=True` SIN `_check_company_auto = True` en el modelo → no valida nada y no avisa** (medido v18+v19); y la guardia solo corre al escribir el enlace — `references/multi-company.md` §4
2. **Group rule without internal complement** → internal users lose access · **record rules AND vs OR** → wrong access logic
3. **Computed store=True with incomplete depends** → never recalculates · **deep depends (>2)** → massive recomputation
4. **Monetary**: el orden de declaración da igual y sin divisa el write **no falla** — lo que se pierde es el redondeo (medido) · **`required` + compute + `store` sin `precompute`** → `NotNullViolation` **ya en v18**, no al migrar a v19 (medido en las dos)
5. **N+1 in compute loop** → query per record · **sudo() in compute** → bypasses rules (use compute_sudo=True)
6. **Coding language & locale** → source in English; only user-facing strings localized (es_ES accents). See `references/dynapps-guidelines.md`.

## Absolute Prohibitions

- ❌ `cr.commit()` (framework-managed) · Direct SQL when ORM exists (breaks rules + cache) · `name_get()` v18+→`_compute_display_name` · `group_operator` v18→`aggregator` · `@api.returns` removed v19 · `record._cr/_context/_uid`→`self.env.*`
