# Plantilla: Entornos — Resumen ejecutivo

Vista de una página del estado del parque de entornos cliente. Reglas comunes en `report-rules.md`.

**Datos:** receta "Foto general del parque" de `query-recipes.md` (un solo `search_read` de todos los `connected`) +
ranking de cores (`database.env.module`). Excluye el autoregistro del master. Clasifica con `metrics-and-thresholds.md`.

## Formato de salida

```
# 🗂️ Entornos inDAWS — Resumen ejecutivo
*A fecha [DD/MM/AAAA] · [N] entornos conectados*

## Versión y edición
| Serie | Entornos | Enterprise | Community |
|---|---|---|---|
| v19  | .. | .. | .. |
| v18  | .. | .. | .. |
| <v18 | .. | .. | .. |

## Hosting
| Tipo | Entornos |
|---|---|
| Indaws | .. |
| Odoo Online/SH | .. |
| Dominio propio | .. |

## Licencias Enterprise
| Estado | Entornos |
|---|---|
| 🔴 Caducadas | .. |
| 🟠 Caducan ≤60 d | .. |
| ✅ OK | .. |

## Actividad (churn)
| Estado | Entornos |
|---|---|
| ✅ Activo (≤30 d) | .. |
| 🟡 Idle (31–60 d) | .. |
| 🟠 En riesgo (61–90 d) | .. |
| 🔴 Dormido (>90 d) | .. |
| ⚪ Sin dato | .. |

## Top 5 cores inDAWS más extendidos
| Core | Entornos |
|---|---|
| indaws_base | .. |
```

## Notas
- Los porcentajes son opcionales; prioriza recuentos claros.
- Si la actividad sale toda ⚪ (sonda aún sin ejecutar en el parque), dilo en una línea y omite el bloque de churn.

**Cierre:** recuerda que pueden pedir el detalle de cualquier bloque o cruzar criterios
(p. ej. "los dormidos que además son v18", "los de hosting propio con licencia caducando").
