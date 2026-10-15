# Ejemplo de review — Margen por línea (continuación del caso del arquitecto)

Cierra el bucle del ejemplo de `odoo-solution-architect` (margen por línea en `sale.order.line`).
El dev implementó y notificó el PR. Dos escenarios: correcciones y, tras arreglar, merge.

---

## Entrada

- PR: `#412` en el repo del cliente, rama `feature/sale-line-margin`.
- MD del arquitecto (contrato): campo `margin_percent` (`# CONTRACT`), `_compute_margin_percent` con
  `depends` fijo, 3 criterios de aceptación, DoD = unit + e2e en verde.

## Ronda 1 — Correcciones

**Step 1 (Docker)**: `docker-deployment` levanta Odoo 18 + dump del cliente, checkout de
`feature/sale-line-margin`, `-u sale_line_margin_percent`. Instala OK (sin errores de arranque).

**Step 3 (contrato)**: OK — firma y `depends` intactos, sin deps nuevas, core no tocado.

**Step 4 (tests en el Docker)** — bloqueante:

| Criterio | Test | Pasa |
|----------|------|------|
| coste 50 / precio 100 → 50% | `test_margin_basic` | ✅ |
| precio 75 → 33.3% | `test_margin_recompute` | ✅ |
| precio 0 → 0% (no div/0) | — (no existe) | ❌ |
| Flujo e2e pedido→línea→margen | — (no existe) | ❌ |

**Decisión**: ❌ Correcciones. Bloqueantes:
1. Falta test del criterio "precio 0 → 0.0" — riesgo de división por cero sin cubrir.
2. Falta flujo end-to-end. DoD innegociable.

Feedback publicado en el PR con `templates/pr-review-feedback.md`. Task marcada "En correcciones".

## Ronda 2 — Merge

El dev añade `test_margin_zero` y un tour e2e.

- **Docker**: re-levantado con la rama actualizada; `-u` OK.
- **Tests en el Docker**: 3/3 criterios unit + e2e, verde real (output capturado del contenedor).
- **Impacto**: registros existentes → el dev añadió recompute en `post_init_hook`; verificado en el
  Docker (las líneas del dump del cliente muestran margen tras el upgrade).
- **Contrato**: intacto. `/review` / `quality-gate`: sin bloqueantes.

**Decisión**: ✅ Aprobado → merge (`templates/merge-approval.md`). Docker tumbado. Task cerrada con
constancia del review vía `odoo-pilot`.

> Nota: si el criterio "precio 0" no hubiese estado en el MD original, el fallo sería **del contrato**
> (arquitecto), no del dev → se escala a `odoo-solution-architect` para añadirlo, no se penaliza al dev.
