# Workflow Details — Odoo Module Creator

## STEP 0: Information Gathering [STOP]

**Pre-fill from registry:** Before asking the user, check if `desarrollos/PROJECT_REGISTRY.md` exists in the project root. If it does, read it and extract:
- **Odoo version** → from frontmatter `odoo_version`
- **Installed modules** → to know available dependencies (no need to ask for paths if symlinks exist)
- **Dependency tree** → read `desarrollos/PROJECT_REGISTRY_DEPS.md` to understand what's already available before designing the new module

Skip asking for any info already available in the registry. Still ask for anything missing.

Use `AskUserQuestion` to collect remaining info. **NEVER proceed without ALL required info.**

**Required:**
1. Target Odoo Version (14.0-19.0) — _skip if in registry_
2. Module Technical Name (`dynapps_<area>_<feature>`)
3. Functional Description (problem, features, users)
4. Architecture Approach:
   - A: "I have a clear plan" → gather models/fields/views/security
   - B: "Help me design" → STEP 0.5 Plan Mode
   - C: "Basic scaffold" → minimal structure

**Paths:**
5. Destination path
6. Odoo CE path (recommended: `$ODOO_SRC/{version}/addons`) — _skip if symlink `base/` exists_
7. Odoo EE path (if applicable) — _skip if symlink `enterprise/` exists_
8. Indaws Core path (recommended) — _skip if symlink `indaws-core/` exists_

**Multi-company (default: YES):**
If the user's prompt does NOT mention multi-company, **ask** whether the
module should be multi-company compatible (recommend YES).
If confirmed (or not explicitly excluded), classify each new model as
parent/child/shared and apply patterns from `references/multi-company.md`.

**STOP** — Wait for response.

### STEP 0.5: Plan Mode (if Option B)

Use `EnterPlanMode`:
1. Search reference paths for similar modules
2. Q&A: clarify requirements, propose approaches
3. Document: models, fields, views, security, dependencies
4. Exit with approved architecture → continue to STEP 1

## STEP 1: Analysis & Planning [STOP]

1. **Consult reference paths** — search inherited models, existing fields, patterns
2. **Read engine-patterns.md** from `/odoo-version-api-reference` — apply performance + security patterns to plan
3. **Check version** — consult `/odoo-version-api-reference` for API differences
4. **Propose plan:**
   - Version + destination confirmed
   - Directory structure
   - Models (new + extended) with field types
   - Views + menus
   - Security (ACL + record rules if needed)
   - Dependencies (verified against reference paths)
   - **Multi-company** (company_id per model: parent=default, child=related, shared=none; ir.rule per model; _check_company_auto on parents) — see `references/multi-company.md`
   - **Performance notes** (indexes, precompute, store decisions)
   - **Security notes** (check_company, field groups, record rules)

### 1a / 1b — invocaciones de subagente (literal)

Estaban en el `SKILL.md` y se movieron aquí el 2026-08-22: son 16 líneas de prompt que solo
hacen falta en el momento de delegar, y el `SKILL.md` no cabía en su límite de 150 con ellas.
**Condicionales**, no obligatorias — ver «Delegación condicional» más abajo.

**1a. Scan existing module [SUBAGENTE SI PROCEDE — si no, inline]** (only if extending an existing module)

Launch a subagent:
- **model:** sonnet · **subagent_type:** general-purpose
- **description:** "scan module for inventory"
- **prompt:** "Invoke the skill 'odoo-module-scanner' on the module at {path}. Return JSON with keys: models, fields, views, security, controllers, js_components. Only scan and report — do NOT modify files."

Store result as `SCAN_RESULT`. Do NOT design architecture without it (subagente o leído por ti).

**1b. Load API reference [SUBAGENTE SI PROCEDE — si no, inline]**

Launch a subagent:
- **model:** sonnet · **subagent_type:** general-purpose
- **description:** "load API ref for v{version}"
- **prompt:** "Invoke the skill 'odoo-version-api-reference'. For version {version}, return a summary of: field parameters, engine patterns (precompute, indexes), and gotchas relevant to module creation."

Store result as `API_RESULT`. Do NOT write code without it (subagente o fuente; nunca memoria).

**STOP** — Wait for confirmation.

## STEP 2: Module Creation

After confirmation, generate files in order:

1. `__manifest__.py` (version format: `{odoo_version}.1.0.0`)
2. `__init__.py` (root + subdirectories)
3. `models/*.py` — following class structure order from `references/dynapps-guidelines.md`
   - **Apply engine patterns:** precompute, indexes, check_company, compute_sudo — see `/odoo-version-api-reference` references/engine-patterns.md
4. `security/ir.model.access.csv`
5. `security/ir_rules.xml` — **ALWAYS** create multi-company rules for every model with `company_id` (see `references/multi-company.md`)
6. `security/*_groups.xml` (if needed)
7. `views/*_views.xml` — **`<list>` desde v18** (NO desde v19), `<tree>` en v17 y anteriores.
   Esta línea decía "`<list>` for v19, `<tree>` for v18-" hasta 2026-08-07 y ponía la frontera
   **un major tarde**: `type = fields.Selection([('tree','Tree'), …])` en
   `odoo17/odoo/addons/base/models/ir_ui_view.py:163`, `View.type` frente a `[('list','List'), …]` en
   `odoo18/…:153` (y `odoo19/…:149`). Emitir `<tree>` en un módulo v18 da un tipo de vista
   inexistente — y esta skill es la **base** del scaffolding, así que el error se replica en
   cada módulo que genere.
8. `views/*_menus.xml`
9. `data/*.xml` (if applicable)
10. `report/*.py` + `report/*.xml` (only if user requested PDF reports) — consult `references/qweb-reports/report-anatomy.md` for ir.actions.report structure, `references/qweb-reports/layout-patterns.md` for page layout, and `references/qweb-reports/css-print.md` for wkhtmltopdf-compatible CSS
11. `README.rst` — follow template from `references/readme-changelog.md`. Include all applicable sections (description, configuration, usage, credits). Write for functional users, not developers.
12. `CHANGELOG.md` — follow template from `references/readme-changelog.md`. Create initial entry with version from manifest, today's date, all features listed under `Added`, and the author.
13. **Provenance marker — stamp every file you just wrote.** See below; it is a generation gate,
    not an optional nicety.

### Provenance marker (generation gate)

Rule and per-file-type syntax: `references/shared/skill-provenance-marker.md`. What this
skill adds on top of it:

- **Who stamps what.** Whoever *writes the bytes* stamps them, with **its own** skill name and
  version. This skill stamps the skeleton files it emits (1–12 above) as
  `Made with odoo-module-creator · v<version>`. A **domain skill layered on this base**
  (`odoo-carrier-developer`, `odoo-pos-developer`, `odoo-accounting-developer`, …) stamps the
  domain files *it* writes with *its own* name and version. A base gate cannot stamp files the
  base never wrote — that assumption is why the marker had zero park coverage before 2026-08-09.
- **Never hardcode the version in prose.** Read it from the stamping skill's own `SKILL.md`
  frontmatter at generation time. A literal `v2.0.0` written into a rule drifts silently the
  first time the skill is bumped (observed: `odoo-carrier-developer` said `v2.0.0` while shipping
  2.0.2).
- **Exempt:** `.csv` and `.json` — no comment syntax. Everything else
  (`.py`/`.xml`/`.js`/`.scss`/`.po`/`.yml`) is stamped with its own comment syntax per the
  shared table. Binary assets (`icon.png`) are obviously exempt.
- **Idempotent:** if the last line already matches `Made with <this-skill> · v`, replace it;
  never append a second one. Re-generating updates the version in place.
- **Verify before presenting the summary** (part of STEP 3):

```bash
# every stamped-eligible file carries exactly one marker
find {module_path} -type f \( -name '*.py' -o -name '*.xml' -o -name '*.js' -o -name '*.scss' \) \
  -exec sh -c 'c=$(grep -c "Made with .* · v" "$1"); [ "$c" = 1 ] || echo "$c  $1"' _ {} \;
# (empty output = OK; anything printed is missing or duplicated)
```

## STEP 3: Validation & Next Steps

1. Validate: dependencies, security, version-specific APIs, naming
2. Run the provenance check above (STEP 2 §13) — report what it prints, do not silence it
3. Present file summary
3. **Suggest next skills:**
   - `/odoo-test-generator` → Generate tests for the created module
   - `/quality-gate` → Validate code quality before merge
   - `/odoo-module-scanner` → Analyze existing module before inheriting

## FAST PATH — edits pequeños (STEP 0.4)

Para evitar el coste de la ruta completa en cambios menores.

### Criterio: ¿fast path o ruta completa?
**FAST PATH** si el cambio es acotado a 1 capa y pequeño:
- 1-2 campos / un compute / un constraint / un método.
- Una vista o un `xpath` de herencia.
- Una línea de ACL o una record rule.
- Un dato / cron / sequence / mail.template.
- Un job asíncrono (`with_delay`/queue_job).
- Un wizard simple o un retoque de report.

**RUTA COMPLETA (STEP 1–3)** si es:
- Módulo NUEVO.
- Extensión sustancial: varios modelos, varias vistas, flujo nuevo, seguridad amplia,
  o tocar varias capas a la vez.
- Cualquier cosa con implicación multicompañía no trivial.

### Pasos del fast path
1. **Confirmar objetivo** (único STOP): módulo/fichero a tocar + versión (deducir del
   `__manifest__.py`; si no resuelve, preguntar). No `AskUserQuestion` completo de STEP 0.
2. **Delegar a la capa concreta** (la skill resuelve su versión vía
   `odoo-version-api-reference`/fuente; ver `references/shared/odoo-source.md`):
   | Cambio | Skill |
   |--------|-------|
   | modelo: campo, compute, constraint, herencia, relación | `orm-modeler` |
   | vista / xpath / widget | `view-designer` |
   | ACL / record rule / grupos | `security-designer` |
   | dato XML / cron / sequence / template | `data-files` |
   | job asíncrono (`with_delay`, dep `queue_job`) | `integration-builder` (receta `robustness.md`); gotchas/runner en `odoo-deep-knowledge` → `ecosystem/queue_job` |
   | chatter / tracking / actividades | `mail-integration` |
   | wizard (TransientModel) | `wizard-builder` |
   | report PDF QWeb | `qweb-report-builder` |
   | componente OWL backend | `odoo-owl-component-builder` |
3. **Aplicar** el cambio respetando convenciones (inglés, no `cr.commit`/SQL, etc.).
4. **Validar solo lo tocado**: re-`-u <módulo>` y, si aplica, un test del cambio
   (`odoo-test-generator`). NO full scan, NO regenerar README/icon/CHANGELOG.

### Qué se SALTA en fast path
Plan mode (0.5), diseño de arquitectura completo (1c), scan completo del módulo (1a/3a
salvo que necesites contexto), README/CHANGELOG/icon, multicompañía global. Si a mitad
descubres que el cambio es mayor de lo previsto → **escala a la ruta completa**.

## Delegación condicional (por qué los pasos 1a/1b/3a NO son obligatorios)

Corregido el **2026-07-31 con datos**, no por criterio: en 15 sesiones de un proyecto real esta
skill se invocó **5 veces** y hubo **4 subagentes en total** contando todos los motivos. Sus tres
pasos marcados entonces `[MANDATORY SUBAGENT]` se saltaron **las cinco veces**.

La causa no es desobediencia sino una **colisión de reglas**: el `CLAUDE.md` global del usuario
exige un pre-flight de contexto antes de cualquier fan-out
(`python3 ~/.claude/scripts/context_preflight.py --agents <N>`) y dice que *«el trabajo mecánico se
scriptea, no se reparte entre agentes»*. Escanear un módulo y consultar la API de una versión son
exactamente eso: grep y lectura. Y hay invocaciones donde el usuario prohíbe el fan-out en el propio
prompt.

**La condición, entonces:** delega si **(a)** el pre-flight está en verde **y (b)** el trabajo no es
mecánico. Si falla cualquiera de las dos, ejecuta el paso **inline y dilo en la respuesta**.

**Lo que NO cambia** — y es la regla que de verdad protegía el resultado: *no trabajes de memoria*.
Ni la API de la versión ni el contenido de un módulo se suponen. O lo trae un subagente, o lo
lees/scripteas tú. Lo que nunca vale es inventarlo.

**Por qué se corrigió en vez de insistir:** una instrucción obligatoria que se incumple siempre
enseña a ignorar las instrucciones obligatorias de esta skill — incluidas las que sí importan, como
la puerta de auto-auditoría previa a la entrega.
