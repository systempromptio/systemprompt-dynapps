<!-- verified: v17.0 @ 2026-06 (source: odoo_enterprise/17/account_budget) -->
# Account Budget — Legacy v17 model (`crossovered.budget`) — EE

> **CAMBIO DE MODELO ENTRE VERSIONES.** El módulo `account_budget` fue **reescrito por completo en v18**. Esta página documenta el modelo **v17 y anteriores** (`crossovered.budget` + budgetary positions), que es totalmente distinto al v18+ (`budget.analytic`/`budget.line`). Si migras de v17 a v18, los datos NO se mapean 1:1 — ver sección de migración. Verificado contra `odoo_enterprise/17/account_budget/models/account_budget.py`.

| Aspecto | v17 (legacy) | v18+ |
|---|---|---|
| Modelo cabecera | `crossovered.budget` | `budget.analytic` |
| Modelo línea | `crossovered.budget.lines` | `budget.line` |
| Posiciones presupuestarias | `account.budget.post` (M2M a cuentas G/L) | (eliminado) |
| Importe presupuestado | `planned_amount` | `budget_amount` |
| Importe real | `practical_amount` (compute SQL) | `achieved_amount` (vía `budget.report`) |
| Importe teórico | `theoritical_amount` | `theoritical_amount` |
| Dependencia | `account` (¡CE/EE indistinto!) | `accountant` (EE) + `purchase` |
| Reporte | computado en Python por línea | `budget.report` (SQL view) |

⚠️ En v17 `account_budget` depende sólo de `account` → **estaba disponible en Community**. En v18 pasa a depender de `accountant` → **pasa a ser Enterprise-only**.

---

## 1. Modelos v17

### `account.budget.post` (Budgetary Position) — `account_budget.py → class AccountBudgetPost` (`v17 account_budget.py L15`)
Posición presupuestaria: agrupa cuentas de mayor.
- `name` (Char, required).
- `account_ids` (M2M → account.account, tabla `account_budget_rel`).
- `crossovered_budget_line` (O2m inversa).
- Constraint `_check_account_ids` (`v17 account_budget.py L28`): no puede quedar sin cuentas.

### `crossovered.budget` (cabecera) — `account_budget.py → class CrossoveredBudget` (`v17 account_budget.py L34`)
- `_inherit = ['mail.thread', 'mail.activity.mixin']`.
- `name` (Char, required), `state` (draft/confirm/validate/done/cancel).
- `date_from` / `date_to`, `user_id`, `company_id`.
- `crossovered_budget_line` (O2m → `crossovered.budget.lines`, `copy=True`).
- Acciones de estado: `action_budget_confirm/draft/validate/cancel/done` (`v17 account_budget.py L54`). **Estado `validate` existe en v17** (lo confirma un responsable distinto); desaparece en v18 (que usa draft/confirmed/revised/done/canceled).

### `crossovered.budget.lines` (línea) — `account_budget.py → class CrossoveredBudgetLines` (`v17 account_budget.py L70`)
- `crossovered_budget_id` (M2o required, cascade).
- `general_budget_id` (M2o → account.budget.post) — la posición presupuestaria (cuentas G/L).
- `analytic_account_id` (M2o → account.analytic.account) — cuenta analítica.
- `date_from` / `date_to` (Date).
- `planned_amount` (Monetary) — importe presupuestado (input manual).
- `practical_amount` (Monetary, computed) — real ejecutado.
- `theoritical_amount` (Monetary, computed) — teórico proporcional al tiempo.
- `percentage` (Float, computed, `group_operator='avg'`) — practical/theoretical.
- `paid_date` (Date) — si está informada, el teórico es 0 hasta esa fecha y luego = planned.
- `is_above_budget` (computed).
- `crossovered_budget_state` (related stored).
- Constraint `_must_have_analytical_or_budgetary_or_both` (`v17 account_budget.py L261`): cada línea debe tener `general_budget_id` y/o `analytic_account_id`.

---

## 2. practical_amount — cómo se calcula (v17)

`_compute_practical_amount` (`v17 account_budget.py L169`). NO usa un SQL view; lanza queries UNION ALL agrupadas por (modelo, fname, fechas):

`_get_accounts_from_line` (`v17 account_budget.py L141`) decide el origen:
- **Si la línea tiene `analytic_account_id`** → fuente = `account.analytic.line`, campo importe = `amount`, agrupa por la columna de plan analítico de esa cuenta.
- **Si NO** (sólo `general_budget_id`) → fuente = `account.move.line`, campo importe = `-balance`, filtra `parent_state='posted'`, agrupa por `account_id` (las cuentas de la budgetary position).

Query (`_get_query_account_analytic_line`, `v17 account_budget.py L147`): dominio por rango de fechas + `account_fname in account_ids`; para move.line añade `parent_state='posted'` y suma `-balance`; para analytic.line suma `amount`.

⚠️ **Diferencia clave con v18**: v17 puede presupuestar contra **cuentas de mayor** (vía budgetary position) Y/O contra **analítica**. v18 SÓLO trabaja con analítica (plans). Un presupuesto v17 montado sobre cuentas G/L (sin analítica) no tiene equivalente directo en v18.

---

## 3. theoritical_amount — proporcional al tiempo (v17)

`_compute_theoritical_amount` (`v17 account_budget.py L207`). **Misma fórmula +1 día que en v18** (off-by-one en día 1):
```python
line_timedelta = date_to - date_from + timedelta(days=1)
elapsed_timedelta = today - date_from + timedelta(days=1)
theo = (elapsed / line_timedelta) * planned_amount   # entre fechas
```
- Antes de `date_from`: `theo = 0` (en v17 sí da 0 si `elapsed.days < 0`; v18 SQL da 0 también, pero el Python de v18 daba ~1 día).
- Si hay `paid_date`: teórico = 0 mientras `today <= paid_date`, luego = planned.
- Tras `date_to`: teórico = planned.

`percentage` (`v17 account_budget.py L235`): `practical/theoritical` si teórico != 0, else 0.

---

## 4. `_read_group` override (v17)

`crossovered.budget.lines._read_group` (`v17 account_budget.py L100`) maneja el set especial `{'practical_amount:sum', 'theoritical_amount:sum', 'percentage:avg'}` igual que v18: como son computados no almacenados, no pueden agregarse en SQL, así que recupera recordsets y agrega en Python. **Mismo patrón frágil que ABG13.**

---

## 5. Migración v17 → v18 (sin mapeo automático)

- `crossovered.budget` → `budget.analytic`: nombre/fechas/responsable mapean; el estado `validate` no existe en v18.
- `crossovered.budget.lines` → `budget.line`: `planned_amount` → `budget_amount`. **Pero**: las líneas v17 basadas en `general_budget_id` (cuentas G/L) NO tienen destino en v18 (que es 100% analítico). Hay que reconvertir esos presupuestos a planes analíticos.
- `account.budget.post` **desaparece** — no hay posiciones presupuestarias en v18+.
- El upgrade oficial de Odoo migra lo que puede; revisa manualmente los presupuestos contra cuentas G/L.

---

## 6. Gotchas v17

**ABG-V17a: presupuesto contra cuentas G/L sin analítica no migra a v18**
v17 permite `general_budget_id` sin `analytic_account_id`. v18 es 100% analítico. Estos presupuestos quedan huérfanos en la migración.

**ABG-V17b: practical_amount sobre move.line usa `-balance` y sólo posted**
`_get_query_account_analytic_line` filtra `parent_state='posted'` y suma `-balance` (`v17 account_budget.py L156`). Asientos en borrador no cuentan. El signo `-balance` invierte: gastos (debe positivo) salen positivos en el practical.

**ABG-V17c: estado `validate` intermedio inexistente en v18**
v17 tiene draft→confirm→validate→done. v18 cambia a draft→confirmed→revised→done→canceled. El doble control (confirm por usuario, validate por responsable) desaparece.

**ABG-V17d: theoretical da 0 antes de date_from en v17 (a diferencia del Python v18)**
v17 devuelve `theo_amt = 0.00` si `elapsed_timedelta.days < 0` (`v17 account_budget.py L225`). Coherente; pero el Python de v18 daba ~1 día por el +1 (ABG2). v19 (SQL) vuelve a dar 0. Comportamiento NO uniforme entre versiones.
