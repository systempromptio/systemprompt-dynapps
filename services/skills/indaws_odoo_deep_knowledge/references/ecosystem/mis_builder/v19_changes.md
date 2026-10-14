<!-- verified: v19.0 @ 2026-07 (OCA mis-builder 19.0.1.0.1 — diffs verificados vs $OCA_SRC/v18 y $OCA_SRC/v19) -->
# OCA mis_builder — Cambios v19

Manifest v19: `version 19.0.1.0.1`. La arquitectura (AEP, KPIs, AccountingNone, eval sandbox, fuentes de
columna) **no cambia**. `accounting_none.py` idéntico a v18. Diffs relevantes son de **API de framework**,
no de semántica de reporting:

## 1. Migración a `odoo.fields.Domain` (`aep.py` y modelos)

v18 usaba `from odoo.models import expression` con `expression.OR(...)`, `expression.AND(...)`,
`expression.normalize_domain(...)`. **v19** migra a la nueva API de dominios:
`from odoo.fields import Domain` → `Domain.OR([...])`, `Domain.AND([...])`, y desaparece
`normalize_domain` (los `Domain` ya vienen normalizados). Además pequeños ajustes de tipos
(`tuple(account_ids)` → `list(account_ids)` en el dominio, `aep.py:get_aml_domain_for_expr`). **Impacto:**
cualquier override custom del AEP o de `_get_additional_move_line_filter` que use `expression.OR/AND`
debe migrarse a `Domain.OR/AND` en v19. Es el cambio general de Odoo 19 (dominios como objetos `Domain`).

## 2. `mis_safe_eval.py` (ajustes menores)

Cambios de import/compatibilidad en el wrapper de `safe_eval` (~15 líneas). El contrato se mantiene:
`#NAME` / `#DIV/0` / `#ERR` (MISG8). Verifica la ruta de `_SAFE_OPCODES`/`_BUILTINS` si extiendes el eval.

## 3. `mis_report.py` / `mis_report_instance.py` (~40 líneas cada uno)

Cambios acompañando la migración v19 (API de dominios, `_read_group`, ajustes de campos). Sin cambios en
la semántica de KPIs, modos AEP, ni fuentes de columna. El comportamiento documentado en `gotchas.md`
(MISG1–MISG18) sigue vigente en v19.

## 4. `_read_group` — ya en v18

Nota: mis_builder **ya usa `_read_group`** en v18 (`mis_builder/models/aep.py:448`, `AccountingExpressionProcessor.do_queries`), no `read_group`. No hay migración
pendiente de ese punto al pasar a v19 (a diferencia de otros módulos). La firma v19 de `_read_group`
(groupby/aggregates posicionales) es la misma que mis_builder ya consume.

## Resumen de riesgo de migración
Bajo-medio y **mecánico**: es sustituir la API de dominios (`expression.*` → `Domain.*`) en el código que
extienda mis_builder. La configuración de informes (plantillas, KPIs, expresiones) es **portable tal cual**.
