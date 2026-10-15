# Accounting v18 -> v19: diffs para devs que extienden `account`

> Las citas son recibos, no adorno: `file::symbol` gana al número de línea (las líneas
> derivan; los símbolos aguantan). Snapshot v18/v19 verificado contra fuente local 2026-07
> (resolver vía `references/shared/odoo-source.md`; NO asumir rutas de máquina). Donde algo NO está verificado, se dice
> explícitamente — no se ha omitido en silencio.

**TL;DR para migrar un override de `account` a v19:**
1. Cambiar `odoo.osv.expression` por `Domain` (`from odoo.fields import Domain`, o `odoo.orm.domains.Domain`); importar `Command`/`Domain` desde `odoo.fields`.
2. Convertir `_sql_constraints` (lista de tuplas) a `models.Constraint(...)`.
3. Si extendías el conciliador vía el TransientModel `bank.rec.widget` / `.line`: **ese modelo ya no existe en v19**; los overrides Python romperán.
4. API núcleo de `account.move` / `account.tax` / motor `account_reports`: estable, sin renombrados en el subconjunto revisado.
5. Widget `account-tax-totals-field` sin cambios; ojo a renombrados de componentes JS auxiliares.

---

## 1. ORM / base

| Área | v18 | v19 | Impacto en un override |
|---|---|---|---|
| Paquete ORM | No existe `odoo/orm/`; dominios en `odoo/osv/expression.py` (1467 líneas) | Nace `odoo/orm/` (`fields_*.py`, `decorators.py`, `domains.py`, `models.py`, `models_transient.py`) | La clase base NO cambia: sigue `from odoo import api, fields, models` y `class AccountMove(models.Model)`. Solo cambia de dónde importas utilidades. |
| Combinar dominios | `from odoo.osv import expression` | `from odoo.fields import Command, Domain`; `class Domain` en `odoo/orm/domains.py::Domain` (+ `DomainAnd`, `DomainOr`, `DomainCondition`) | Código nuevo/migrado usa `Domain` en vez de `expression.AND/OR`. `odoo/osv/expression.py` **sigue existiendo** en v19 pero reducido a 473 líneas (shim de compat) — funciona, pero el core ya no lo usa. |
| `class Domain` | **No existe** (`odoo/fields.py` v18 no la define) | Novedad v19 | No uses `Domain` si el módulo debe correr también en v18. |
| Imports en `account_move.py` | `from odoo import api, fields, models, _, Command, SUPERUSER_ID, modules, tools` + `from odoo.osv import expression` (L18) | `from odoo.fields import Command, Domain`; **elimina** `from odoo.osv import expression` | `from odoo import Command` sigue reexportado, pero el propio `account` pasó a `from odoo.fields import ...`. |

Citas: `account/models/account_move.py::imports` (v18 L18 / v19 `from odoo.fields import Command, Domain`); `odoo/orm/domains.py::Domain` (v19); `odoo/osv/expression.py` (v18 1467 líneas -> v19 473 líneas); `odoo/fields.py` (v18 sin `class Domain`).

## 2. `account.move` / `account.move.line`

| Método | v18 | v19 | Impacto |
|---|---|---|---|
| `account.move::_post(self, soft=True)` | L5101 | L5524 | Misma firma; super-override estable. |
| `account.move::action_post(self)` | L5558 | L6112 | Sin cambios. |
| `account.move::button_draft(self)` | L5605 | L6201 | Sin cambios. |
| `account.move.line::_compute_amount(self)` | L1132 | L1166 | Sin cambios. |
| `_compute_amount_total_words`, `_post_process_link_to_purchase_order(self, invoice)` | presentes | presentes, misma firma | Sin cambios. |

Los métodos núcleo mantienen nombre y firma idénticos. **Honestidad:** solo se revisó un subconjunto representativo; NO se ha auditado exhaustivamente los cientos de métodos del fichero, así que no puedo afirmar categóricamente que no exista ningún rename fuera de este subconjunto.

Citas: `account/models/account_move.py` (símbolos arriba, v18 y v19).

## 3. `account.tax`

| Método | v18 | v19 | Impacto |
|---|---|---|---|
| `_prepare_base_line_for_taxes_computation(self, record, **kwargs)` | L1367 | L1593 | Idéntico. |
| `_add_tax_details_in_base_line(self, base_line, company, rounding_method=None)` / `_add_tax_details_in_base_lines(self, base_lines, company)` | presentes | presentes, misma firma | Idéntico. |
| `_get_tax_totals_summary(self, base_lines, currency, company, cash_rounding=None)` | L2738 | L2715 | Idéntico. Claves del dict de salida (`base_amount`, `tax_amount`, `subtotals`, `has_tax_groups`...) conservadas. |
| `compute_all(self, price_unit, currency=None, quantity=1.0, product=None, partner=None, is_refund=False, handle_price_include=True, include_caba_tags=False, rounding_method=None)` | L4177 | L4960 | Idéntico. |
| `_prepare_base_lines_for_down_payment(...)` | — | L3984 (**nuevo**) | Adición, NO rename. |

La API de cálculo de impuestos introducida en v18 se mantiene idéntica en v19. Único delta: v19 **añade** `_prepare_base_lines_for_down_payment`.

Citas: `account/models/account_tax.py` (símbolos arriba, v18 y v19).

## 4. Widget de conciliación bancaria (`bank.rec.widget`)

**Aviso:** la premisa "dir `bank_rec_widget` (v18) -> `bank_reconciliation` (v19)" es **incorrecta**: la carpeta se llama `bank_reconciliation` en AMBAS versiones (`account_accountant/static/src/components/bank_reconciliation/`). Lo que cambió es la **arquitectura**.

| Área | v18 (modelo servidor / transient) | v19 (servicio cliente) | Impacto |
|---|---|---|---|
| Modelo | `bank.rec.widget` (TransientModel) + `bank.rec.widget.line` | Ficheros `bank_rec_widget.py` y `bank_rec_widget_line.py` **eliminados**; el modelo transient ya no existe | **Overrides Python de `bank.rec.widget` romperán.** La lógica pasa a `account.bank.statement.line` + servicio OWL. |
| Front | OWL monolítico: `bank_rec_form.xml`, `bank_rec_record.js` (`BankRecRecord`, `BankRecRelationalModel`), `embedded_list_view.js`, `view_embedder.js`, `global_info.js`, `finish_buttons.js`, `kanban.js` | Servicio OWL: `bank_reconciliation_service.js` -> `registry.category("services").add("bankReconciliation", ...)`; componentes en subcarpetas (`button/`, `list_view/`, `statement_line/`, `statement_summary/`, `search_dialog/`, `quick_create/`, `apply_amount/`...) | Extensiones JS al form monolítico no encajan; hay que enganchar al servicio/componentes nuevos. |
| Vistas | `views/bank_rec_widget_views.xml` sobre `bank.rec.widget` | mismas vistas apuntan a `account.bank.statement` y `account.move.line` | Herencia de vista por `bank.rec.widget` rota. |

Registries (claves de views/fields mayormente preservadas):

| Registry | v18 | v19 |
|---|---|---|
| `bank_rec_widget_kanban` | `account_accountant/static/src/components/bank_reconciliation/kanban.js:1243`, `bank_rec_widget_kanban` | `account_accountant/static/src/components/bank_reconciliation/kanban_renderer.js:234`, `bank_rec_widget_kanban` |
| `bank_rec_list` | `account_accountant/static/src/components/bank_reconciliation/list.js:37`, `bank_rec_list` | `list_view/list.js:79` |
| `bank_rec_list_many2one_multi_id` | `many2one_field_multi_edit.js` | `list_view/list_view_many2one_multi_edit.js` |
| Solo v18 | `bank_rec_amls_list_view`, `bank_rec_list_view_switcher` | — |
| Nuevos v19 | — | `bank_rec_dialog_list`, `bank_rec_dialog_kanban`, `bankrec_edit_line`, `apply_amount_html` |

Citas: v18 `account_accountant/models/bank_rec_widget.py::bank.rec.widget` (L15), `models/bank_rec_widget_line.py`; v19 (ausencia de esos ficheros), `.../bank_reconciliation/bank_reconciliation_service.js::bankReconciliationService` (L215), `views/bank_rec_widget_views.xml` (records sobre `account.bank.statement` / `account.move.line`).

**No verificado:** la superficie de métodos expuestos por el servicio v19 `bankReconciliationService` (solo se verificó su registro en `services`, no los métodos). Tampoco se localizó el script `migrations/` que borra el modelo `bank.rec.widget` — la eliminación se deduce por ausencia de ficheros y por las vistas v19, no por un script de upgrade leído.

## 5. Motor de informes (`account_reports`)

| Elemento | v18 | v19 | Impacto |
|---|---|---|---|
| Dispatch `engine_function_name = f'_compute_formula_batch_with_engine_{formula_engine}'` | presente | presente | Sin cambios. |
| Los 5 engines `_compute_formula_batch_with_engine_{tax_tags,domain,account_codes,external,custom}(self, options, date_scope, formulas_dict, current_groupby, next_groupby, offset=0, limit=None, warnings=None)` | L3782-4352 | L3947-4507 | Mismo nombre y firma exacta. |
| `account.report::_get_lines(self, options, all_column_groups_expression_totals=None, warnings=None)` | L2614 | L2733 | Mismo nombre/firma. |
| `_report_expand_unfoldable_line_with_groupby` | presente | presente | Sin cambios. |

El motor de fórmulas es idéntico v18->v19: NO se detecta rename de engine ni de método público del motor.

Citas: `account_reports/models/account_report.py` (v18 L2614/L3776-4352 / v19 L2733/L3941-4507).

## 6. EDI (`account_edi`)

| Área | v18 | v19 | Impacto |
|---|---|---|---|
| Existencia del framework | `account_edi`, `account_edi_proxy_client`, `account_edi_ubl_cii` | **Siguen existiendo** en v19 CE (`addons/account_edi`) | El framework NO se ha eliminado. |
| Manifest | `installable: True`, `LGPL-3`, `data`: `account_edi_document_views.xml`, `account_move_views.xml`, `account_journal_views.xml`, el `cron.xml` de datos | Casi idéntico; único cambio: añade `'author': 'Odoo S.A.'` | Sin impacto real de instalación. |
| Constraints en `account.edi.format` | `_sql_constraints` (lista de tuplas) | `_unique_code = models.Constraint('unique (code)', 'This code already exists')` | **Regla general v19**: `_sql_constraints` lista deja de aplicarse. Migrar a `models.Constraint`. |
| Imports | con `-*- coding -*-` y `from odoo.osv import expression` | reordena: `from odoo import api, fields, models`; quita coding y `expression` | Cosmético, alinea con la regla ORM. |
| `account.edi.format::_get_move_applicability` (hook) | vigente | vigente | Sin cambios. |

Citas: `account_edi/__manifest__.py` (v18 vs v19); `account_edi/models/account_edi_format.py::account.edi.format` (`_sql_constraints` v18 -> `_unique_code = models.Constraint(...)` v19).

**No verificado (EDI):**
- Estado de deprecación funcional de `account_edi` más allá del manifest (p.ej. si `account.move` dejó de usar `edi_document_ids` en algún flujo): solo se confirmó que módulo y modelo `account.edi.format` siguen instalables/presentes.
- `account_edi_proxy_client` EE v18 vs v19: no abierto, solo listado de directorios.
- Localización española (`l10n_es_edi_tbai` / `l10n_es_edi_verifactu` / `l10n_es_edi_facturae` / `l10n_es_edi_sii`): confirmado que existen en CE `addons/` v19, pero **NO** verificada su presencia/diffs frente a v18 ni su dependencia de `account_edi`. TicketBAI/Verifactu/Facturae/SII **no auditados**.

## 7. XML / vistas / widgets JS de `account`

| Elemento | v18 | v19 | Impacto |
|---|---|---|---|
| Widget tax_totals `registry.category("fields").add("account-tax-totals-field", taxTotalsComponent)` | `account/static/src/components/tax_totals/tax_totals.js:175` | `account/static/src/components/tax_totals/tax_totals.js:173` | **Sin cambios** en la clave. |
| Componente `many2many_tax_tags` | presente | renombrado a `many2x_tax_tags` | Ajustar imports/registro. |
| `autosave_many2many_tags` | presente | renombrado a `autosave_many2many_tax_tags` | Ajustar. |
| `account_many2one_barcode` | presente | **eliminado** | Roto en v19. |
| `json_checkboxes` | `account/static/src/components/json_checkboxes/json_checkboxes.js:42`, `account_json_checkboxes` | **se mudó, no se eliminó**: `web/static/src/views/fields/json_checkboxes/json_checkboxes_field.js:58`, mismo nombre de widget `account_json_checkboxes` y con `// TODO: remove in saas~19.1` | El widget **sigue funcionando** en v19. Lo que rompe es un `import` desde `@account/...`: cambia la ruta, no el nombre. Corregido 2026-08-15 — esta fila decía «eliminado, roto en v19». |
| `journal_dashboard_activity` | **solo en ficheros `.po`** (64 traducciones + 1 `.pot`), cero JS/XML/PY | igual | No era un widget vivo en v18, así que no hay nada que se rompa al migrar. Corregido 2026-08-15 — estaba listado como «presente en v18». |
| Nuevos v19 | — | `many2one_account_account`, `many2many_tags_banks`, `many2many_tags_journals`, `fetch_einvoices`, `receipt_selector`, `currency_form`, `char_with_placeholder_field_to_check` | Disponibles para reutilizar. |
| `<list>` (antes `<tree>`) | ya `<list>` en v18 | `<list>` | **NO es diff v18->v19** (el cambio `<tree>`->`<list>` fue en v18). |

Citas: `account/static/src/components/tax_totals/tax_totals.js::account-tax-totals-field` (v18 L175 / v19 L173); listado `account/static/src/components/` (v18 vs v19).

---

## Qué NO se pudo verificar (resumen honesto)
- Auditoría **exhaustiva** de todos los métodos de `account.move` y `account.tax`: solo subconjunto representativo.
- Superficie de métodos del servicio v19 `bankReconciliationService`.
- Localización española EDI (TBAI/Verifactu/Facturae/SII): existencia sí, diffs no.
- Deprecación funcional de `account_edi` más allá del manifest.
- `account_edi_proxy_client` EE v18 vs v19.
- Script `migrations/` concreto que elimina `bank.rec.widget`: la eliminación se **deduce** por ausencia de ficheros + vistas v19, no por script leído.
