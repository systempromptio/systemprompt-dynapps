# Plantilla: Renovaciones Enterprise

Licencias Odoo Enterprise por caducidad. **Solo** `odoo_edition = "enterprise"`, campo `license_expiration_date`
(Community no tiene caducidad). **Datos:** receta "Licencias Enterprise que caducan" de `query-recipes.md`. Reglas en
`report-rules.md`. Umbrales en `metrics-and-thresholds.md`.

## Formato de salida

```
# 🔑 Renovaciones Enterprise — [DD/MM/AAAA]

## 🔴 Caducadas
| Entorno | Caducó | Vencida hace | Versión |
|---|---|---|---|
| cliente-x.indaws.cloud | 03/05/2026 | 46 d | 18.0+e |

## 🟠 Caducan ≤ 60 días
| Entorno | Caduca | Faltan | Versión |
|---|---|---|---|
| cliente-y.indaws.cloud | 16/07/2026 | 28 d | 19.0+e |

## 🟡 Próximas (61–180 días)
| Entorno | Caduca | Faltan | Versión |
|---|---|---|---|
| cliente-z.indaws.cloud | 30/09/2026 | 104 d | 18.0+e |
```

## Reglas de contenido
- Orden por `license_expiration_date` ascendente dentro de cada bloque.
- Si un bloque no tiene entornos, escribe "Ninguna" en lugar de la tabla.
- Ignora Community y los entornos sin `license_expiration_date`.
- **Candidato a +25%:** para Enterprise en una versión fuera de las 3 más recientes (hoy ≤ v16), marca "posible +25% al renovar (confirmar fecha de renovación)".
  Es la política del SSOT `references/shared/odoo-lifecycle-licensing.md`; no la afirmes como cerrada (el
  registro no tiene la fecha de renovación del contrato).

**Cierre:** ofrece cruces útiles para comercial (p. ej. "las que caducan y además son v18" = renovar + migrar, o
"las caducadas de hosting propio"). Para las que además están en Odoo.sh, añade la fecha en que su versión sale de
Odoo.sh (deadline forzado → palanca para traerlas a hosting Indaws). Calendario y regla del +25% en el SSOT.
