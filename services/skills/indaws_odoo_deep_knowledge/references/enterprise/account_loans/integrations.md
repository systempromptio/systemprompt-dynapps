<!-- verified: v18.0 @ 2026-06 -->
# account_loans — Integrations — EE

Cómo `account_loans` se conecta con otros módulos. Foco en bridges existentes (account.move, account.asset, diarios, account.report/return) y en el diagrama del cuadro de amortización → asientos.

---

## 1. Bridges oficiales

### `account.move` — asientos de amortización (bridge principal)

`models/account_move.py` extiende `account.move`:

- `generating_loan_line_id` (M2o → account.loan.line, `ondelete='cascade'`, `copy=False`, `readonly`, `index`) — `account_loans/models/account_move.py:7-15`, `class AccountMove`.
- `loan_id` (related `generating_loan_line_id.loan_id`) — `account_loans/models/account_move.py:16`, `class AccountMove`.
- `is_loan_payment_move` (Boolean) — distingue el asiento de pago de los de reclasificación — `account_loans/models/account_move.py:17`, `class AccountMove`.

**Override `_post`** (`account_loans/models/account_move.py:19-25`, `AccountMove._post`): tras postear, si el move pertenece a un loan y TODAS las líneas del loan están pagadas (`is_payment_move_posted`) o son anteriores a `skip_until_date`, marca el loan `state='closed'`. Este es el mecanismo de cierre automático del préstamo.

**`open_loan()`** (`account_loans/models/account_move.py:27-36`, `AccountMove.open_loan`): desde el asiento, smart-link al loan origen.

Cada `account.loan.line` genera (en `action_confirm`, `account_loans/models/account_loan.py:198-289`, `AccountLoan.action_confirm`):
- 1 move de **pago** (`is_loan_payment_move=True`): principal + interés.
- 1 move de **reclasificación** LT→ST (salvo última línea).
- 1 move de **reversión** de la reclasificación (día siguiente).

Posteo: inmediato si `date <= hoy`; el resto vía `auto_post='at_date'` + cron core `account.ir_cron_auto_post_draft_entry`.

### `account.asset` / `account.asset.group` — vínculo activos

`models/account_asset.py`, `models/account_asset_group.py`. El vínculo préstamo↔activo es **indirecto a través de `account.asset.group`** (no hay M2M directo):

- `account.loan.asset_group_id` (M2o → account.asset.group) — `account_loans/models/account_loan.py:73`, `class AccountLoan`.
- `account.asset.group.linked_loan_ids` (O2m ← account.loan por `asset_group_id`) — `account_asset_group.py`.
- `account.asset.linked_loans_ids` (related `asset_group_id.linked_loan_ids`) — `account_asset.py`.
- `account.loan.linked_assets_ids` (computed = `asset_group_id.linked_asset_ids`) — `account_loans/models/account_loan.py:168-172`, `class AccountLoan`.

Smart buttons cruzados: `account.asset.action_open_linked_loans` y `account.loan.action_open_linked_assets`. Caso de uso ES: préstamo que financia la compra de un activo (vehículo, maquinaria) → mismo asset_group para reporting conjunto.

`account_loans` **depende de `account_asset`** (`account_loans/__manifest__.py:10`) y reutiliza su patrón de generación de moves en batch.

### `account.journal` — properties por diario

`models/account_journal.py`: `loan_properties_definition` (PropertiesDefinition). Define el esquema de las `loan_properties` del préstamo (`account_loans/models/account_loan.py:74`, `class AccountLoan`). El diario del préstamo debe ser `type='general'` (`account_loans/models/account_loan.py:71`, `class AccountLoan`).

### `account.report` / `account.return` — auditoría (sólo v19)

En v18 NO hay bridge con reportes. En **v19** se añade `models/account_return.py`: extiende `account.return._check_suite_annual_closing` con un check `check_loans` que avisa en el cierre anual si no hay préstamos registrados en el periodo (`odoo_enterprise/19/account_loans/models/account_return.py`). Además `data/account_return_check_template.xml` define 2 plantillas de check ("Loans recording", "Loans classification") asociadas al ciclo `treasury_financing`. Ver v19_changes.md.

---

## 2. Diagrama — cuadro de amortización → asientos

Ejemplo ES (cuentas 170 LP / 520 CP / 662 gasto financiero). Cuota mensual con principal P, interés I, cuota C = P + I.

```
account.loan
  ├─ amount_borrowed, interest, duration, date
  ├─ long_term_account_id (170) / short_term_account_id (520) / expense_account_id (662)
  └─ line_ids  ── CUADRO DE AMORTIZACIÓN ──────────────────────────────────
       line[0]  date  principal=P0  interest=I0  payment=C0
       line[1]  date  principal=P1  interest=I1  payment=C1
       ...
       line[n]  (última: sólo genera payment move)

action_confirm()  ──▶ por cada line ──▶ 3 account.move (auto_post='at_date'):

 (1) PAGO  is_loan_payment_move=True   date = fin de mes
     ┌─────────────────────────────────────────────────┐
     │ DEBE   170 (LP)     principal  P                 │
     │ HABER  520 (CP)     payment    C = P + I         │
     │ DEBE   662 (gasto)  interest   I                 │
     └─────────────────────────────────────────────────┘   → se concilia con cargo banco (572)

 (2) RECLASIF LT→ST   is_loan_payment_move=False   date = fin de mes
     próximas 12 cuotas → corto plazo
     ┌─────────────────────────────────────────────────┐
     │ DEBE   170 (LP)   sum(next_12.principal)         │
     │ HABER  520 (CP)   sum(next_12.principal)         │
     └─────────────────────────────────────────────────┘

 (3) REVERSIÓN de (2)   date = fin de mes + 1 día (primer día mes siguiente)
     ┌─────────────────────────────────────────────────┐
     │ HABER  170 (LP)   sum(next_12.principal)         │
     │ DEBE   520 (CP)   sum(next_12.principal)         │
     └─────────────────────────────────────────────────┘   reversed_entry_id → (2)

cron account.ir_cron_auto_post_draft_entry  ──▶ postea cuotas futuras al vencer
último pago posteado  ──▶ account.move._post  ──▶ loan.state = 'closed'
```

La reclasificación (2)+(3) sólo "existe" el último día del mes, justo para que el balance refleje correctamente la deuda a corto plazo en el corte mensual.

---

## 3. Modelos implicados

| Modelo | Rol | Fichero |
|---|---|---|
| `account.loan` | Préstamo (cabecera + cuadro) | `models/account_loan.py` |
| `account.loan.line` | Línea del cuadro de amortización | `models/account_loan_line.py` |
| `account.move` (ext) | Asientos generados (`generating_loan_line_id`, `is_loan_payment_move`, `loan_id`) | `models/account_move.py` |
| `account.asset` (ext) | Activos vinculados vía grupo | `models/account_asset.py` |
| `account.asset.group` (ext) | Puente loan↔asset | `models/account_asset_group.py` |
| `account.journal` (ext) | `loan_properties_definition` | `models/account_journal.py` |
| `account.return` (ext, v19) | Checks de cierre anual | `19/models/account_return.py` |
| `account.loan.compute.wizard` | Genera cuadro (lib pyloan) | `wizard/account_loan_compute_wizard.py` |
| `account.loan.close.wizard` | Cierre con fecha | `wizard/account_loan_close_wizard.py` |
| `base_import.import` | Importa cuadro CSV/XLSX | dep `base_import` |

---

## 4. Config (checklist de puesta en marcha)

1. **Diario** `type='general'` para los asientos del préstamo (`account_loans/models/account_loan.py:71`, `class AccountLoan`). Opcional: definir `loan_properties_definition` en él.
2. **3 cuentas** (ES típico):
   - Largo plazo `long_term_account_id` → **170** (Deudas a largo plazo con entidades de crédito).
   - Corto plazo `short_term_account_id` → **520** (Deudas a corto plazo con entidades de crédito).
   - Gasto `expense_account_id` → **662** (Intereses de deudas), `account_type` expense (`account_loans/models/account_loan.py:66`, `class AccountLoan`).
3. **Cuadro de amortización**: vía wizard `account.loan.compute.wizard` (rellena amount/rate/term/start/método) o importando CSV/XLSX del banco (`action_upload_amortization_schedule`).
4. **`date`** (Loan Date) <= fecha de la primera cuota (`account_loans/models/account_loan.py:127-130`, `class AccountLoan`).
5. **Cuadre**: `sum(principal)==amount_borrowed`, `sum(interest)==interest`, `len(lines)==duration` al céntimo (`account_loans/models/account_loan.py:182-192`, `AccountLoan.action_confirm`).
6. **Cron del core activo**: `account.ir_cron_auto_post_draft_entry` debe estar habilitado para postear cuotas futuras.
7. (Opcional) `asset_group_id` para vincular el préstamo a los activos que financia.
8. (Opcional) `skip_until_date` si las primeras cuotas ya se contabilizaron a mano.

---

## 5. Gotchas cross-module (ver gotchas.md, prefijo LOANG)

- **LOANG1**: sin cron propio; depende de `account.ir_cron_auto_post_draft_entry` del core.
- **LOANG3**: el asiento de pago NO es contra banco; DEBE 170 / HABER 520 / DEBE 662. La conciliación con la 572 es posterior y manual.
- **LOANG4 / LOANG11**: corto plazo = 12 cuotas por POSICIÓN (hardcoded), no por fecha → mal con cadencia no mensual.
- **LOANG5 / LOANG14**: reversión y cancelación no usan `_reverse_moves` estándar; con audit trail se generan moves extra.
- **LOANG18**: vínculo con activos es indirecto vía `asset_group_id` (no M2M directo).
- **LOANG17 / LOANG19**: domains de `expense_account_id` (expense) y `journal_id` (general) restringen la config.

---

## 6. v19 (resumen — detalle en v19_changes.md)

- **Nuevo bridge `account.return`**: check `check_loans` en cierre anual + `account.return.check.template` (`treasury_financing`).
- **`expense_account_id`** acepta también `expense_other` en el domain.
- **`display_name`** deja de ser computed (pasa a Char stored simple), aunque conserva `_compute_display_name`.
- `default_get(fields)` renombra el parámetro (`fields_list` → `fields`).
- Sin cambios en la lógica de generación de asientos ni en `account.loan.line`.
