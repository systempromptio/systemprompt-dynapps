# Support Tools

Auxiliary scripts available on demand. Not main skill functionalities but can be invoked when the user requests them.

## Create Migration Tasks

**Trigger:** "Crea las tareas de migración", "Monta las tareas"

**Script:** `scripts/create_migration_tasks.py` — datos operativos en `scripts/migration_spec.json`.
La orquestación completa (inputs, fechas, Knowledge) la hace el skill `migration-project-creator`; este script es el motor.

Dos carriles según `--profile`:
- **`internal`** (default): 18 tareas — 8 `[MIG]`/`[INFRA]` (análisis, despliegue, módulos [cada uno 1:1/Core/Studio], BD, testing, validación, go-live, soporte) + 8 gates (0-7) + 2 gestión.
- **`upgrade-lab`** (Dynapps): 8 tareas — 4 `[MIG]` (análisis e informe, código, BD, corrección+re-migración) + 3 gates A/B/C + 1 gestión. Fuerza 1:1, sin INFRA/Studio.

```bash
python3 scripts/create_migration_tasks.py \
    --analysis /path/to/analysis.md \
    --url "$ODOO_URL" --db "$ODOO_DB" --user "$ODOO_USER" --key "$ODOO_KEY" \
    --project-id {id} --version "$ODOO_VERSION" \
    --client-name "{client}" --from-version {X.0} --to-version {Y.0} \
    --profile {internal|upgrade-lab} \
    --date-start {YYYY-MM-DD} --date-golive {YYYY-MM-DD} \
    --project-size medium \
    --with-checklists --with-gates \
    --dry-run  # Quitar para ejecutar
```
(Upgrade Lab usa `--analysis-hours --db-hours --fix-hours`; interno usa además `--testing-hours --infra-hours --validation-hours --golive-hours --support-hours`.)

**Analysis file format:** Markdown table with columns: Module, Strategy, Notes, Hours, Risk, Dependencies.

**Requires:** API + local analysis file.
**Output template:** `templates/task-creation-report.md`
**Rule:** ALWAYS execute `--dry-run` first and CONFIRM with user before creating.

## Dependency Analysis

**Script:** `scripts/analyze_task_dependencies.py`

```bash
python3 scripts/analyze_task_dependencies.py \
    --url "$ODOO_URL" --db "$ODOO_DB" --user "$ODOO_USER" --key "$ODOO_KEY" \
    --project-id {id} --version "$ODOO_VERSION"
```

**Requires:** API.
**Used by:** F5 (Technical Planning) in API mode.

---

## Naming Convention and Tags

> Documented in Article 2602

### Task Names
| Type | Format | Example |
|------|---------|---------|
| Module migration | `[MIG] Migración de módulos` | Phase 2 — module checklist |
| DB migration | `[MIG] Migración de base de datos` | Phase 3 — iteration table |
| Functional testing | `[MIG] Testing funcional` | Phase 4 — flow checklist |
| Server deployment | `[INFRA] Despliegue de servidor` | Phase 1 — target server data |
| Infra validation | `[INFRA] Validación de infraestructura` | Phase 5 — pre go-live checks |
| Gate task | `[GATE] Gate {N} — {name}` | `[GATE] Gate 2 — Código Migrado` |
| Management task | No prefix | `Reuniones`, `Gestión del proyecto` |

### Tags (project.tags)
| Tag | Usage |
|-----|-------|
| **Migration** | All `[MIG]` and `[INFRA]` tasks |

Strategies (Standard, Studio, Core, Custom) are indicated in the module task checklist, not as separate tags.

**Rule:** sin subtareas; la estructura de tareas y sus conteos por carril (interno / upgrade-lab) los define `scripts/migration_spec.json` + el motor.

### Known Tag IDs (indaws instance)
| Tag | ID |
|-----|-----|
| Migration | 80 |
| Studio | 55 |
| inDAWS Core | 83 |
| Standard | (resolve via API) |
| Custom | (resolve via API) |
