# [MIG-V{Y}] {modulo_tecnico} (Capa {N} — {grupo})

**Modulo**: `{modulo_tecnico}` | **Version**: {X.0} → {Y.0}
**Horas planificadas**: {rango}h | **Prioridad**: {Alta/Normal} ({justificacion: bloquea X / independiente})
**Etiquetas**: `migracion`, `v{Y}`, `capa-{N}`

---

## Context

### What it does

{Functional description of the module — what it does for ANY user, not for a specific client.
Describe: which models/views/JS it has, which flows it covers, where the end user sees it.
The module is Indaws Core — the context is generic, not client-specific.}

### Why it must be migrated

{Why it is needed in v{Y}: what breaks if not migrated, which modules depend on it,
which functionality is blocked. Speak about the dependency chain, not a client.}

**Decision**: {Migrate / Simplify / Refactor} — {1-line justification}

## Dependencies

- **Depends on (indaws)**: `{module_1}` {migrated/pending}, `{module_2}`
- **Depends on (Odoo)**: `{odoo_module_1}`, `{odoo_module_2}`
- **Unblocks**: `{waiting_module_1}`, `{waiting_module_2}`

## Known v{Y} breaking changes

{Verified changes in Odoo v{Y} source code that affect this module. Cite `file:line` of the
source — not from memory (NEVER #8 / gate Step 2.5). The central mechanism the module relies on
must be traced to the target-version source before writing the task.}

- `{api/field/model}` — {what changed} → {expected action} (`{addon/file.py:line}` @ v{Y})
- `{api/field/model}` — {what changed} → {expected action} (`{addon/file.py:line}` @ v{Y})

{If none: "No breaking changes identified — verify anyway during analysis."}

## Module inventory

| Component | Files | Detail |
|-----------|-------|--------|
| Python | {N} | {summary: models, fields, methods} |
| XML | {N} | {summary: views, data, reports} |
| JS/OWL | {N} | {summary or "—"} |
| Security | {N} | {summary or "No custom ACLs/rules"} |
| Tests | {N} | {summary or "None — write as part of task"} |
| i18n | {N} | {files} |

## Scope

### Gate 1 — Compatibility analysis

- Review all models, views, JS and assets against v{Y}
- Verify `_inherit` chains are still valid: parent fields/methods may have changed
- Document each incompatibility in table: file | what breaks | action
- Ref: Odoo v{Y} source at `$ODOO_SRC/odoo{Y}`

**Skills**: `odoo-module-scanner` to inventory components, `odoo-version-api-reference` for API changes

### Gate 2 — Migration scripts (if needed)

- Determine if there are schema changes, renamed fields, data to transform
- If yes: create `migrations/{Y}.0.1.0.0/pre-migrate.py` and/or `post-migrate.py`
- Requirements: idempotent, with logging, documented
- If not needed: document the reason in the commit

**Skills**: `odoo-module-upgrader` for migration script patterns and known v{Y} fixes

### Gate 3 — Code

- Manifest: version `{Y}.0.1.0.0`, review depends, license, assets
- Python: replace deprecated APIs, verify imports, respect `super()` chain
- XML: adapt xpaths, verify structural changes (`tree→list`, `kanban-box→card`)
- JS/OWL (if applies): `.include()` → `patch()`, verify services and imports

**Skills**: `odoo-module-upgrader` for transformations, `odoo-module-creator` for Indaws/Dynapps standards, `indaws-core-developer` if it touches the 5-layer architecture

### Gate 4 — Tests

- Minimum 1 test per model with custom logic (computes, constrains, onchange, business methods)
- If module had no tests in v{X} → write them as part of this task
- Pattern: `@tagged('post_install', '-at_install')`, base `TransactionCase`
- Verify: `./odoo-bin -d testdb --test-tags /{module} -i {module} --stop-after-init`

**Skills**: `odoo-test-generator` for test suite generation following official Odoo v{Y} patterns

## Acceptance criteria

- [ ] Compatibility analysis documented
- [ ] Migration scripts created or justification why not needed
- [ ] Installs cleanly in empty v{Y} DB
- [ ] Works in DB migrated from v{X}
- [ ] Tests pass green
- [ ] No TODOs, temporary hacks, or commented-out v{X} code
- [ ] PR with description of what was migrated and decisions taken

**Final validation**: `quality-gate` to verify code ready for merge, `odoo-code-auditor` if module is complex

## Resources

- Original v{X} module: `$PROJECTS_ROOT/indawsodoo/{X}/{modulo_tecnico}`
- Odoo v{Y} source: `$ODOO_SRC/odoo{Y}`
- Already-migrated modules (reference): `$PROJECTS_ROOT/indawsodoo/{Y}/`

## Constraints

{Module-specific constraints + WHY. If none, remove this section.}

---

## Skills summary by gate

| Gate | Skill | For what |
|------|-------|----------|
| 1. Analysis | `odoo-module-scanner` | Inventory module components |
| 1. Analysis | `odoo-version-api-reference` | Consult API changes v{X}→v{Y} |
| 2. Scripts | `odoo-module-upgrader` | Migration script patterns, v{Y} fixes |
| 3. Code | `odoo-module-upgrader` | Transformations: manifest, Python, XML, JS |
| 3. Code | `odoo-module-creator` | Indaws/Dynapps code standards |
| 3. Code | `indaws-core-developer` | 5-layer architecture, indaws dependencies |
| 4. Tests | `odoo-test-generator` | Generate tests with official v{Y} patterns |
| Validation | `quality-gate` | Verify code ready for merge |
| Validation | `odoo-code-auditor` | Deep audit (complex modules) |

---

## Output final

Generar la tarea como **JSON API payload** listo para crear via `odoo-pilot` (ver `templates/migration-task-api.json`).

La informacion se distribuye en campos Odoo (ver `references/odoo-task-field-mapping.md`):

| Destino | Contenido |
|---------|-----------|
| `description` (HTML) | Context + inventory + dependencies + acceptance criteria + resources |
| `x_studio_task_testing` (HTML) | Testing plan |
| `x_studio_explicacin` (text) | Breaking changes / gotchas |
| `xx_checklist_ids` | Pasos del alcance como checklist items |
| Campos nativos | `allocated_hours`, `x_studio_task_difficulty`, `priority`, `tag_ids`, etc. |

Defaults fijos al crear:
- `stage_id`: `1008` (A planificar)
- `x_studio_is_planned`: `false`
- `x_studio_fecha_de_revision`: `false` (vacio)
- `x_studio_review_priority`: `"1"` (1 estrella)
- `tag_ids`: `[[6, 0, [97, 83, 54]]]` (MIGR + inDAWS Core + Desarrollo)

El `description` se genera en **ingles** (idioma del equipo de desarrollo).
El nombre de la tarea: `[MIG-V{Y}] {modulo_tecnico} (Capa {N} — {grupo})`
