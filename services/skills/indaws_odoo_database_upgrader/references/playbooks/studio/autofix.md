# Playbook: AutoFix vistas Studio archivadas

Vistas Studio que NO pasaron `reactivate` por referenciar campos renombrados.
Aplicar catálogo de renames (`catalogs/studio_renames`) y reintentar.

## Requisitos del contexto
- `features.studio: true` (obligatorio)
- `from_version`, `to_version`: ambos requeridos (para seleccionar renames)

## Aplicabilidad
- Vistas Studio aún archivadas tras `studio_reactivate`
- Existen patterns en `catalogs.studio_renames.get_catalog(from, to)`

## Flujo
1. Cargar catálogo de renames aplicable
2. Para cada vista archivada:
   a. Obtener `arch_db` (jsonb en v18, text en versiones < 18)
   b. Aplicar regex de renames (model-aware si el catálogo lo especifica)
   c. Re-intentar reactivate con el arch_db transformado
3. Las vistas que sigan falladas → `studio/pending.md`

## Helpers a invocar
```yaml
- id: studio_autofix
  type: indaws_migration_kit.helpers.studio.autofix.StudioAutoFix
  params: {}    # from_version/to_version se auto-inyectan del contexto
```

## Pitfalls
- **v18 arch_db es jsonb**, no text → usar `jsonb_object_agg` para reemplazos
- Renames *model-aware* evitan falsos positivos (ej. `<field name="date"`
  solo en `account.invoice.report`)
- Hay vistas con XPath roto post-upgrade que **no son autofixables** — son las
  que acaban en `studio/pending.md`
