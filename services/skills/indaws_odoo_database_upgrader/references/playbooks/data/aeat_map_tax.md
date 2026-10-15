# Playbook: AEAT Map Tax (España)

Cerrar mapeos de impuestos AEAT (`l10n_es_aeat_map_tax`) que quedaron abiertos
con `date_from / date_to NULL`, lo que rompe los modelos AEAT 111/115/123/303/349/390
en versiones posteriores.

## Requisitos del contexto
- `countries`: contiene "ES" (obligatorio)
- `to_version`: cualquiera (>= 13.0)

## No requiere
- features específicas

## Aplicabilidad
- Tabla `l10n_es_aeat_map_tax` existe
- Existen filas con `date_from IS NULL OR date_to IS NULL`
- Módulos AEAT con `_id` en {111, 115, 123, 303, 349, 390} declarados en `module_id`

## Flujo
1. Detectar mapeos abiertos
2. Aplicar `close_at` (por defecto `2022-12-31`, parametrizable)
3. Para cada modelo afectado, recalcular consistencia

## Helpers a invocar
```yaml
- id: aeat_map_tax
  type: indaws_migration_kit.helpers.spain.aeat.AeatMapTaxFix
  params:
    close_at: '2022-12-31'    # opcional
    models: [111, 115, 123, 303, 349, 390]   # opcional
```

## Pitfalls
- El `close_at` debe ser **antes** de la fecha del primer ejercicio v18
- Si el cliente cambió de NIF, los mapeos deben re-emitirse manualmente
