# Version Compatibility Reference

## Chart Types

| Type | v18 | v19+ | v18 alternative |
|------|:---:|:----:|-----------------|
| `odoo_bar`, `odoo_line`, `odoo_pie` | YES | YES | — |
| Scorecard, Gauge | YES | YES | — |
| `odoo_funnel` | NO | YES | `odoo_bar` horizontal por etapa |
| `odoo_geo` | NO | YES | Pivot agrupado por país/región |
| `odoo_treemap` | NO | YES | `odoo_pie` por categoría |
| `odoo_sunburst` | NO | YES | Pivot multinivel |
| `odoo_waterfall` | NO | YES | `odoo_bar` con colores manuales |
| `odoo_pyramid`, `odoo_radar`, `odoo_scatter` | NO | YES | Sin alternativa directa |
| `odoo_combo` | NO | YES | Superponer `odoo_bar` + `odoo_line` |
| Carousel | NO | YES | Figuras independientes |

## Filter Types

| Type | v18 | v19+ | v18 alternative |
|------|:---:|:----:|-----------------|
| `date`, `relation`, `text` | YES | YES (enhanced) | — |
| `selection` | NO | YES | Filtro `text` o `relation` |
| `numeric` | NO | YES | Sin filtro nativo |
| `boolean` | NO | YES | Condición en dominio pivot |
| Relative periods (today, last_7_days...) | NO | YES | Fecha: año/trimestre/mes |

## Features

| Feature | v18 | v19+ |
|---------|:---:|:----:|
| `odoo://view` navigation | YES | YES |
| Conditional formatting | YES | YES |
| Dynamic tables | YES | YES |
| Zoomable charts | NO | YES |
| Fullscreen figures | NO | YES |
| Multi-company (`company_ids`) | NO | YES |
| Dashboard favorites | NO | YES |
| Search bar filters | NO | YES |
| Calculated measures | NO | YES |

## Incompatibility Response Template

```
⚠️ COMPATIBILIDAD DE VERSIÓN

Versión objetivo: v{X}

✅ Compatible:
  - [list]

❌ No disponible en v{X} (requiere v{Y}+):
  - [feature]: [explanation + alternative]

¿Cómo quieres proceder?
  A) Usar las alternativas propuestas para v{X}
  B) Cambiar a v{Y} donde todo está disponible
  C) Eliminar las funcionalidades no compatibles
```

## v18 vs v19 JSON Differences

| Property | v18 | v19 |
|----------|-----|-----|
| `version` | `21` (integer) | `"18.5.1"` (semver string) |
| `odooVersion` | `12` | **Removed** |
| Figure positioning | `x`, `y` (pixels) | `col`, `row`, `offset: {x, y}` (cell-anchored) |
| Filter `rangeType` | Required on date filters | **Removed** |
| API read method | `get_readonly_dashboard()` | `_get_serialized_readonly_dashboard()` |
| Company field | `company_id` (M2O) | `company_ids` (M2M) |

## Visualization Options by Version

- **v18**: tabla pivot, scorecards/KPI, gráficos (barras/líneas/pie), ranking Top N, lista de registros, gauge
- **v19**: tabla pivot, scorecards/KPI, gráficos (barras/líneas/pie/embudo/mapa/treemap/sunburst/cascada/pirámide/radar/dispersión/combo), carousel multi-vista, ranking Top N, lista de registros, gauge
