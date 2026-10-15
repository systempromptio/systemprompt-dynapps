<!-- verified: v19.0 @ 2026-06 (diff odoo_enterprise/18 vs odoo_enterprise/19 account_saft + l10n_*_saft) -->
# account_saft — v19 changes

Diff verificado entre `odoo_enterprise/18/account_saft/` y `odoo_enterprise/19/account_saft/`. Líneas referidas al fichero v19 salvo indicación.

## Contexto España (sin cambios)

España sigue **sin SAF-T en v19**: `ls odoo_enterprise/19/ | grep saft` no incluye `l10n_es_saft`. Mismo stack AEAT (SII / VeriFactu / TicketBAI). SAFTG1 sigue vigente.

## Manifest

Único cambio: añade `'author': 'Odoo S.A.'` (`account_saft/__manifest__.py:24`). Mismo `depends=['account_reports']`, mismo `version='1.0'`, mismo `OEEL-1`.

## Clase renombrada

`GeneralLedgerCustomHandler` → **`AccountGeneralLedgerReportHandler`** (`account_saft/models/account_general_ledger.py:10`, `class GeneralLedgerCustomHandler`). Sigue siendo `_inherit = 'account.general.ledger.report.handler'`. Si tienes un bridge custom que hace `_inherit` por nombre de clase Python (no por `_name`), revisa — el `_inherit` del modelo no cambia, solo el nombre de la clase.

## Cálculo de saldos de cuentas — REESCRITO

Cambio más relevante. En v18 los saldos de apertura/cierre del plan de cuentas se obtenían vía:
- `_get_accounts_results` → `_query_values(report, options)`
- `_get_accounts_initial_balances_map` → `_get_initial_balance_values(report, account_ids, options)`

Ambos **eliminados en v19**. Se sustituyen por un único método nuevo:

```python
def _get_report_values(self, report, options, values=None):
    options = report.get_options(previous_options={**options,
        'export_mode': 'file', 'ignore_totals_below_sections': True, 'unfold_all': True})
    lines = report._get_lines(options)
    ...
    for line in lines:
        model, res_id = report._get_model_info_from_id(line['id'])
        if model == 'account.account':
            ... report_values[current_account_id]['sum'] = {debit, credit, balance}
        elif values and report._get_markup(line['id']) == 'undistributed_profits_losses':
            ... values['errors']['undistributed_earnings'] = {...}
        if isinstance(res_id, str) and 'balance_line' in res_id:
            report_values[current_account_id]['initial_balance'] = ...
```

Es decir: v19 ya **no recalcula con queries propias**, sino que **parsea las líneas renderizadas del report** (`report._get_lines`) con `unfold_all` y `ignore_totals_below_sections`. Más alineado con lo que el usuario ve en pantalla, pero depende del formato de líneas del report engine.

`_get_formatted_account_vals_list(accounts, report_values)` cambia firma: ya no recibe `(account_results, initial_balances_map)` sino `(accounts, report_values)`, y lee `report_values[account.id]['initial_balance']` y `['sum']['balance']` (`:88-100`). `_saft_fill_report_general_ledger_accounts` ahora hace `browse(report_values.keys())`.

**Impacto en bridges**: si una localización sobreescribía `_get_accounts_results` o `_get_accounts_initial_balances_map`, esos métodos ya no existen — hay que migrar al nuevo `_get_report_values`.

## Detección de undistributed earnings (nuevo warning)

`_get_report_values` detecta líneas con markup `undistributed_profits_losses` y, si el balance no es cero, añade `values['errors']['undistributed_earnings']` con `level='info'` y mensaje *"Undistributed profits or losses may cause export rejection."* (`:74-81`). Es informativo (no bloquea), pero avisa de un motivo común de rechazo por la administración.

## `_saft_prepare_report_values` partido en dos

v18 tenía un único `_saft_prepare_report_values`. v19 lo separa:
- **`_saft_prepare_report_initial_values(options, values)`** (`:420`): rellena `company`, `xmlns`, `file_version`, `accounting_basis`, fechas, formatters, `errors` en el dict `values` recibido.
- **`_saft_prepare_report_values(report, options)`** (`:440`): valida column group, `_init_currency_table`, fija `single_column_group` con `next(iter(...))` (antes `tuple(...)[0]`), crea `template_values={}`, llama a `_saft_prepare_report_initial_values` y luego a los 4 fills.

Hook nuevo para extensión más limpia de la cabecera por país.

## `_get_all_partners` (nuevo hook)

Nuevo método `_get_all_partners(self, values, balance_result)` (`:289`) que encapsula `browse([partner_id for partner_id, *__ in balance_result])`. `_saft_fill_report_partner_ledger_values` lo llama en `:338` en vez de hacer el browse inline. Permite a las localizaciones inyectar/filtrar partners adicionales sin reescribir todo el fill.

## Domain de partners amplía account_type

El domain de selección de customers/suppliers añade `'asset_current'`:
- v18: `('account_id.account_type', 'in', ('asset_fixed', 'asset_non_current'))`
- v19: `('account_id.account_type', 'in', ('asset_fixed', 'asset_current', 'asset_non_current'))` (`:311`)

Más cuentas de activo entran en el cálculo de partners (relevante para depreciaciones/activos corrientes).

## `_customize_warnings`: solo phone (no mobile)

- v18 avisaba si faltaba `phone OR mobile`.
- v19 avisa solo si falta `phone` (`:18-19`). Mensaje: "the phone number" (antes "the phone or mobile number").

Coherente con el cambio de plantilla (ver abajo). En v19 el campo `mobile` deja de considerarse para SAF-T.

## `self._cr` → `self.env.cr`

Sustitución sistemática de `self._cr` por `self.env.cr` en las queries (`:185,191,257,269`) y eliminación del uso de `tax_description` (ver abajo). Cambio de estilo/API recomendado en v19.

## Plantilla `saft_report.xml` (data)

Dos cambios (diff confirmado):
1. **Se elimina `<Title>`** del ContactPerson (v18 tenía `<Title t-if="partner_contact.title" t-out="partner_contact.title.name"/>`).
2. **`<Telephone>` solo usa `phone`**: v18 `t-if="phone or mobile" t-out="(phone or mobile)[:18]"`; v19 `t-if="phone" t-out="(phone)[:18]"`. El `mobile` desaparece del XML.

## tax_description eliminado del tax detail

En v18, `_saft_fill_report_tax_details_values` seleccionaba `tax.description` (`tax_description = _field_to_sql('tax','description')`) y lo metía en `tax_vals_map[...]['description']`. **v19 lo elimina** de la query y del map (`:250,268`). Las localizaciones que dependían de `tax_vals['description']` (ej. NO parsea el standard code de ahí) deben verificar — en v19 ese dato ya no viene del base.

## NO: campo dedicado `l10n_no_standard_code` (resuelve el hack de SAFTG8)

Cambio relevante para Noruega. En v18 el "standard tax code" se parseaba con regex del **nombre/descripción** del impuesto (SAFTG8, era un hack con `TODO`). En v19 ya hay un **campo dedicado** `account.tax.l10n_no_standard_code`: el export hace `.mapped('l10n_no_standard_code')` y si falta lanza `UserError("Please set up standard tax codes for your Norwegian taxes.")` (`l10n_no_saft/models/account_general_ledger.py:29-33`, `GeneralLedgerCustomHandler._l10n_no_prepare_saft_report_values`). Ya NO se renombran descripciones de impuesto. Coherente con la eliminación de `tax_vals['description']` del base (arriba). **SAFTG8 aplica solo a v18.**

## Localizaciones — v19

- Las firmas de los métodos `_l10n_<país>_*` cambian para seguir el nuevo `_saft_prepare_report_initial_values` / `_get_report_values`.
- **Nuevo módulo `l10n_ro_saft_stock`** (solo v19): genera la declaración **D.406 de stocks** (inventario) en formato SAF-T. `depends: account_edi_ubl_cii, l10n_ro_saft, stock_account`. Añade `data/saft_report.xml`, `views/stock_picking_type_views.xml`, demo de stock.
- Resto de localizaciones (`l10n_no_saft`, `l10n_lt_saft`, `l10n_at_saft`, `l10n_pl_reports_account_saft`, imports DK/LT/RO) presentes igual que en v18.

## Resumen de impacto para migración de bridges custom

| Cambio | Acción si tienes bridge custom |
|--------|-------------------------------|
| `_get_accounts_results`/`_get_accounts_initial_balances_map` eliminados | migrar a `_get_report_values` |
| `_saft_prepare_report_values` partido | mover overrides de cabecera a `_saft_prepare_report_initial_values` |
| `_get_all_partners` nuevo | usar este hook en vez de reescribir el fill de partners |
| `tax_vals['description']` eliminado | recalcular si dependías de él |
| plantilla quita Title / mobile | revisar xpaths que apuntaran a esos nodos |
| clase renombrada | revisar `_inherit` por clase Python |
