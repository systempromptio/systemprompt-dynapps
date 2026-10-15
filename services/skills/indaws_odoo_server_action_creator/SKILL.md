---
name: indaws-odoo-server-action-creator
version: 3.1.2
description: >
  Crea y valida acciones de servidor (ir.actions.server) en Odoo 18 y 19.
  Traduce codigo Python estandar de Odoo al sandbox de safe_eval,
  evitando los errores semanticos mas comunes (STORE_ATTR, imports, self vs record).
  Detecta si indaws_base esta instalado para desbloquear capacidades avanzadas
  (wrap_module, import_module, open, json garantizado).
  Cubre diferencias v18 vs v19 (groups_id a group_ids, nuevos triggers, validacion).
effort: medium
model: opus
---

# Odoo Server Action Creator

Skill para crear acciones de servidor correctas en Odoo 18 y 19.
El codigo de server actions se ejecuta en un sandbox (safe_eval) con reglas MUY diferentes al codigo fuente estandar de Odoo.

> **El codigo Python dentro de la server action es compatible entre v18 y v19.**
> Las diferencias estan en la DEFINICION XML (campos renombrados, nuevos triggers, validacion).

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.

## Fundamental Rule

> **El codigo de una server action NO es codigo Python de Odoo normal.**
> No hay `self` (como modelo), no hay `import`, no hay asignacion de atributos,
> no hay decoradores, no hay `return`. Es un sandbox con variables inyectadas.
> Con `indaws_base`, el sandbox se relaja: puedes importar modulos via funciones helper.

## Two Modes

| Modo | Cuando | Capacidades extra |
|------|--------|-------------------|
| **Odoo Estandar** | Sin `indaws_base` | Solo variables pre-inyectadas, sin imports |
| **Modo indaws** | Con `indaws_base` instalado | `wrap_module()`, `import_module()`, `open()`, `json` garantizado |

**Deteccion**: Preguntar al usuario o verificar si el modulo destino depende de `indaws_base`.
En proyectos Indaws/Dynapps, asumir siempre **Modo indaws**.

## Repo

`ai-claude-common` — Disponible para todos los perfiles (Dev, PM, BC, TL).

**Categoria**: Code Scaffolding & Templates

**Provenance marker (ALWAYS):** cuando emitas la acción como fichero commiteable (XML de datos / `.py`), termínalo con `Made with odoo-server-action-creator · v<version del frontmatter de esta skill>`, envuelto en el comentario del tipo, idempotente. (Si solo creas el registro vía API, no aplica.) Regla completa: `references/shared/skill-provenance-marker.md`.

## GOTCHAS
0. **Code in English** — variables, comments and docstrings in the Python action always in English regardless of client locale; only user-facing strings are localized. See `references/shared/coding-standards.md`.
1. **safe_eval blocks STORE_ATTR** — `record.field = value` **NO falla en silencio (MEDIDO v18 y v19)**: lanza `ValidationError` con el culpable escrito —`forbidden opcode(s) in "record.name = 'x'": STORE_ATTR`— o sea un diálogo en la cara del usuario, y no se escribe nada. Use `record.write({'field': value})`.
2. **import is forbidden in safe_eval** — use `env['ir.module.module']` pattern or indaws_base wrap_module.
3. **`self` doesn't exist in server actions** — use `record` (single) or `records` (multi). Common Claude mistake.
4. **v19 renamed groups_id to group_ids** — server actions referencing the old field fail with AttributeError.
5. **datetime is available but timedelta is not** — use `datetime.timedelta()` not `timedelta()`.

## Agent Routing
| Phase | Model | Why |
|-------|-------|-----|
| Detect indaws_base and version | sonnet | Mechanical check |
| Translate code to safe_eval | **opus** | Complex semantic mapping |
| Validate with checklist | sonnet | Rule-based checks |
| Generate delivery (XML/API/Studio) | **opus** | Correct code generation |

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | API validation and remote execution |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-creator | developer | Server action scaffolding in new modules |
| odoo-configurator | consultant | No-code automated actions |
| odoo-data-files | developer | When the data record is an ir.actions.server or an automation |

## References

### Propias (este skill)

| Referencia | Cuando leer |
|------------|-------------|
| `references/eval-context.md` | **SIEMPRE** - Variables disponibles (estandar + indaws_base), builtins, restricciones safe_eval |
| `references/semantic-differences.md` | **SIEMPRE** - Tabla de traduccion codigo fuente vs server action (ambos modos) |
| `references/gotchas.md` | **SIEMPRE** - 28 errores comunes (G01-G20 estandar, G21-G25 indaws_base, G26-G28 v19) |
| `references/patterns.md` | Cuando necesites recetas (P01-P30 estandar, P31-P40 indaws_base, P41-P46 v19) |
| `references/v19-changes.md` | **Si version es 19** - Diferencias v18 vs v19, nuevos triggers, validacion |
| `references/indaws-base-features.md` | Si modo indaws - Detalle de wrap_module, import_module, open, json |
| `references/workflow-checklists.md` | Durante STEP 1-3 - Checklists PRE y POST escritura |
| `references/delivery-templates.md` | Durante STEP 4 - XML, Studio, API y base.automation templates |

**OBLIGATORIO**: Leer `eval-context.md`, `semantic-differences.md` y `gotchas.md` ANTES de escribir cualquier codigo.

## Workflow

### STEP 0: Entender el Requerimiento (STOP)

Preguntar al usuario:
1. **Version de Odoo** - 18 o 19 (afecta XML, triggers, validacion - NO el codigo Python)
2. **Que debe hacer la accion** - Descripcion funcional clara
3. **Tipo de trigger**:
   - **Boton en formulario** (1 registro) -> `record` disponible
   - **Accion en listado** (multi-seleccion) -> `records` disponible
   - **Accion automatizada** (on create/write/delete) -> `record` disponible
   - **Accion planificada / cron** -> `record` y `records` VACIOS
   - **Webhook** -> `payload` disponible
   - **Al archivar/desarchivar** (v18 y v19) -> `records` disponible
   - **Al recibir/enviar mensaje** (v18 y v19) -> `records` disponible
4. **Modelo objetivo** - En que modelo se ejecuta
5. **Debe retornar algo?** - Abrir ventana, descargar PDF, abrir URL, nada
6. **Destino** - XML en modulo? Studio? API (odoo-pilot)?
7. **indaws_base instalado?** - Indaws/Dynapps -> SI

**Si version es 19**: Leer `references/v19-changes.md` ademas de las obligatorias.

### STEP 1: Disenar el Codigo

**Leer las 3 referencias obligatorias** + `references/workflow-checklists.md`.
Apply translation rules from `semantic-differences.md`. Run PRE-writing checklist.

### STEP 2: Escribir el Codigo

```python
# 1. Validaciones iniciales
if not record:
    raise UserError("No hay registro seleccionado.")

# 2. Logica de negocio (patron de patterns.md)

# 3. (Opcional) Retorno: action = {...}
```

**Reglas de formato**: Indentacion desde columna 0, sin lineas en blanco al inicio, `log()` solo para debug.

### STEP 3: Validar el Codigo

Run POST-writing checklist from `references/workflow-checklists.md`.
Cross-check against `references/gotchas.md` (all 28 entries).

### STEP 4: Entregar

Use templates from `references/delivery-templates.md` according to destination chosen in STEP 0:
- **A) XML en modulo** - Server action record + optional button
- **B) Via Studio** - Manual steps for user
- **C) Via API** - odoo-pilot create call
- **D) Automated action** - base.automation record with trigger config
