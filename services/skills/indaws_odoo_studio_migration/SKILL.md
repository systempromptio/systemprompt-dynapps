---
name: indaws-odoo-studio-migration
description: >
  Migra modulos de Odoo desde codigo tradicional Python/XML a implementaciones No-Code
  usando Odoo Studio (V18/V19). Use when: migrate module to studio, migrar modulo a studio,
  convert to no-code, convertir a no-code, python to server action, legacy to studio,
  migrate fields to studio, crear script migracion studio.
version: 4.1.1
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, WebFetch, AskUserQuestion]
effort: medium
model: opus
---

# Odoo Studio Migration

Migra modulos de Odoo desde codigo Python/XML a Studio No-Code (V18/V19).
Todas las respuestas en espanol. Codigo tecnico en ingles.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Scan Python module | sonnet | Component inventory |
| Map to Studio equivalents | **opus** | Decide what converts vs stays as code |
| Generate Studio config | **opus** | Server actions, automated actions |
| Validate | sonnet | Check completeness |

## Referencias (leer segun la fase)

| Referencia | Cuando leer |
|---|---|
| `references/engine-knowledge.md` | **SIEMPRE PRIMERO** — como funciona Studio por dentro (3 capas, 13 opciones modelo, XPath priority=99, safe-eval scope, export requisitos) |
| `references/gotchas.md` | **ANTES de implementar** — 20 errores reales (SQL views, M2M collision, XPath silent skip, no auto-index) |
| `references/patterns.md` | Fase 2: mapeo codigo→Studio (onchange→AA, button→SA, with_delay→trigger, XML ID portable) |
| `docs/v18_changes.md` | Fase 2: cambios V18 (tree→list, attrs→inline, statusbar→badge) |
| `docs/api_reference.md` | Fase 3: campos ir.model.fields, base.automation, server actions |
| `docs/troubleshooting.md` | Cuando falla algo — catalogo de errores+soluciones |
| `docs/export_verification.md` | Fase 4: verificacion pre-export (studio=True, noupdate=False) |
| `docs/examples.md` | Ejemplos completos de migracion |

## Scripts

| Modulo | Clase | Descripcion |
|--------|-------|-------------|
| `scripts/connection.py` | `OdooConnection` | Conexion con auth automatica (JSON2/JSON-RPC) |
| `scripts/operations.py` | `OdooOperations` | CRUD: search, read, create, write, unlink |
| `scripts/studio.py` | `StudioMigration` | Crear campos, vistas, acciones, menus, registro Studio |
| `templates/migration_template.py` | — | Template completo de migracion |

## Workflow

### Fase 0: Recogida de Informacion [STOP]

1. Preguntar: path modulo, version origen/destino, credenciales Odoo destino
2. **Scan module [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "scan module") — invoke odoo-module-scanner on {path}. Output: models, fields, views, security, controllers, JS inventory.
3. Evaluar viabilidad: si hay **logica no trivial**, NO migrar a Studio — mantener como modulo gobernado (criterio en `odoo-classification-guide`). El tamano (>20 campos / >50 lineas Python) es solo una señal de alarma, no el criterio.
4. Presentar plan y **esperar confirmacion**

### Fase 1: Analisis del Origen

1. Modelos: `_name` (nuevo) vs `_inherit` (heredado)
2. Campos: nombre, tipo, relaciones, required, readonly
3. Logica: compute → Server Actions, onchange → Automated Actions
4. **Detectar bloqueantes** (leer `references/gotchas.md`): SQL views, imports en codigo, deep inheritance

### Fase 2: Diseno Studio

Leer `references/patterns.md` + `docs/v18_changes.md`.

Decisiones clave:
- Modelo nuevo → usar opciones built-in (13 opciones, ver `engine-knowledge.md` seccion 2)
- Logica con `import` → NO migrable a SA, mantener como codigo
- Vistas complejas → evaluar si auto-generadas cubren el caso (8 tipos, ver `engine-knowledge.md` seccion 3)

### Fase 3: Implementacion

**Orden de creacion CRITICO:**
```
1. Verificar modulos instalados
2. Crear modelos (state='manual') + registrar en Studio
3. Crear campos M2O hijos → campos basicos → campos O2M padres
4. Crear Server Actions
5. Crear vistas (referencian IDs de acciones)
6. fix_button_action_ids() (XML IDs portables)
7. Crear Automated Actions
8. Crear menus
```

### Fase 4: Export y Verificacion [STOP]

```python
status = studio.verify_all_exportable()
if status['not_exportable'] > 0:
    studio.fix_all_exportable()
studio.fix_button_action_ids()  # IDs numericos → XML IDs
```

**3 requisitos de exportacion:**
- `module = 'studio_customization'`
- `studio = True`
- `noupdate = False`

### Fase 5: Post-migracion

1. Verificar que TODAS las vistas Studio tienen efecto visible (gotcha #8: XPath silent skip)
2. Crear indices SQL para campos M2O criticos (gotcha #18: no auto-index)
3. Auditar campos computed con codigo Python si destino es v19 (gotcha #20)

## GOTCHAS
1. **SQL views intocables** — `table_kind() != 'Regular'` → detectar en Fase 1 y excluir
2. **XPath rotos silenciosos** — personalizaciones desaparecen sin error post-upgrade
3. **Safe-eval sin import** — codigo con `import` = NO migrable a Server Action
4. **studio=False = invisible para export** — SIEMPRE `verify_all_exportable()`
5. **No auto-indexing** — M2O sin indice = full scan, crear manualmente
6. **Computed fields** — can't map 1:1 to Studio, use server actions for complex compute
7. **_sql_constraints** — not supported in Studio, keep as code
8. **View inheritance priority** — changes in Studio, test overrides
9. **x_studio_* vs regular fields** — naming collision if both exist
10. **Automated action triggers** — only on_create/on_write/on_unlink, not on_time in all versions

Full 20 gotchas in `references/gotchas.md`.

## Reglas

**NUNCA:** campos sin prefijo `x_` | `import` en server actions | botones con ID numerico | migrar sin verificar exportabilidad (studio=True + noupdate=False)

**SIEMPRE:** leer `references/engine-knowledge.md` primero | campos ANTES de vistas | registrar en ir.model.data con studio=True | `fix_all_exportable()` pre-export | indices M2O post-migracion

For shared ecosystem patterns, see common:references/shared/.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-studio | common | Studio conventions and scripts |
| odoo-module-scanner | common | Inventory code module before converting |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| dev-assessment | common | When assessment recommends Studio over custom |
| odoo-database-upgrader | developer | Migrating Studio customizations during a DB upgrade |
| odoo-rescue-pack | developer | Estructura de records Studio del módulo `odoo_studio` en el rescate |
