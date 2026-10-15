# Plantilla: Churn / inactividad

Entornos ordenados por inactividad humana. **Datos:** receta "Churn / inactividad" de `query-recipes.md`
(`last_activity_date`). Clasifica con los umbrales 30/60/90 de `metrics-and-thresholds.md`. Reglas en `report-rules.md`.

## Formato de salida

```
# 📉 Churn / inactividad — [DD/MM/AAAA]

## Resumen
| Estado | Entornos |
|---|---|
| ✅ Activo (≤30 d) | .. |
| 🟡 Idle (31–60 d) | .. |
| 🟠 En riesgo (61–90 d) | .. |
| 🔴 Dormido (>90 d) | .. |
| ⚪ Sin dato | .. |

## En riesgo y dormidos (acción)
| Entorno | Última actividad | Inactividad | Estado |
|---|---|---|---|
| cliente-x.indaws.cloud | 12/02/2026 | 96 d | 🔴 Dormido |
| cliente-y.indaws.cloud | 02/04/2026 | 77 d | 🟠 En riesgo |

## ⚪ Sin dato (sonda no ejecutada aún)
| Entorno |
|---|
| cliente-z.indaws.cloud |
```

## Reglas de contenido
- Por defecto lista los **🟠/🔴** (los accionables), orden por inactividad descendente. Si piden "todos", añade ✅/🟡.
- Los ⚪ **sin dato** van en su propio bloque — nunca cuentan como dormidos.
- "Inactividad" = días enteros desde `last_activity_date` hasta hoy.

**Cierre:** ofrece cruzar (p. ej. "de los dormidos, cuáles tienen licencia Enterprise caducando" o "cuáles son v18").
