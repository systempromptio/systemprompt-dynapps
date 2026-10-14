# Playbook: Custom port (mantener código en v18)

Módulo custom que el cliente quiere **conservar** funcionando en v18. Implica
portar el código fuente y volver a instalarlo.

## Requisitos del contexto
- `to_version`: cualquiera

## Aplicabilidad
- El cliente confirma que mantiene la funcionalidad
- Existe carpeta `migrated_modules/<modulo>/` o se va a crear

## Flujo
1. **Delegar a `odoo-module-upgrader`** para portar el código:
   - Manifest version → `<to_version>.0.0.X`
   - Renames API conocidos (ver `references/post-upgrade-custom-fixes.md`)
   - Decoradores `@api.multi` deprecados, `_search` con `access_rights_uid`, etc.
2. **Validar build**: `python -m py_compile` + smoke import
3. **Re-instalar / upgrade**:
   ```python
   env['ir.module.module'].search([('name','=','<modulo>')]).button_immediate_upgrade()
   ```
4. **Validar funcionalmente** con `odoo-pilot` (iterar fixes hasta clean)

## Helpers a invocar
- Skill externa: `odoo-module-upgrader`
- Skill externa: `odoo-pilot` (validación iterativa)
- Sin steps en `migration.yaml` (decisión `KEEP`)

## Pitfalls
- Reports inheritedan templates v18 (`address_layout`, BS5 classes, `tax_totals`)
- `arch_db` jsonb en v18: si el módulo manipulaba arch SQL → recodear
- `_search()` ya no admite `access_rights_uid` en v18
- `<field t-field>` deprecado → usar `<span t-field>`
