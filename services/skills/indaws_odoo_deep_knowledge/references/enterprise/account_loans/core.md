<!-- verified: v18.0 @ 2026-06 -->
# account_loans — Core (Odoo 18.0 Enterprise) — EE

Módulo `account_loans` (`Loans Management`). Gestiona préstamos/financiación con cuadro de amortización (líneas: principal + intereses + cuota) y genera automáticamente los asientos contables por cada línea: el pago (principal+interés) y la reclasificación largo plazo → corto plazo. `auto_install: True`, depende de `account_asset` + `base_import`, `license OEEL-1`.

Path raíz: `odoo_enterprise/18/account_loans`.

---

## Modelos

| Modelo | Fichero | Tipo |
|---|---|---|
| `account.loan` | `models/account_loan.py` | Model (`mail.thread`) |
| `account.loan.line` | `models/account_loan_line.py` | Model (línea del cuadro) |
| `account.move` (ext) | `models/account_move.py` | Inherit |
| `account.asset` / `account.asset.group` (ext) | `models/account_asset.py`, `account_asset_group.py` | Inherit (bridge assets) |
| `account.journal` (ext) | `models/account_journal.py` | Inherit (`loan_properties_definition`) |
| `account.loan.compute.wizard` | `wizard/account_loan_compute_wizard.py` | TransientModel (genera schedule) |
| `account.loan.close.wizard` | `wizard/account_loan_close_wizard.py` | TransientModel (cierre) |

No hay `ir.cron` propio del módulo. El posteo automático lo hace el cron core `account.ir_cron_auto_post_draft_entry` (ver sección Cron).

---

## `account.loan` — campos clave

`account_loans/models/account_loan.py:34-93`, `class AccountLoan`.

- `name` (Char, required, tracking) — `account_loans/models/account_loan.py:34`, `class AccountLoan`.
- `state` Selection `draft / running / closed / cancelled`, default `draft` — `account_loans/models/account_loan.py:38-49`, `class AccountLoan`.
- `date` ("Loan Date") — fecha del préstamo; debe ser anterior o igual a la fecha de la primera línea — `account_loans/models/account_loan.py:50`, `class AccountLoan`.
- `amount_borrowed` (Monetary, tracking) — capital prestado — `account_loans/models/account_loan.py:51`, `class AccountLoan`.
- `interest` (Monetary) — interés total — `account_loans/models/account_loan.py:52`, `class AccountLoan`.
- `duration` (Integer) — nº de cuotas esperado — `account_loans/models/account_loan.py:53`, `class AccountLoan`.
- `skip_until_date` (Date) — al confirmar, ignora las líneas con `date < skip_until_date` (comparación **estricta**, NO inclusiva, pese a que el `help` dice "up to this date (included)") y NO genera asientos para ellas (útil si ya hubo asientos manuales previos) — `account_loans/models/account_loan.py:54-58`, `AccountLoan.skip_until_date` y `account_loans/models/account_loan.py:202`, `AccountLoan.action_confirm`.
- `currency_id` (related `company_id.currency_id`) — `account_loans/models/account_loan.py:36`, `class AccountLoan`.

### Cuentas y diario (config obligatoria al confirmar)

- `long_term_account_id` (account.account) — cuenta deuda a largo plazo (ES: **170**) — `account_loans/models/account_loan.py:60`, `class AccountLoan`.
- `short_term_account_id` (account.account) — cuenta deuda a corto plazo (ES: **520**) — `account_loans/models/account_loan.py:61`, `class AccountLoan`.
- `expense_account_id` (account.account, domain `account_type in (expense, expense_depreciation)`) — gasto financiero / intereses (ES: **662**) — `account_loans/models/account_loan.py:62-67`, `class AccountLoan`.
- `journal_id` (account.journal, domain `type='general'`) — diario donde se postean los asientos — `account_loans/models/account_loan.py:68-72`, `class AccountLoan`.

`default_get` (`account_loans/models/account_loan.py:15-32`, `class AccountLoan`) precarga estas 4 cuentas/diario copiándolas del **primer** loan (`search(..., limit=1)` con `_order='date'` → el de fecha más antigua, no el más reciente) de la company que tenga las 4 informadas (evita reconfigurar en cada préstamo). Guard (`:18`): si NINGUNO de los 4 campos está en `fields_list` hace `return` temprano sin precargar; basta que UNO esté solicitado para disparar la búsqueda y setear los 4.

### Otros campos

- `asset_group_id` (account.asset.group) — vincula el préstamo a un grupo de activos (bridge assets) — `account_loans/models/account_loan.py:73`, `class AccountLoan`.
- `loan_properties` (Properties, definición en `journal_id.loan_properties_definition`) — `account_loans/models/account_loan.py:74`, `class AccountLoan`.
- `line_ids` (One2many → account.loan.line) — **el cuadro de amortización** — `account_loans/models/account_loan.py:76`, `class AccountLoan`.

### Campos computados

- `start_date` / `end_date` = `line_ids[0].date` / `line_ids[-1].date` — `account_loans/models/account_loan.py:117-125`, `class AccountLoan`.
- `is_wrong_date` = no hay `date` o alguna línea tiene `date < loan.date` — `account_loans/models/account_loan.py:127-130`, `class AccountLoan`.
- `amount_borrowed_difference` = `abs(amount_borrowed - round(sum(line.principal)))` — `account_loans/models/account_loan.py:132-138`, `class AccountLoan`.
- `interest_difference` = `interest - round(sum(line.interest))` — `account_loans/models/account_loan.py:140-146`, `class AccountLoan`.
- `duration_difference` = `duration - len(line_ids)` — `account_loans/models/account_loan.py:148-151`, `class AccountLoan`.
- `outstanding_balance` (Monetary) — capital pendiente **basado en asientos posteados**: parte de `amount_borrowed` y resta el `principal` de cada línea con `is_payment_move_posted` o anterior a `skip_until_date`, sólo si `state='running'` — `account_loans/models/account_loan.py:158-166`, `class AccountLoan`.
- `nb_posted_entries` — nº de asientos generados posteados — `account_loans/models/account_loan.py:153-156`, `class AccountLoan`.
- `linked_assets_ids` / `count_linked_assets` — activos del `asset_group_id` — `account_loans/models/account_loan.py:168-172`, `class AccountLoan`.

### Constraint

`_require_positive_values` (`account_loans/models/account_loan.py:96-104`, `class AccountLoan`): `amount_borrowed >= 0`, `interest >= 0`, `duration >= 0`.

---

## `account.loan.line` — la línea del cuadro de amortización

`account_loan_line.py`. `_order = 'date, id'`.

- `loan_id` (M2o required, `ondelete=cascade`) — `account_loans/models/account_loan_line.py:10`, `class AccountLoanLine`.
- `date` (Date, required) — fecha de vencimiento de la cuota — `account_loans/models/account_loan_line.py:18`, `class AccountLoanLine`.
- `principal` (Monetary) — amortización de capital de la cuota — `account_loans/models/account_loan_line.py:19`, `class AccountLoanLine`.
- `interest` (Monetary) — intereses de la cuota — `account_loans/models/account_loan_line.py:20`, `class AccountLoanLine`.
- `payment` (Monetary, computed **store**) = `principal + interest` — cuota total — `account_loans/models/account_loan_line.py:21-25`, `AccountLoanLine.payment` y `account_loans/models/account_loan_line.py:50-53`, `AccountLoanLine._compute_payment`.
- `outstanding_balance` (computed, NO store) = `loan.amount_borrowed - sum(principal de líneas con date <= esta)` — saldo teórico a esa fecha — `account_loans/models/account_loan_line.py:26-29`, `AccountLoanLine.outstanding_balance` y `account_loans/models/account_loan_line.py:55-58`, `AccountLoanLine._compute_outstanding_balance`.
- `long_term_theoretical_balance` (computed **store**) = `sum(principal)` de las líneas con `date > esta` **a partir de la 13ª** (`filtered[12:]`) — `account_loans/models/account_loan_line.py:30-34`, `AccountLoanLine.long_term_theoretical_balance` y `account_loans/models/account_loan_line.py:60-65`, `AccountLoanLine._compute_theoretical_balances`.
- `short_term_theoretical_balance` (computed **store**) = `sum(principal)` de las **12 primeras** líneas con `date > esta` (`filtered[:12]`) — `account_loans/models/account_loan_line.py:35-39`, `AccountLoanLine.short_term_theoretical_balance` y `account_loans/models/account_loan_line.py:60-65`, `AccountLoanLine._compute_theoretical_balances`.
- `sequence` (Integer, computed, "#") — posición ordinal por fecha — `account_loans/models/account_loan_line.py:9`, `AccountLoanLine.sequence` y `account_loans/models/account_loan_line.py:67-70`, `AccountLoanLine._compute_sequence`.
- `generated_move_ids` (One2many ← account.move por `generating_loan_line_id`) — asientos generados por esta línea — `account_loans/models/account_loan_line.py:40-47`, `class AccountLoanLine`.
- `is_payment_move_posted` (Boolean computed) — True si el move de pago (`is_loan_payment_move=True`) está posteado. Maneja el caso de audit trail: si hay >1 move generado tras cerrar/cancelar y reconfirmar, toma el que no tiene reversal — `account_loans/models/account_loan_line.py:48`, `AccountLoanLine.is_payment_move_posted` y `account_loans/models/account_loan_line.py:72-80`, `AccountLoanLine._compute_is_payment_move_posted`.

Regla de **corto vs largo plazo**: las próximas **12** cuotas son corto plazo; el resto, largo plazo. Es la base de la reclasificación mensual.

---

## Generación de asientos — `action_confirm()`

`account_loans/models/account_loan.py:175-301`, `AccountLoan.action_confirm`. Patrón análogo al de `account_asset` (crear todos los moves en batch con `auto_post='at_date'` y postear los de fecha <= hoy).

### Verificaciones previas (UserError si fallan)

`account_loans/models/account_loan.py:178-196`, `AccountLoan.action_confirm`: `name` informado, `is_wrong_date=False`, `amount_borrowed_difference==0`, `interest_difference==0`, `duration_difference==0`, las 3 cuentas informadas, `journal_id` informado.

### Por cada línea (saltando las anteriores a `skip_until_date`)

Se construyen 3 tipos de asiento (`account.move`):

**1. Asiento de pago (Principal & Interest)** — `account_loans/models/account_loan.py:206-232`, `AccountLoan.action_confirm`. Uno por línea. `is_loan_payment_move=True`, `date = line.date` ajustada a fin de mes (`relativedelta(day=31)`), `generating_loan_line_id=line.id`. 3 apuntes:

```
DEBE   long_term_account_id   = principal      (170)
HABER  short_term_account_id  = payment        (520)  ← principal + interés (la cuota a pagar)
DEBE   expense_account_id     = interest       (662)
```

Es el asiento que se concilia luego contra el extracto bancario (el cargo del banco).

**2. Reclasificación LT → ST** — `account_loans/models/account_loan.py:247-263`, `AccountLoan.action_confirm`. Uno por línea (salvo la última, `account_loans/models/account_loan.py:235-236`, `AccountLoan.action_confirm`). `is_loan_payment_move=False`, `date` a fin de mes. Reclasifica de largo a corto plazo la suma de principal de las **próximas 12 cuotas** (`next_lines = line_ids[i+1:i+13]`, `account_loans/models/account_loan.py:237`, `AccountLoan.action_confirm`):

```
DEBE   long_term_account_id   = sum(next_12.principal)   (170)
HABER  short_term_account_id  = sum(next_12.principal)   (520)
```

**3. Reversión de la reclasificación** — `account_loans/models/account_loan.py:265-281`, `AccountLoan.action_confirm`. Asiento inverso del anterior, con `date = fin de mes + 1 día` (primer día del mes siguiente). Se crea manualmente (NO con `_reverse_moves()`) por rendimiento, y luego se enlaza con `reversed_entry_id` (`account_loans/models/account_loan.py:291-293`, `AccountLoan.action_confirm`). Efecto neto: la reclasificación sólo vive durante 1 día, sirve para reflejar el saldo a corto plazo en el corte mensual del reporte.

### Posteo y estado

`account_loans/models/account_loan.py:283-301`, `AccountLoan.post_moves`: se crean los 3 batches de moves; `post_moves()` postea inmediatamente los de `date <= context_today`. Los futuros quedan en `draft` con `auto_post='at_date'`. Si queda algún move sin postear, el loan pasa a `state='running'` (`account_loans/models/account_loan.py:300-301`, `AccountLoan.action_confirm`).

El paso a `state='closed'` ocurre en `account.move._post` (`account_loans/models/account_move.py:19-25`, `AccountMove._post`): cuando TODAS las líneas del loan tienen su payment move posteado (o son anteriores a `skip_until_date`), el loan se cierra.

---

## Cron de posteo

**No existe cron en `account_loans`.** Los moves futuros se crean con `auto_post='at_date'`. El posteo lo ejecuta el cron del core `account`:

- `account.ir_cron_auto_post_draft_entry` (`odoo18/addons/account/data/service_cron.xml:3-9`, `ir_cron_auto_post_draft_entry`), código `model._autopost_draft_entries()`.

Cuando llega la fecha de cada cuota, el cron postea el payment move; al postearse el último, el `_post` cierra el loan.

---

## Estados (state machine)

```
draft ──action_confirm()──▶ running ──(cron postea último pago)──▶ closed
  ▲                            │
  │                            └─action_close() (wizard)──▶ closed
  │ action_set_to_draft()
  └──────────────── cancelled ◀── action_cancel()
```

- `action_confirm` (`account_loans/models/account_loan.py:175`, `AccountLoan.action_confirm`): genera asientos, → `running` (o queda en draft si todo se postea en el acto y se cierra).
- `action_cancel` (`account_loans/models/account_loan.py:392-394`, `AccountLoan.action_cancel`): `_unlink_or_reverse()` de todos los moves no cancelados, → `cancelled`.
- `action_set_to_draft` (`account_loans/models/account_loan.py:396-398`, `AccountLoan.action_set_to_draft`): igual, → `draft`.
- `action_close` (`account_loans/models/account_loan.py:378-390`, `AccountLoan.action_close`) → wizard `account.loan.close.wizard`: borra los moves draft posteriores a la fecha de cierre y pone `closed` (`account_loans/wizard/account_loan_close_wizard.py:20-24`, `AccountCloseWizard.action_save`).
- `action_reset` (`account_loans/models/account_loan.py:374-376`, `AccountLoan.action_reset`): borra `line_ids` (sólo cuadro, sin tocar moves).

---

## Construcción del cuadro de amortización

Dos vías:

1. **Wizard de cálculo** `account.loan.compute.wizard` (`wizard/account_loan_compute_wizard.py`). Usa la librería embebida `lib/pyloan` (`from ..lib import pyloan`, `account_loans/wizard/account_loan_compute_wizard.py:7`, `def pyloan`). Parámetros: `loan_amount`, `interest_rate` (digits (12,10)), `loan_term` (años), `start_date`, `first_payment_date`, `payment_end_of_month` (`end_of_month` / `at_anniversary`), `compounding_method` (8 convenciones: `30E/360` default, `A/365F`, `A/A ISDA`...). `loan_type = 'annuity' if interest_rate else 'linear'` (`account_loans/wizard/account_loan_compute_wizard.py:99`, `AccountLoanComputeWizard._get_loan_payment_schedule`). `action_save` (`:132-156`) crea las `account.loan.line` y escribe `date/amount_borrowed/interest/duration` en el loan.

2. **Importación de fichero** (CSV/XLSX). `action_upload_amortization_schedule(attachment_id)` (`account_loans/models/account_loan.py:303-330`, `AccountLoan.action_upload_amortization_schedule`) lanza el wizard de `base_import` con `res_model='account.loan.line'` y tag cliente `import_loan`. Tras importar, `action_file_uploaded` (`account_loans/models/account_loan.py:332-352`, `AccountLoan.action_file_uploaded`) recalcula `amount_borrowed/interest/duration/date` desde las líneas. El `post_init_hook` `_account_loans_post_init` (`__init__.py`) usa esta vía para cargar el demo.

---

## Acciones de navegación

- `action_open_loan_entries` (`account_loans/models/account_loan.py:400-409`, `AccountLoan.action_open_loan_entries`): abre los `account.move` generados (list view `account_loans.account_loan_view_account_move_list_view`).
- `action_open_linked_assets` (`account_loans/models/account_loan.py:411-419`, `AccountLoan.action_open_linked_assets`): activos vinculados vía `asset_group_id`.
- `account.move.open_loan` (`account_loans/models/account_move.py:27-36`, `AccountMove.open_loan`): desde el asiento, abre el loan origen.

---

## Seguridad

`security/ir.model.access.csv`: lectura para `account.group_account_readonly`, CRUD para `account.group_account_manager`. Reglas multi-company en `security/account_loans_security.xml` (`domain [('company_id', 'parent_of', company_ids)]`) sobre `account.loan` y `account.loan.line`.
