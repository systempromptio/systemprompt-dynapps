# Playbook: Reactivar vistas Studio válidas

Tras el upgrade, las vistas del módulo `studio_customization` quedan archivadas
(`active=false`). Muchas siguen siendo válidas — reactivarlas.

## Requisitos del contexto
- `features.studio: true` (obligatorio)
- `to_version`: cualquiera

## No requiere
- países específicos

## Aplicabilidad
- Existe módulo `web_studio` instalado
- Existen vistas con `key LIKE 'studio_customization%'` y `active = false`

## Flujo
1. Listar vistas Studio archivadas
2. Para cada vista, en savepoint:
   a. Activar (`active=true`)
   b. Llamar `env[model].with_context(...).get_view()` para forzar render
   c. Si OK → confirmar; si error → rollback savepoint y dejar archivada
3. Producir contadores: reactivadas / falladas
4. Failures van a `studio/pending.md`

## Helpers a invocar
```yaml
- id: studio_reactivate
  type: indaws_migration_kit.helpers.studio.reactivate.StudioReactivateValid
  params:
    validate_with_odoo: true   # exige container Odoo arriba
```

## Pitfalls
- Sin `validate_with_odoo=true` el helper solo intenta el toggle en SQL; no
  detecta errores de render reales
- Usar **savepoints** (`SAVEPOINT studio_check`) para no contaminar la BD
