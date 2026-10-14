<!-- verified: v18.0 @ 2026-06 (source: odoo_enterprise/18/account_saft + l10n_*_saft + account_saft_import) -->
# account_saft — Integrations

Cómo `account_saft` se conecta con el resto de Odoo. Paths relativos a `odoo_enterprise/18/`. El módulo base es deliberadamente "delgado": casi toda la integración real ocurre en las localizaciones `l10n_*_saft`.

---

## 1. Bridges

### 1.1 ↔ `account_reports` (el motor real)

`account_saft` **depende exclusivamente de `account_reports`** (`account_saft/__manifest__.py:13-14`). No reinventa el report: hace `_inherit = 'account.general.ledger.report.handler'` (AbstractModel, `account_saft/models/account_general_ledger.py:11`, `class GeneralLedgerCustomHandler`) y se cuelga del report **General Ledger**.

Puntos de enganche que usa del engine:
- `report._get_report_query(options, 'strict_range')` → devuelve `from_clause` + `where_clause` que se inyectan en las queries SQL crudas (`account_saft/models/account_general_ledger.py:102`, `GeneralLedgerCustomHandler._saft_fill_report_general_ledger_entries` y `account_saft/models/account_general_ledger.py:167`, `GeneralLedgerCustomHandler._saft_fill_report_general_ledger_entries`).
- `report._get_report_query(options, 'from_beginning', domain=domain)` → para saldos de apertura de partners (`:307`).
- `report._init_currency_table(options)` → tabla de conversión de divisa (`:415`).
- `report._generate_file_data_with_error_check(options, render, {...}, errors)` → genera el fichero y aplica el check de errores bloqueantes (lo llama la localización, ej. `l10n_no_saft/models/account_general_ledger.py:76`, `GeneralLedgerCustomHandler.l10n_no_export_saft_to_xml`).
- `account.move.line._get_query_tax_details(from, where)` → base+cuota por impuesto (`:232`).
- Helpers `_query_values` / `_get_initial_balance_values` (v18) para saldos de cuentas (en v19 se sustituyen por `_get_report_values`; ver `v19_changes.md`).

El botón SAF-T se inyecta en `options['buttons']` del General Ledger — no hay menú propio.

### 1.2 ↔ `account.move` / `account.move.line` (origen de los asientos)

`GeneralLedgerEntries` se construye con un SELECT directo sobre `account_move_line` JOIN `account_move` (`account_saft/models/account_general_ledger.py:108-168`, `GeneralLedgerCustomHandler._saft_fill_report_general_ledger_entries`). Campos leídos: `display_type`, `date`, `name`, `account_id`, `partner_id`, `currency_id`, `amount_currency`, `debit`, `credit`, `balance`, `tax_line_id`, `quantity`, `price_unit`, `product_id`, `product_uom_id`, `tax_base_amount`; del move: `move_type`, `create_date`, `invoice_date`, `invoice_origin`, `statement_line_id`.

- `sign` por move: `-1 if move_type in get_inbound_types(include_receipts=True) else 1` (`:173,195`).
- Si la línea no tiene `name`, se sintetiza desde el producto: `[default_code] product_name` (`:208-209`).
- Agrupación jerárquica: `journal_vals_map` → `move_vals_map` → `line_vals_list`.

### 1.3 ↔ `res.partner` (master data: Customers / Suppliers / Owners)

`_saft_fill_report_partner_ledger_values` (`account_saft/models/account_general_ledger.py:276-401`, `GeneralLedgerCustomHandler._get_all_partners`):
- Clasifica partners en `customer_vals_list` / `supplier_vals_list` según saldos receivable/payable calculados con SQL `FILTER` por `account_type` (`asset_receivable`, `liability_payable`, y `asset_fixed`/`asset_non_current` para depreciaciones, `:295`).
- `partner_detail_map[id]` = `{type, types, addresses, contacts, partner}`.
- Direcciones: dedup por `(zip, city)`; solo si zip+city presentes (salvo `saft_allow_empty_address`).
- Contactos: individual = sí mismo; empresa = hijo `type='contact'` de menor ID; company = todos los hijos contacto.
- La company (`company.partner_id`) entra como `Owner` en MasterFiles vía `company_header` (`account_saft/data/saft_report.xml:210-215`, `saft_template`).

Campos de partner usados en plantilla (`account_saft/data/saft_report.xml:21-37`, `addresses_contacts`): `name`, `street`, `street2`, `city`, `zip`, `country_id.code`, `title.name`, `phone`/`mobile`, `email`, `website`, `vat`.

### 1.4 ↔ `account.tax` (TaxTable + TaxInformation)

`_saft_fill_report_tax_details_values` (`:227-274`) agrupa impuestos en `tax_vals_map` (id, name, description, amount, amount_type, type, create_date) y adjunta `tax_detail_vals_list` por línea base. La plantilla emite `TaxTable/TaxTableEntry/TaxCodeDetails` con `TaxPercentage` (si `amount_type=='percent'`) o `FlatTaxRate` (`account_saft/data/saft_report.xml:193-208`, `saft_template`). `TaxCode` = `tax.id` (ID interno).

### 1.5 ↔ extensiones de país `l10n_*_saft` (el patrón clave)

Cada país es un **bridge sobre `account_saft`** que:
1. Depende de `l10n_<país>` + `account_saft` (a veces `auto_install`).
2. Hereda `account.general.ledger.report.handler`.
3. Añade botón en `_custom_options_initializer` con `action='export_file'`, `action_param='<l10n>_export_saft_to_xml'`, solo si `account_fiscal_country_id.code == '<PAÍS>'`.
4. Define `_l10n_<país>_prepare_saft_report_values` → `super()._saft_prepare_report_values()` + override de `xmlns`/`file_version`/`accounting_basis` + datos país.
5. Hereda la plantilla con `<template id="saft_template_inherit_<país>" inherit_id="account_saft.saft_template" primary="True">` y hace `xpath` para reemplazar/añadir nodos.
6. Opcionalmente sobreescribe `_saft_get_account_type` para emitir `<AccountType>`.

| Módulo | depends | auto_install | Modelos/campos propios |
|--------|---------|--------------|------------------------|
| `l10n_no_saft` | `l10n_no`, `account_saft` | `True` | solo plantilla + override; parsea standard tax code del name |
| `l10n_lt_saft` | `l10n_lt`, `account_saft` | `True` | plantilla + owner_accounts + partner accounts |
| `l10n_ro_saft` | `l10n_ro`, `account_saft`, `account_asset` | `['l10n_ro','account_saft']` | modelo `l10n_ro_saft.tax.type`; campos en `account.tax`, `account.move`, `account.asset`, `res.company`; `post_init_hook` |
| `l10n_at_saft` | `l10n_at`, `account_saft` | `['l10n_at','account_saft']` | CSV `l10n_at_saft.account.csv`; res_config_settings |
| `l10n_pl_reports_account_saft` | `l10n_pl_reports`, `account_saft` | `True` | export **JPK_KR_PD** (income tax book, `jpk_export_templates.xml`) |
| `account_saft_import` | `account_saft`, `account_base_import` | — | wizard `account.saft.import.wizard`; parsing de SAF-T externos (namespace OCDE genérico) |
| `l10n_dk_saft_import`, `l10n_lt_saft_import`, `l10n_ro_saft_import` | `account_saft_import`, `l10n_<cc>` | `True` (DK) | bridge import: solo override de `_get_account_types()` |
| `l10n_ro_saft_stock` (solo v19) | `l10n_ro_saft`, `stock_account`, `account_edi_ubl_cii` | — | D.406 de stocks/inventario |

---

## 2. Diagrama

```
                       account_reports
                  (account.report "General Ledger"
                   + account.general.ledger.report.handler)
                              ▲  _inherit
                              │
                       ┌──────┴───────┐
                       │ account_saft │   (BASE — sin botón, sin modelo propio)
                       │  models/     │
                       │  account_general_ledger.py
                       │  data/saft_report.xml (QWeb templates)
                       └──────┬───────┘
            _inherit handler  │  inherit_id plantilla (primary=True)
        ┌─────────┬───────────┼───────────┬──────────┬─────────────┐
        ▼         ▼           ▼           ▼          ▼             ▼
   l10n_no_saft l10n_lt_saft l10n_ro_saft l10n_at_saft l10n_pl_reports_account_saft  ...
   (NO 1.30)    (LT 2.01)    (RO D.406)   (AT)         (PL JPK)
        │         │           │
        │         │           ├─ l10n_ro_saft.tax.type (modelo)
        │         │           ├─ account.tax.l10n_ro_saft_tax_type_id / _tax_code
        │         │           ├─ account.move.l10n_ro_is_self_invoice
        │         │           └─ account.asset categories
        ▼         ▼           ▼
   SQL crudo sobre: account_move_line · account_move · account_journal ·
   account_account · account_tax · res_partner · res_currency ·
   product_product · product_template · uom_uom
        │
        ▼
   ir.qweb._render(saft_template_inherit_<país>) ──► XML SAF-T (file_data, descarga)

   ESPAÑA: NO existe l10n_es_saft → ninguna rama → usar SII / VeriFactu (otro stack)
```

---

## 3. Modelos

`account_saft` (base): **0 modelos `_name` nuevos**. Solo `_inherit` de `account.general.ledger.report.handler` (AbstractModel).

Métodos clave del handler base (`account_general_ledger.py`):

| Método | Línea | Propósito |
|--------|-------|-----------|
| `_customize_warnings` | 13 | warning datos company faltantes |
| `action_fill_company_details` | 33 | abre form res.company |
| `_saft_get_account_type` | 42 | hook AccountType (override país) |
| `_saft_fill_report_general_ledger_accounts` | 83 | `account_vals_list` |
| `_saft_fill_report_general_ledger_entries` | 93 | `journal_vals_list` + `move_vals_list` |
| `_saft_fill_report_tax_details_values` | 227 | `tax_vals_list` |
| `_saft_fill_report_partner_ledger_values` | 276 | customers/suppliers/`partner_detail_map` |
| `_saft_prepare_report_values` | 404 | orquestador (lo llama la localización) |

Modelos añadidos por localizaciones (ejemplo RO, `l10n_ro_saft/models/`):
- `l10n_ro_saft.tax.type` (`_name`): code + description (`l10n_ro_saft/models/l10n_ro_saft_tax_type.py:5-13`, `class L10nRoSaftTaxType`).
- `account.tax` (+`l10n_ro_saft_tax_type_id`, `l10n_ro_saft_tax_code`).
- `account.move` (+`l10n_ro_is_self_invoice`).
- `account.asset` / `l10n_ro_saft.account.asset.category`.
- `res.company` / `res.config.settings` (+`l10n_ro_saft_tax_accounting_basis`).
- `account.chart.template` (extensión).

---

## 4. Config

`account_saft` **no tiene configuración propia** (sin settings, sin menú). La config relevante vive en datos maestros que el export valida:

- **Company** (`res.company`): `company_registry` (→ `RegistrationNumber`), `phone`/`mobile`, `zip`/`city`, `currency_id` (→ `DefaultCurrencyCode`), `account_fiscal_country_id` (→ `AuditFileCountry` y gating del botón), `bank_ids` (IBAN/cuenta), al menos un contacto hijo. `_customize_warnings` avisa de los que falten (`account_saft/models/account_general_ledger.py:13-27`, `GeneralLedgerCustomHandler._customize_warnings`).
- **Partners**: VAT, dirección completa (zip+city), contacto.
- **Impuestos**: en RO, `l10n_ro_saft_tax_type_id` + `l10n_ro_saft_tax_code` obligatorios o el export bloquea (`l10n_ro_saft/models/account_general_ledger.py:341-354`, `GeneralLedgerCustomHandler._l10n_ro_saft_fill_partner_values`). En NO, el código estándar debe ir embebido en la descripción del impuesto.
- **País**: la company debe tener instalada la localización `l10n_<país>` correspondiente Y su `l10n_<país>_saft`. Selección del periodo + un único column group en el report (ver SAFTG4).
- **AT**: plan de cuentas oficial cargado vía `l10n_at_saft.account.csv` (id, code, name, account_type, account_class).

Acceso: Contabilidad → Informes → **Libro Mayor** → botón **SAF-T** (solo si la company es de un país soportado).

---

## 5. Gotchas SAFTG (resumen — detalle en `gotchas.md`)

| ID | Resumen |
|----|---------|
| SAFTG1 | España NO usa SAF-T (no hay `l10n_es_saft`) → usar SII/VeriFactu |
| SAFTG2 | El base no genera nada sin un `l10n_*_saft` |
| SAFTG3 | SAF-T reutiliza el report General Ledger; no tiene report propio |
| SAFTG4 | `UserError` si >1 column group |
| SAFTG5 | Queries SQL crudo; record rules ORM no aplican igual |
| SAFTG6 | `xmlns`/`file_version`/`accounting_basis` los pone la localización |
| SAFTG7 | `accounting_basis` (devengo K/A vs caja P) varía por país |
| SAFTG8 | NO: código fiscal estándar dentro del nombre del impuesto |
| SAFTG9 | NO: RedirectWarning si falta `l10n_no_saft.balance_account` |
| SAFTG10 | Partners sin Zip/City bloquean (salvo `saft_allow_empty_address`) |
| SAFTG11 | La company debe tener ≥1 contacto hijo |
| SAFTG12 | `CustomerID`/`SupplierID` = ID interno Odoo, no código fiscal |
| SAFTG13 | `partner_detail_map['type']` legacy; usar `types` (lista) |
| SAFTG14 | Líneas a cero igualmente emiten Debit/Credit según move_type |
| SAFTG15 | `rate` = abs(amount_currency)/abs(balance), 1.0 si balance=0 |
| SAFTG16 | RO/AT/PL añaden modelos y campos propios, no es "solo XML" |
| SAFTG17 | El detalle de impuestos descarta líneas donde signo(cuota) ≠ signo(base) |
| SAFTG18 | base+cuota salen de `_get_query_tax_details`, no del campo `tax_base_amount` |
| SAFTG19 | nombre de línea sintetizado desde producto; `sign` por move invierte en refunds |
| SAFTG20 | `sign` de TaxInformation = -1 si la línea tiene credit (por línea, no por move) |
| SAFTG21 | `FirstName` siempre literal `"NotUsed"`; nombre completo va en LastName |
| SAFTG22 | CustomerID/SupplierID: transacción usa `types` (ambos), línea usa `type` singular |
| SAFTG23 | RO: dos botones (D.406 mensual vs activos) con secciones distintas; v19 +stock |
| SAFTG24 | RO valida company/partners/taxes/products/assets y bloquea por muchos motivos |
| SAFTG25 | RO elimina líneas con balance==0 (contradice SAFTG14) |
| SAFTG26 | AT mapea cada cuenta a un `l10n_at_saft.account` vía un único tag numérico |
| SAFTG27 | LT `_saft_get_account_type` lanza KeyError si el account_type no está en el dict |
| SAFTG28 | Import solo entiende namespace genérico OCDE, no los nacionales |
| SAFTG29 | Import deduplica por claves naturales (code/name+vat/desc+amount/journal+name) |
| SAFTG30 | Import fija `sequence.mixin.constraint_start_date` global (no se restaura) |
| SAFTG31 | Import de opening balances crea asiento de AJUSTE (diferencia), no réplica |

---

## 6. v19 (resumen — detalle en `v19_changes.md`)

- Clase renombrada `GeneralLedgerCustomHandler` → `AccountGeneralLedgerReportHandler` (`account_saft/models/account_general_ledger.py:10`, `class GeneralLedgerCustomHandler`).
- Cálculo de saldos de cuentas reescrito: `_get_accounts_results`/`_get_accounts_initial_balances_map` (basados en `_query_values`/`_get_initial_balance_values`) → nuevo `_get_report_values` que parsea las **líneas del report** (`report._get_lines`).
- `_saft_prepare_report_values` se parte en `_saft_prepare_report_initial_values` + `_saft_prepare_report_values`. Nuevo hook `_get_all_partners`.
- Detección de undistributed earnings → warning `level='info'` (riesgo de rechazo).
- `_customize_warnings`: ya solo exige `phone` (no `mobile`); plantilla quita `Title` y deja `Telephone` solo con `phone`.
- Domain de partners añade `asset_current`.
- Nuevo módulo `l10n_ro_saft_stock` (declaración D.406 de stocks).
- España sigue SIN SAF-T en v19.
