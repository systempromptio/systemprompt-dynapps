# FASE 3 — Compile + iterate + module port

## 3.1 Compile

```bash
indaws-migrate compile --plan migration.yaml
```

Genera `steps_generated` desde `modules + fixes`. El catálogo `FIX_TEMPLATES` mapea decisiones macro a steps técnicos.

### Pre-compile: escaneo de BD (`db-detector`)

Antes de compilar, escanea la BD restaurada para detectar patrones no
declarados aún en el yaml (P1-P8) y módulos no listados en el estudio:

```python
Agent(model="sonnet", description="Escanear BD restaurada",
      prompt=<ver agent-tree.md §2>)
```

Devuelve `{installed, patterns, suggested_actions}`. Si `suggested_actions`
incluye flags no presentes en `migration.yaml.fixes`, **[STOP] preguntar**
si añadirlos antes de compilar.

## 3.2 Iterate (dry-run automatizado)

```bash
indaws-migrate iterate --dump $DUMP_PATH/original.dump --strategy interactive
```

Loop (4 fases desde 2026-05-26): restore → `apply(pre)` → `update-list` (`-u base`) → `apply(pre_markers)` → `-u all` → `apply(post)` → parsea log → fix → repeat. Persistencia por iteración en `iterations/N/`.

- **pre**: SQL prep + cleanup módulos sin código + FKs huérfanas + validators. NO toca markers.
- **update-list**: `odoo -u base --stop-after-init` registra manifests nuevos de `migrated_modules` en `ir_module_module` (Odoo 18 no tiene `--update-list` CLI).
- **pre_markers**: neutralize + SuccessorMarker + replace_with + core_replace + drop + studio + `mark_keep_modules_to_install`. Requieren manifests registrados.
- **-u all**: instala los marcados `to install` + upgrade installed.
- **post**: data migrations + stub_cleanup + smoke.

Cada iteración escribe:
- `iterations/N/state.json` — resumen
- `iterations/N/upgrade.log` — log completo
- `iterations/N/update_list.log` — log del `-u base`
- `iterations/N/apply_pre.log` / `apply_pre_markers.log` / `apply_post.log`

**Truco velocidad** (debugging del pipeline): restaurar una vez a `<db>_pristine`
y clonar con `CREATE DATABASE <db> TEMPLATE <db>_pristine` (~30s vs ~14min
pg_restore). Pre-poblar `iterations/.dump_sha` con el sha del dump para que
`iterate` skipee el restore. Patrón en `clone_pristine.sh` del workdir Tecniman.
- `iterations/N/parse.json` — errores detectados con map a playbooks

### Auto-feedback (kit v2.2.0+)

Si `iterate` deja errores parseados, `suggest` propone fixes al `migration.yaml`:

```bash
indaws-migrate suggest --workdir .                  # ver propuestas
indaws-migrate suggest --workdir . --apply --commit # aplica + auto-commit
```

Action types: `FIX_FLAG`, `SQL_TEMPLATE`, `MODULE_CHANGE`, `MANUAL_REVIEW`.
Threshold de confianza por defecto: 0.5. Flujo completo, confidence thresholds,
extender catálogo: `../developer/auto-feedback.md`.

### Sub-agentes que envuelven el loop

Cuando el log de `iterate` supera ~500 líneas, no lo leas inline:

```python
# Tras iterate fallido → parsear errores agrupados
Agent(model="sonnet", description="Parse iter logs",
      prompt=<ver agent-tree.md §3 — iter-parser>)

# Tras `suggest --apply` → validar coherencia del yaml resultante
Agent(model="sonnet", description="Validar yaml tras suggest",
      prompt=<ver agent-tree.md §4 — suggest-validator>)
```

El primero devuelve `{errors agrupados, proposed_yaml_changes, manual_review}`.
El segundo confirma que el yaml editado por `suggest` es válido y predice
si la siguiente iter pasa.

## 3.3 Module port (cuando toca)

Para cada módulo con `action: KEEP` o `custom_port`:

1. Leer `playbooks/modules/custom_port.md`
2. **Delegar a `odoo-module-upgrader` vía Agent()** (Pattern A):
   ```python
   Agent(model="opus", description=f"Port {modulo} v{from}→v{to}",
         prompt=<ver agent-tree.md §5 — module-port>)
   ```
   El sub-agente devuelve `{ported, files_changed, manifest_diff, smoke_status, todos_pendientes}`.
3. Tras el port: **validación RPC** vía `odoo-pilot`:
   ```python
   Agent(model="sonnet", description=f"RPC validate {modulo}",
         prompt=<ver agent-tree.md §Delegaciones — odoo-pilot>)
   ```
4. Si necesita data migration → escribir `sql/NNN_<descripcion>.sql`
5. Añadir step a `migration.yaml.steps_custom`:
   ```yaml
   - id: port_iban_split
     type: indaws_migration_kit.helpers.sql.RunSqlFile
     params: { file: sql/001_iban_split.sql }
   ```
6. Re-ejecutar `iterate` para validar

## Fixes catalogados (genéricos)

`references/fixes/fix_vXX_to_vXX.sql` — selección automática por rango de versión.

Clasificación obligatoria:

| Categoría | Aplica a | Condición |
|---|---|---|
| **GENERIC** | Bug en script de migración Odoo base/EE | cualquier BD |
| **OCA-CONDITIONAL** | Datos/artefactos OCA | solo si módulo installed |
| **CUSTOM-CONDITIONAL** | Módulo custom cliente | NUNCA en el catálogo del skill |

Reglas: cabecera estándar, idempotente, `IF EXISTS` para OCA-CONDITIONAL.

## Helpers disponibles desde el compiler

Ver catálogo completo: `../helpers-catalog.md`. Resumen de los que el
compiler emite automáticamente:

- `manifest_preflight` — auto-inyectado si `context.source_repos` no vacío
- `aeat_map_tax` — auto-inyectado si `context.countries` contiene `'ES'`
- `successor_marker` — auto-emitido antes de REPLACE_WITH/CORE_REPLACE con
  la lista de sucesores
- `replace_with_successors` / `core_replace_modules` — con
  `finalize_uninstall: "force"` para CORE_REPLACE (M17, Tecniman 2026-05)
- `run_sql_file` — emitido por cada módulo con `data_sql:` en su
  `ModuleDecision`
- `studio_modules` (modo `rpc_direct` por defecto si `ctx.odoo`) — crea
  campos manuales + vistas desde `studio_spec`
- `legacy_to_studio_copy` — auto-emitido tras `studio_modules` poblando
  `fields` desde `studio_spec.fields[*].legacy_column`
- `smoke_rpc` — auto-inyectado al final si `plan.odoo_url`

## Extensiones declarables en `ModuleDecision`

Dos campos opcionales en `modules.<name>` (YAML):

```yaml
modules:
  sale_order_type:
    action: CORE_REPLACE
    successor: indaws_sale_order_type
    data_sql: sql/02_migrate_sale_order_type.sql   # → emite RunSqlFile

  label_tax_pnt:
    action: STUDIO
    studio_spec:
      fields:
        - {model: account.tax, name: x_label_tax, ttype: char,
           label: "Label tax", legacy_column: label_tax}
      views:
        - {parent: account.view_tax_form, xml_id: view_tax_form_label,
           model: account.tax, arch: "<data>...</data>"}
```

## Validar el plan antes del run

```bash
indaws-migrate inspect --plan migration.yaml
```

Muestra el pipeline compilado en tabla legible + valida specs incompletos
(data_sql apuntando a fichero inexistente, studio_modules sin spec,
SuccessorMarker con lista vacía).
