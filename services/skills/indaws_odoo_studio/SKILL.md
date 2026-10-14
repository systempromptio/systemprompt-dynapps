---
name: indaws-odoo-studio
version: 1.1.3
effort: medium
model: opus
description: >
  Crea y gestiona personalizaciones Odoo Studio (campos, modelos, menús, vistas, zips x_studio_*)
  vía API. NOT código Python/XML → odoo-module-creator.
  Triggers: "add field", "create model", "nuevo campo", "campo studio", "import studio",
  "export studio", "studio zip", "configurar en studio", "personalizar en studio".
---

# Odoo Studio Customizations Manager

Create fields, models, menus, and views following Odoo Studio conventions.

## Prerequisites

**Configure Odoo connection via `odoo-pilot` skill (auth.sh).** See odoo-pilot for protocol selection (JSON-RPC <=v18, JSON2 >=v19).

**El destino tiene que ser Enterprise con `web_studio` instalado.** El campo `studio` de
`ir.model.data` lo añade `web_studio` (`web_studio/models/ir_model_data.py:10`, `IrModelData.studio`); en Community no
existe. Comprobado en v18: contra una CE, `create_model.py` **crea el modelo y luego falla** con
`Invalid field 'studio' on model 'ir.model.data'` — deja la instancia a medias, porque ninguno de
estos scripts es transaccional. Verificar antes, con `odoo-pilot`: buscar en `ir.module.module` un
`web_studio` en estado `installed`. Si no está, parar — no empezar y ver qué pasa.

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.
See `references/shared/field-mapping-syntax.md` for field types and command tuples.

## Scripts

| Script | Purpose |
|--------|---------|
| `create_model.py` | Create new models |
| `create_field.py` | Create single field |
| `create_fields_batch.py` | Batch field creation |
| `create_menu.py` | Create menus and actions |
| `create_page.py` | Create tabs/pages in forms |
| `add_to_view.py` | Position fields in views |
| `validate_studio.py` | Validate conventions |
| `export_customizations.py` | Export as module |
| `validate_zip.py` | Pre-import zip validator |
| `import_studio_zip.py` | Import Studio zip |

## Workflow

### Creating Customizations
1. Create model (if needed): `./scripts/create_model.py "Name" '{}'`
2. Create fields: `./scripts/create_field.py model "Label" type '{opts}'`
3. Position in view: `./scripts/add_to_view.py model form field after target '{}'`
4. Create menu (if needed): `./scripts/create_menu.py model "Label" '{}'`

### Importing Studio Zips
1. Validate offline: `./scripts/validate_zip.py customization.zip`
2. Auto-fix: `./scripts/validate_zip.py customization.zip --fix -o fixed.zip`
3. Validate online: `./scripts/validate_zip.py fixed.zip --online`
4. Import: `./scripts/import_studio_zip.py fixed.zip`

### Exporting
```bash
./scripts/export_customizations.py crm.lead --output ./my_module
```

## Key Rules

- All fields are prefixed `x_studio_` automatically (snake_case from label)
- Views use `Odoo Studio: {view_name} customization` naming (this skill's convention; Studio core uses `Default {view_type} view for {model}`)
- XML IDs: los scripts generan `studio_customization.model_{modelo}_{hex8}` y `studio_customization.field_{campo}_{hex8}`. Los helpers de Odoo usan `model_<modelo>` y `field_<modelo>__<campo>`; los scripts no siguen ese formato.
- Fields are `state=manual` (database-only, no Python files)
- Studio metadata tracked in `ir.model.data` con `studio=True` **y `noupdate=True`, pero eso hay que
  ponerlo a mano**: `web_studio` solo fuerza `noupdate` en su `write()`, nunca en el `create()`
  (`web_studio/models/ir_model_data.py:19-26`, `IrModelData.write`, con el docstring que explica justo esa protección).
  Los scripts ponen `noupdate=True`; si creas la metadata por otro camino, ponlo tú (`web_studio` solo lo fuerza en `write()`).
- Multiple fields at same position are grouped in one xpath

## Gotchas

- **Selection fields** need `selection_options` array, not a plain list
- **Monetary fields** require `currency_field` option, y tiene que **existir**: Odoo rechaza uno
  inventado (`Unknown field specified ... in currency_field`) pero **acepta en silencio no poner
  ninguno**, y ahí se pierde el redondeo sin aviso. Si el modelo no tiene divisa, créala primero
  (un `many2one` a `res.currency`) y pásala.
- **Relational fields** require `relation` option with target model — y aplica a `many2one`,
  `many2many` **y** `one2many` (el `one2many` necesita además `relation_field`). Medido en v18: un
  `many2many` creado sin `relation` **se crea, el script decía «created successfully»**, y al
  siguiente arranque el registry lo descarta con
  `ERROR ir_model: Failed to load field <modelo>.<campo>: skipped` / `KeyError: None`. El campo
  queda en `ir.model.fields` y no existe en runtime: se ve en la lista técnica y no aparece en
  ninguna vista. Desde la 1.1.0 el script se niega en vez de crearlo.
- **Los scripts no son transaccionales.** Si uno falla a mitad, lo anterior queda creado. Antes de
  reintentar, mirar qué llegó a existir.
- **Batch creation** accepts JSON array or file path; use `--add-to-view` for positioning
- **Import --force** overwrites existing records — use only after validation
- **File-by-file import** isolates errors per data file (safer for large zips)

## RULES
### NEVER
- Create x_studio_* fields without checking if the field already exists
- Modify fields created by other modules (only extend with new fields)
- Use Studio for computed fields with complex Python logic — use server actions instead
### ALWAYS
- Follow naming convention: x_studio_* for Studio UI, x_* for API/script
- Field technical names and any Python (server actions, computed code) in English regardless of client locale — only user-facing labels are localized. See `references/shared/coding-standards.md`
- Validate field after creation with verification script
- Export customizations as module for version control

## Agent Routing
| Phase | Model | Why |
|-------|-------|-----|
| Determine action type | sonnet | Classification |
| Plan field/model creation | **opus** | Naming and type decisions |
| Execute via scripts | sonnet | Script execution |
| Validate result | sonnet | Mechanical verification |

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | API operations for field/model creation |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-studio-migration | developer | Studio customization management |
| odoo-configurator | consultant | No-code configuration via Studio |

## References

- [script-usage.md](references/script-usage.md) — Full script examples with all options
- [field-types.md](references/field-types.md) — Field types, aliases, and common options
- [studio-conventions.md](references/studio-conventions.md) — Naming, DB structure, best practices
- [xpath-patterns.md](references/xpath-patterns.md) — View positioning details
- [import-studio-zip.md](references/import-studio-zip.md) — Import guide
- [import-troubleshooting.md](references/import-troubleshooting.md) — Import error catalog
- [engine-gotchas.md](references/engine-gotchas.md) — Studio engine internals: what you can/can't customize, field/model/view limits
- [improvements-proposal.md](references/improvements-proposal.md) — Roadmap
