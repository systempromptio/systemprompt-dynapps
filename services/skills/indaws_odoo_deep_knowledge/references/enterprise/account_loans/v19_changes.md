# account_loans — v19 vs v18 (diffs) — EE

Comparativa `odoo_enterprise/18/account_loans` vs `odoo_enterprise/19/account_loans`. Verificado con `diff` de los modelos y manifests.

La lógica central (generación de asientos en `action_confirm`, cuadro de amortización, estados, wizards) es **idéntica**. Los cambios son: nuevo bridge con `account.return`, ampliación de un domain, simplificación de `display_name` y rename de parámetro.

---

## 1. Nuevo bridge `account.return` — checks de cierre anual

**Nuevo fichero**: `19/account_loans/models/account_return.py` (no existe en v18).

```python
class AccountReturn(models.Model):
    _inherit = 'account.return'

    def _check_suite_annual_closing(self, check_codes_to_ignore):
        checks = super()._check_suite_annual_closing(check_codes_to_ignore)
        if 'check_loans' not in check_codes_to_ignore:
            domain = [('company_id', 'in', self.company_ids.ids),
                      ('date', '<=', ...date_to), ('date', '>=', ...date_from)]
            loans_exist = self.env['account.loan'].sudo().search_count(domain)
            if not loans_exist:
                checks.append({'name': _("Loans"), 'code': 'check_loans',
                               'result': 'todo', 'message': _("...No loans were found...")})
        return checks
```

Añade un check al cierre anual (`account.return`) que avisa si no hay préstamos registrados en el periodo. `result='todo'` (recordatorio, no bloqueante).

**Nuevo data**: `19/account_loans/data/account_return_check_template.xml` con 2 `account.return.check.template`:
- `account_return_check_template_loans_recording` ("Loans recording") — verificar que los préstamos contabilizados existen y casan con los contratos.
- `account_return_check_template_loans_classification` ("Loans classification") — clasificación corriente/no corriente. La descripción cita GAAP belga (cuenta 42 a <1 año, 17 a >1 año) — texto orientado a BE, conceptualmente equivale al 520/170 español.

Ambas: `cycle='treasury_financing'`, `return_type` ref `account_reports.default_audit_return_type`, `action_id` ref `account_loans.action_view_account_loans`.

**Implicación**: en v19 `account_loans` se integra con el motor de auditoría de `account_reports` (sin dependencia añadida en el manifest; el inherit es soft via `_inherit` y los data sólo cargan si los modelos existen). Estos checks NO existen en v18.

---

## 2. `expense_account_id` — domain ampliado

`account_loans/models/account_loan.py:66`, `AccountLoan.expense_account_id`:

- **v18**: `domain="[('account_type', 'in', ('expense', 'expense_depreciation'))]"`
- **v19**: `domain="[('account_type', 'in', ('expense', 'expense_depreciation', 'expense_other'))]"`

Se añade `expense_other`. Ahora pueden seleccionarse cuentas de gasto "otros" para los intereses. Relevante si tu plan mapea intereses financieros a una cuenta de tipo `expense_other`. (Ver LOANG17.)

---

## 3. `display_name` — deja de ser computed declarado

`account_loans/models/account_loan.py:79`, `AccountLoan.display_name`:

- **v18**: `display_name = fields.Char("Loan name", compute='_compute_display_name', store=True)`
- **v19**: `display_name = fields.Char(string="Loan name", store=True)`

En v19 ya no se declara `compute=` en el campo (el método `_compute_display_name` sigue existiendo, `account_loans/models/account_loan.py:108-115`, `AccountLoan._compute_display_name`). Alinea con el manejo estándar de `display_name` del ORM en v19. El resultado funcional (`"name: MM y - MM y"`) es el mismo.

---

## 4. `default_get` — rename de parámetro

`account_loans/models/account_loan.py:16`, `AccountLoan.default_get`:

- **v18**: `def default_get(self, fields_list):` … usa `fields_list`.
- **v19**: `def default_get(self, fields):` … usa `fields`.

Cambio cosmético (nombre del argumento). Misma lógica de precarga de cuentas/diario desde el último préstamo.

---

## 5. Manifest

`__manifest__.py`:
- v19 añade `'author': 'Odoo S.A.'`.
- v19 añade al `data`: `'data/account_return_check_template.xml'`.
- v19 añade en `assets`: `'web.assets_unit_tests': ['account_loans/static/tests/*']` (tests JS).
- `depends` sin cambios: `['account_asset', 'base_import']`.

---

## 6. Sin cambios (verificado por `diff` vacío)

- `models/account_loan_line.py` — **idéntico** (diff vacío). Cuadro de amortización, corto/largo plazo por posición 12, computes, todo igual.
- `models/account_move.py` — **idéntico** (diff vacío). `generating_loan_line_id`, `is_loan_payment_move`, override `_post` con cierre automático, `open_loan` — sin cambios.
- `action_confirm` y la generación de los 3 tipos de move (pago / reclasif LT→ST / reversión) — sin cambios.
- Wizards `account.loan.compute.wizard` (lib `pyloan`) y `account.loan.close.wizard` — sin cambios estructurales.
- Bridge `account.asset` / `account.asset.group` — sin cambios.
- Sigue SIN cron propio: depende de `account.ir_cron_auto_post_draft_entry` del core (LOANG1 aplica en v18 y v19).

---

## Resumen para migración v18 → v19

| Cambio | Impacto |
|---|---|
| Nuevo `account.return` + check templates | Aparecen 2 checks de cierre anual si tienes `account_reports`. No bloqueantes. |
| `expense_account_id` domain +`expense_other` | Más cuentas elegibles para intereses. No rompe nada. |
| `display_name` sin `compute=` declarado | Transparente; mismo valor. |
| `default_get(fields)` rename | Sólo afecta si tenías override por `super(...).default_get(fields_list)` con kwarg nombrado. |
| Lógica de asientos / cuadro | **Sin cambios** — migración directa. |
