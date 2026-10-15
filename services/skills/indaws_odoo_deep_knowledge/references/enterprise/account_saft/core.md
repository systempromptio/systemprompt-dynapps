<!-- verified: v18.0 @ 2026-06 (source: odoo_enterprise/18/account_saft + l10n_*_saft; v19 spot-checked) -->
# account_saft — Standard Audit File for Tax (SAF-T)

Module path: `odoo_enterprise/18/account_saft/` (EE only).
Manifest: `name='Standard Audit File for Tax Base module'`, `version='1.0'`, `license='OEEL-1'`, `depends=['account_reports']` (`account_saft/__manifest__.py:5-24`).

## Qué es SAF-T

SAF-T (Standard Audit File for Tax) es un fichero XML estandarizado por la **OCDE** para que las administraciones tributarias auditen la contabilidad de una empresa de forma uniforme. Contiene, en un único XML, el plan de cuentas, maestro de clientes/proveedores, tabla de impuestos y el detalle de TODOS los asientos contables de un periodo. Lo exigen varios países europeos con su propio esquema XSD nacional (versión, namespace y campos extra varían por país).

**`account_saft` es un módulo BASE**: por sí solo NO genera ningún fichero ni añade ningún botón. Solo provee la maquinaria común (cálculo de valores + plantilla QWeb genérica). El botón "SAF-T" y el método de export los añade SIEMPRE un módulo de localización `l10n_<país>_saft` (ver `integrations.md`). El propio manifest lo dice: *"This is meant to be used with localization specific modules."* (`account_saft/__manifest__.py:11`).

## CONTEXTO ESPAÑA — NO APLICA

**España NO usa SAF-T.** No existe ni existió `l10n_es_saft` en el árbol EE (verificado: `ls odoo_enterprise/18/ | grep saft` no devuelve nada de ES). España cumple con la AEAT mediante:
- **SII** (Suministro Inmediato de Información) — envío de registros de facturación casi en tiempo real vía SOAP.
- **VeriFactu / TicketBAI** — sistemas de facturación verificable.

No propongas `account_saft` a un cliente español: no hay esquema XSD español ni módulo de localización que lo soporte. Es un módulo para PT/RO/LT/NO/AT/PL/DK etc.

## Países soportados (módulos `l10n_*_saft` en v18 EE)

| Módulo | País | xmlns / versión SAF-T |
|--------|------|-----------------------|
| `l10n_no_saft` | Noruega | `urn:StandardAuditFile-Taxation-Financial:NO`, v`1.30`, basis `A` (`l10n_no_saft/models/account_general_ledger.py:66-68`, `GeneralLedgerCustomHandler._l10n_no_prepare_saft_report_values`) |
| `l10n_lt_saft` | Lituania | `https://www.vmi.lt/cms/saf-t`, v`2.01`, basis `K`/`P` (`l10n_lt_saft/models/account_general_ledger.py:41-43`, `GeneralLedgerCustomHandler._l10n_lt_saft_prepare_report_values`) |
| `l10n_ro_saft` | Rumanía | declaración **D.406**, depende también de `account_asset` (`l10n_ro_saft/__manifest__.py`) |
| `l10n_at_saft` | Austria | `urn:OECD:StandardAuditFile-Taxation:AT_1.01`, v`1.01`; mapea cada cuenta a una "virtual account" del CoA oficial vía CSV (`l10n_at_saft.account.csv` → modelo `l10n_at_saft.account`) |
| `l10n_ro_saft` | Rumanía | `mfp:anaf:dgti:d406:declaratie:v1`, v`2.4.8` (D.406); `l10n_ro_saft/models/account_general_ledger.py:178-180`, `GeneralLedgerCustomHandler._l10n_ro_saft_fill_header_values` |
| `l10n_pl_reports_account_saft` | Polonia | export **JPK_KR_PD** (income tax book; templates `jpk_export_templates.xml`) |
| `l10n_dk_saft_import` | Dinamarca | solo import (bridge de `account_saft_import`) |
| `account_saft_import` | base import | parsing de ficheros SAF-T externos (wizard `account.saft.import.wizard`) |

v19 añade `l10n_ro_saft_stock` (declaración D.406 de stocks/inventario; `depends: account_edi_ubl_cii, l10n_ro_saft, stock_account`).

## Modelo base — NO hay modelo nuevo

`account_saft` **NO declara ningún modelo `_name` propio**. Todo cuelga del report engine de `account_reports`:

- Único fichero Python: `models/account_general_ledger.py` (437 líneas v18).
- Hace `_inherit = 'account.general.ledger.report.handler'` (AbstractModel; clase `GeneralLedgerCustomHandler`, `account_saft/models/account_general_ledger.py:10-11`, `class GeneralLedgerCustomHandler`).
- SAF-T **reutiliza el report `account.report` del Libro Mayor (General Ledger)**. No existe un `account.report` dedicado a SAF-T en `data/` — solo `data/saft_report.xml`, que es una plantilla QWeb (`<template>`), no un registro `account.report`.

## Report engine — cómo se genera el XML

El flujo es el patrón estándar de "export button" de `account_reports`:

1. **Botón**: la localización añade en `_custom_options_initializer` un botón a `options['buttons']` con `action='export_file'` y `action_param='<l10n>_export_saft_to_xml'` (ej. `l10n_no_saft/models/account_general_ledger.py:17-23`, `GeneralLedgerCustomHandler._custom_options_initializer`). El botón solo aparece si `company.account_fiscal_country_id.code == '<PAÍS>'`.
2. **Preparación de valores**: el método de export llama a `_saft_prepare_report_values(report, options)` (base, `account_saft/models/account_general_ledger.py:404`, `GeneralLedgerCustomHandler._saft_fill_report_partner_ledger_values`), que orquesta 4 fills:
   - `_saft_fill_report_general_ledger_accounts` → `account_vals_list` (plan de cuentas + saldos apertura/cierre).
   - `_saft_fill_report_general_ledger_entries` → `journal_vals_list` + `move_vals_list` + totales debe/haber.
   - `_saft_fill_report_tax_details_values` → `tax_vals_list` + detalle de impuesto por línea.
   - `_saft_fill_report_partner_ledger_values` → `customer_vals_list`, `supplier_vals_list`, `partner_detail_map`.
3. **Override de localización**: `_l10n_<país>_prepare_saft_report_values` hace `super()` y luego inyecta `xmlns`, `file_version`, `accounting_basis` y datos país-específicos.
4. **Render**: `report._generate_file_data_with_error_check(options, self.env['ir.qweb']._render, {'values': ..., 'template': 'l10n_X_saft.saft_template_inherit_X', 'file_type': 'xml'}, errors)` (`l10n_no_saft/models/account_general_ledger.py:76-82`, `GeneralLedgerCustomHandler.l10n_no_export_saft_to_xml`). Devuelve `file_data` que el framework ofrece como descarga.

## Datos requeridos (qué consulta el motor)

Las queries son **SQL crudo** (no ORM) por volumen. Tablas implicadas:

- **Cuentas** (`_saft_fill_report_general_ledger_accounts`): `account.account` (code, name, account_type) + saldos calculados vía `_query_values` / `_get_initial_balance_values` del report engine. En v19 esto cambia a `_get_report_values` que parsea las líneas del report (ver `v19_changes.md`).
- **Asientos** (`_saft_fill_report_general_ledger_entries`, `account_saft/models/account_general_ledger.py:93-225`, `GeneralLedgerCustomHandler._saft_fill_report_general_ledger_entries`): gran SELECT sobre `account_move_line` JOIN `account_move`, `account_journal`, `account_account`, `res_currency`, LEFT JOIN `product_product`/`product_template`/`uom_uom`/`account_tax`. Agrupa por journal → move → line. Procesa en batches de `10**4` con `dictfetchmany`.
- **Impuestos** (`_saft_fill_report_tax_details_values`, `:227-274`): usa `account.move.line._get_query_tax_details(...)` para obtener base + cuota por línea base; agrupa en `tax_vals_map`.
- **Partners** (`_saft_fill_report_partner_ledger_values`, `:276-401`): SELECT con `FILTER` para opening/closing receivable/payable por partner (cuentas `asset_receivable`/`liability_payable`, más `asset_fixed`/`asset_non_current` para depreciaciones). Construye `partner_detail_map` con direcciones (zip+city dedup) y contactos.

Reglas de contactos (`:360-389`):
- Partner individual → es su propio ContactPerson.
- Partner empresa → el hijo `type='contact'` con menor ID es el ContactPerson.
- La propia company → TODOS los hijos contacto son ContactPersons (flexibilidad para varios firmantes); si no hay ninguno → error bloqueante `missing_company_contact`.

## Plantilla QWeb (`data/saft_report.xml`, 261 líneas v18)

Plantillas reutilizables que las localizaciones heredan con `inherit_id=... primary="True"`:

- `address` — bloque de dirección de un partner.
- `addresses_contacts` — Name + Address(es) + Contact(s) + TaxRegistration de un partner.
- `company_header` — RegistrationNumber + addresses_contacts + BankAccount (IBAN vs cuenta normal).
- `line_debit_credit_amount` — DebitAmount/CreditAmount con conversión de divisa (CurrencyCode/CurrencyAmount/ExchangeRate si la moneda de la línea ≠ moneda company).
- `tax_information` — TaxCode/TaxPercentage/TaxBaseDescription/TaxAmount.
- `saft_template` — raíz `<AuditFile>`: `Header` (versión, país `company.account_fiscal_country_id.code`, "Odoo SA"/"Odoo", periodo, `TaxAccountingBasis`) + `MasterFiles` (GeneralLedgerAccounts, Customers, Suppliers, TaxTable, Owners) + `GeneralLedgerEntries` (NumberOfEntries, TotalDebit, TotalCredit, Journal → Transaction → Line).

Las variables `xmlns`, `file_version`, `accounting_basis` se renderizan vacías por defecto (`account_saft/models/account_general_ledger.py:419-421`, `GeneralLedgerCustomHandler._saft_prepare_report_initial_values`: `'undefined'`/`''`) y SOLO las rellena la localización. Por eso el base no produce un XML válido.

## Helpers de validación / warnings

- `_customize_warnings` (`account_saft/models/account_general_ledger.py:13-27`, `GeneralLedgerCustomHandler._customize_warnings`): warning no bloqueante si falta company_registry, teléfono o ciudad/zip de la company. Botón `action_fill_company_details` abre el form de `res.company`.
- `_saft_get_account_type(account_type)` (`:42-47`): hook que devuelve `False` por defecto; las localizaciones que necesitan un AccountType propio lo sobreescriben.
- `errors` dict: se acumula durante los fills y se pasa a `_generate_file_data_with_error_check`; los de `level='danger'` bloquean el export.

## `_saft_get_account_type` — AccountType por país (hook)

`_saft_get_account_type(account_type)` (`:42-47`) devuelve `False` en el base → la plantilla NO emite `<AccountType>`. Cada localización lo sobreescribe con su propia taxonomía:
- **NO**: devuelve siempre `"GL"` (ignora el `account_type`; ojo, el override declara la firma como `(self, account)` aunque el base le pasa el `account_type` string, `l10n_no_saft/models/account_general_ledger.py:84-88`, `GeneralLedgerCustomHandler`).
- **LT**: mapea los `account_type` de Odoo a 7 códigos LT (`IT`/`TT`/`NK`/`I`/`P`/`S`/`KT`) vía dict; si llega un `account_type` no contemplado → **KeyError** (`l10n_lt_saft/models/account_general_ledger.py:130-157`, `GeneralLedgerCustomHandler`).
- **RO**: `Activ` / `Pasiv` / `Bifunctional` según listas de tipos (`l10n_ro_saft/models/account_general_ledger.py:593-606`, `GeneralLedgerCustomHandler._l10n_ro_saft_fill_payment_values`).
- **AT**: no usa este hook; mapea por `tag_ids` numéricos del account → registro `l10n_at_saft.account` → tipo `1/2/3/4` (Bestandskonto/Aufwandskonto/Erlöskonto), `l10n_at_saft/models/account_general_ledger.py:37-68`, `GeneralLedgerCustomHandler._l10n_at_prepare_saft_report_values`, `l10n_at_saft.account`.

## Flujo de IMPORT (`account_saft_import`)

Wizard `account.saft.import.wizard` (`TransientModel`, `account_saft_import/wizard/import_wizard.py`). Importa **accounts, taxes, partners, journals, moves** desde un XML SAF-T externo (típico para arrancar historia contable de una empresa migrada). depende de `account_saft` + `account_base_import`. Botón en Contabilidad.

Pipeline (`_get_data` → `action_import`):
1. Parsea el XML con `lxml.etree`; limpia el namespace asumiendo `urn:StandardAuditFile-Taxation-Financial...` (hardcoded, `:46`) → solo entiende ese esquema genérico, NO los namespaces nacionales propios (AT/RO/NO/LT exportan con otro xmlns).
2. `_prepare_account_data`: matchea por `code` contra el CoA existente; crea los que faltan con `account_type` mapeado por `_get_account_types()` (vacío en base; cada `l10n_<país>_saft_import` lo rellena, ej. DK `Asset→asset_current`, `Sale→income`). Default si no mapea: `asset_current`.
3. `_prepare_tax_data`: dedup por `(name/description, amount_type, amount)`; crea las nuevas en un tax group "SAF-T taxes".
4. `_prepare_partner_data`: dedup por `(name, vat)`; crea contactos hijos (ignora `FirstName=='NotUsed'`, el placeholder que pone el export).
5. `_prepare_journal_data` + `_prepare_move_data`: crea journals por código y moves; dedup por xml_id `{company}_move_{journal}_{name}` → **reimportar el mismo fichero NO duplica** moves/partners/cuentas.
6. Opcional `import_opening_balance`: crea un asiento de ajuste con la diferencia entre el `OpeningDebit/CreditBalance` del XML y el saldo real en Odoo a la `SelectionStartDate`.
7. Antes de cargar fija `sequence.mixin.constraint_start_date` al move más antiguo para que las secuencias no rechacen fechas históricas. Carga vía `account.chart.template._load_data(..., skip_invoice_sync=True)`.

`l10n_dk_saft_import` (DK), `l10n_lt_saft_import`, `l10n_ro_saft_import` son bridges finos: solo sobreescriben `_get_account_types()`.

## Restricción importante

`_saft_prepare_report_values` lanza `UserError` si `len(options["column_groups"]) > 1`: **SAF-T solo admite un único column group** (`account_saft/models/account_general_ledger.py:412-413`, `GeneralLedgerCustomHandler.format_date`). Es decir, no se puede exportar comparando dos periodos/companies a la vez.
