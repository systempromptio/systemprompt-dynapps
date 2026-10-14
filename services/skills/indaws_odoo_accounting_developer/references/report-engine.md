# account_reports engine (EE) — `account.report`

> Nota de lectura: las citas `file::symbol` (y líneas) son **recibos**, no adornos. Ante conflicto,
> el símbolo manda sobre el número de línea (las líneas derivan de un snapshot). Snapshot v18/v19
> verificado 2026-07. Lo que NO se pudo verificar está marcado explícitamente como tal — no se ha
> silenciado nada. **Todo esto es EE**: requiere el módulo `account_reports`.

## 0. Dónde vive cada cosa (leer primero)

Reparto CE/EE que confunde a todo el mundo:

- Los **modelos de datos** (`account.report`, `.line`, `.column`, `.expression`, `.external.value`)
  se definen en el módulo **CE `account`** → `account/models/account_report.py`. Verificado:
  `account/models/account_report.py::AccountReport` (v18 CE línea 25, `_name = "account.report"`;
  v19 CE línea 45).
- El **motor de cálculo** (evaluación de expresiones, engines, unfold, opciones, custom handlers) y
  la **acción cliente OWL** viven en **EE `account_reports`** →
  `account_reports/models/account_report.py` (≈7778 líneas en v18 / ≈8148 en v19). EE hace
  `_inherit = 'account.report'` y añade todos los `_compute_formula_batch_with_engine_*`,
  `_get_lines`, `_get_custom_report_function`, etc. Los `_report_custom_engine_*` **solo** existen en EE.

Consecuencia práctica: para tocar la *estructura* miras `account`; para tocar *cómo se calcula*
miras `account_reports`.

## 1. El cuarteto de modelos (definidos en CE `account`)

### `account.report` — `AccountReport` (`_order='sequence, id'`)
Campos núcleo verificados (`account/models/account_report.py::AccountReport`, v18 líneas 24-59):
- `name` (Char, required, translate), `sequence`, `active`.
- `line_ids` One2many → `account.report.line` (inverse `report_id`).
- `column_ids` One2many → `account.report.column` (inverse `report_id`).
- `root_report_id` Many2one → `account.report` ("report this report is a variant of") +
  `variant_report_ids` (inverse) — mecanismo de **variantes**.
- `section_report_ids` / `section_main_report_ids` Many2many (relación `account_report_section_rel`)
  + `use_sections` — **informes compuestos** por secciones.
- `chart_template`, `country_id`, `availability_condition`
  (`[('country'),('coa'),('always')]`).
- `load_more_limit`, `search_bar`, `prefix_groups_threshold` (default 4000), `integer_rounding`.
- Batería de `filter_*` (compute `_compute_report_option_filter`): `filter_multi_company`
  (`disabled`/`selector`/`tax_units`), `filter_date_range`, `filter_show_draft`,
  `filter_unreconciled`, `filter_unfold_all`, `filter_hide_0_lines`, `filter_period_comparison`, …
- `default_opening_date_filter`, `currency_translation` (`current`/`cta`).
- **EE añade** por herencia: `custom_handler_model_id` Many2one → `ir.model` y
  `custom_handler_model_name` (related) — EE v18 líneas 98-99.

### `account.report.line` — `AccountReportLine` (`_order='sequence, id'`)
Verificado (`account/models/account_report.py::AccountReportLine`, v18 líneas 310-355):
- `name` (required, translate), `expression_ids` One2many → `account.report.expression`.
- `report_id` (compute recursivo, ondelete cascade), `parent_id`/`children_ids` (jerarquía),
  `hierarchy_level` (compute).
- `groupby` (Char, "Comma-separated list of fields from account.move.line") y `user_groupby` (compute).
- `code` (identificador único por report; sql_constraint `code_uniq` UNIQUE(report_id, code)),
  `sequence`, `foldable`, `print_on_new_page`, `action_id` (→ `ir.actions.actions`; convierte la
  línea en enlace), `hide_if_zero`, `horizontal_split_side` (`left`/`right`).
- **Atajos de fórmula** — campos `store=False` con inverse que crean expresiones vía
  `_create_report_expression(engine=...)` (`account/models/account_report.py::AccountReportLine._create_report_expression`,
  v18 líneas 466-524): `domain_formula`, `account_codes_formula`, `aggregation_formula`,
  `external_formula`, `tax_tags_formula`. Ej.: `domain_formula` usa
  `DOMAIN_REGEX = re.compile(r'(-?sum)\((.*)\)')` → `subformula='sum'`, `formula`=el dominio.

### `account.report.column` — `AccountReportColumn` (`_order='sequence, id'`)
`account/models/account_report.py::AccountReportColumn` (v18 líneas 869-881):
- `name` (required, translate), `expression_label` (Char, required — **vincula la columna a la label
  de una expresión**), `sequence`, `report_id`, `sortable`, `figure_type` (default `'monetary'`,
  required), `blank_if_zero`, `custom_audit_action_id` (→ `ir.actions.act_window`).

### `account.report.expression` — `AccountReportExpression` (`_rec_name='report_line_name'`)
`account/models/account_report.py::AccountReportExpression` (v18 líneas 537-595):
- `report_line_id` (required, ondelete cascade), `label` (required), `formula` (Char, required),
  `subformula`, `figure_type`, `green_on_positive`, `blank_if_zero`, `auditable` (compute),
  `carryover_target`.
- sql_constraints: `domain_engine_subformula_required`
  (CHECK `engine != 'domain' OR subformula IS NOT NULL`) y `line_label_uniq`
  (UNIQUE(report_line_id, label)).

### `account.report.external.value` — `AccountReportExternalValue`
`account/models/account_report.py::AccountReportExternalValue` (v18 líneas 884-918;
`_check_company_auto=True`, `_order='date, id'`): `name`, `value` (Float), `text_value`,
`date` (required), `target_report_expression_id` → expresión (required, ondelete cascade),
`company_id`, `foreign_vat_fiscal_position_id`, campos de carryover. Es el **almacén** del engine
`external`.

**`FIGURE_TYPE_SELECTION_VALUES`** (constante del módulo): `monetary`, `percentage`, `integer`,
`float`, `date`, `datetime`, `boolean`, `string`.

## 2. El campo `engine` de la expresión — los 6 valores (idénticos v18/v19)

`account.report.expression.engine` (Selection, required). Verificado v18 CE líneas 545-556 y
v19 CE líneas 587-597:

| engine | etiqueta | qué hace |
|---|---|---|
| `domain` | Odoo Domain | suma AMLs que casan un dominio; **requiere `subformula`** (`sum`, `-sum`, …) |
| `tax_tags` | Tax Tags | saldos por etiquetas de impuesto |
| `aggregation` | Aggregate Other Formulas | combina resultados de otras líneas por `code.label` |
| `account_codes` | Prefix of Account Codes | por prefijos de código de cuenta |
| `external` | External Value | lee `account.report.external.value` |
| `custom` | Custom Python Function | despacha a un método `_report_custom_engine_*` |

**No** hay un séptimo engine ni renombrados en v19: los 6 son idénticos.

**`date_scope`** (Selection, required, default `strict_range`) — **AQUÍ SÍ hay diff v18→v19**
(`AccountReportExpression`, v18 líneas 559-571 / v19 líneas 580-633):
- v18: `from_beginning`, `from_fiscalyear`, `to_beginning_of_fiscalyear`,
  `to_beginning_of_period`, `strict_range`, **`previous_tax_period`** ("From previous tax period").
- v19: los mismos salvo el último → **`previous_return_period`** ("From previous return period"),
  ligado al nuevo modelo `account.return` de v19.

## 3. Engines de cálculo (EE) y el hook `custom`

El dispatcher es
`_compute_formula_batch(column_group_options, formula_engine, date_scope, formulas_dict, current_groupby, next_groupby, offset=0, limit=None, warnings=None)`
(`account_reports/models/account_report.py::_compute_formula_batch`, EE v18 líneas 3745-3780):

```python
engine_function_name = f'_compute_formula_batch_with_engine_{formula_engine}'
```

Métodos existentes: `_compute_formula_batch_with_engine_tax_tags`, `_domain`, `_account_codes`,
`_external`, `_custom`. El **aggregation** se procesa aparte del dispatch genérico (ver "no
verificado" al final). Todos con firma
`(self, options, date_scope, formulas_dict, current_groupby, next_groupby, offset=0, limit=None, warnings=None)`.

**Return shape** de un formula batch (docstring literal, EE v18 líneas 3770-3774):
- Con groupby: `{(formula, expressions): [(grouping_key, {'result': value, 'has_sublines': bool}), ...], ...}`
- Sin groupby: `{(formula, expressions): {'result': value, 'has_sublines': bool}, ...}`
- `'result'` es la clave por defecto; un engine puede devolver otras (`sum`, `sum_if_pos`, …).

### El engine `custom` (el hook de extensión principal)
`account_reports/models/account_report.py::_compute_formula_batch_with_engine_custom`
(EE v18 línea 4352 / v19 línea 4507):

```python
for formula, expressions in formulas_dict.items():
    custom_engine_function = self._get_custom_report_function(formula, 'custom_engine')
    rslt[(formula, expressions)] = custom_engine_function(
        expressions, options, date_scope, current_groupby, next_groupby,
        offset=offset, limit=limit, warnings=warnings)
```

El `formula` de una expresión con `engine='custom'` **ES el nombre del método**, que **debe empezar
por `_report_custom_engine_`**. `_get_custom_report_function(function_name, prefix)`
(`account_reports/models/account_report.py::_get_custom_report_function`, EE v18 líneas 2594-2612;
v19 línea 2713):
1. valida el prefijo → si no, `UserError`. **Ojo con el segundo argumento: NO es el prefijo
   completo.** El método lo construye — `function_name_prefix = f'_report_{prefix}_'`
   (`:2601`) — así que hay que pasarle **`custom_engine`**, no `_report_custom_engine_`.
   Quien pase el prefijo completo, que es como lo lee cualquiera, recibe un error que se
   delata solo: *«must start with the `_report__report_custom_engine__` prefix»*. Medido;
   sonda en `tools/bench-orm/addons/dyn_acc_ee`;
2. busca el método **primero en el custom handler model** (`self.custom_handler_model_id` →
   `self.env[custom_handler_model_name]`) y, si no, en el propio `account.report`;
3. si no existe → `UserError`.

**Firma canónica de un custom engine** (idéntica en todos los ejemplos reales):

```python
def _report_custom_engine_<name>(self, expressions, options, date_scope,
        current_groupby, next_groupby, offset=0, limit=None, warnings=None):
    ...
```

**Ejemplo mínimo, sin groupby** (`executive_summary_report.py::_report_custom_engine_executive_summary_ndays`,
EE v18 líneas 10-15) — lanza `UserError` si se pide groupby y devuelve un dict simple:

```python
def _report_custom_engine_executive_summary_ndays(self, expressions, options, date_scope,
        current_groupby, next_groupby, offset=0, limit=None, warnings=None):
    if current_groupby or next_groupby:
        raise UserError(_("Can't group by for this expression."))
    date_from = fields.Date.from_string(options['date']['date_from'])
    date_to = fields.Date.from_string(options['date']['date_to'])
    return {'result': (date_to - date_from).days + 1}   # clave 'result'
```

**Patrón completo con groupby** (`account_multicurrency_revaluation_report.py::_report_custom_engine_multi_currency_revaluation_to_adjust`,
EE v18 líneas 380-387):

```python
if not current_groupby:
    return build_result_dict(...)          # dict {'result'/..., 'has_sublines': bool}
else:
    rslt = []
    for query_res in query_res_lines:
        rslt.append((query_res['grouping_key'], build_result_dict(...)))
    return rslt                            # lista de (grouping_key, dict)
```

- `current_groupby` = **solo el nivel actual** a computar. Si la línea tiene
  `groupby='partner_id,account_id,id'`, en la primera pasada `current_groupby='partner_id'` y
  `next_groupby='account_id,id'`. El motor te vuelve a llamar por nivel al hacer unfold.
- Es habitual entrar con `report = self.env['account.report'].browse(options['report_id'])` y validar
  con `report._check_groupby_fields(...)`.
- Otros ejemplos reales verificados: `account_aged_partner_balance.py::_report_custom_engine_aged_receivable`
  / `_aged_payable` (EE v18 líneas 84-88); `bank_reconciliation_report.py`
  (`_misc_operations`, `_outstanding_receipts`, `_last_statement_balance_amount`, …, líneas 68-144).

### El custom handler model (lógica que no cabe en engines)
`account.report.custom.handler` (AbstractModel;
`account_reports/models/account_report.py::AccountReportCustomHandler`, EE v18 líneas 7680-7771;
sigue existiendo en v19, línea ≈8076). Se referencia desde el report vía `custom_handler_model_id`.
Ganchos a sobreescribir (todos con docstring verificado):
- `_dynamic_lines_generator(report, options, all_column_groups_expression_totals, warnings=None)`
  → lista `[(sequence, line_dict), ...]` — líneas 100% dinámicas (sin `line_ids`).
- `_custom_options_initializer(report, options, previous_options)` — añade filtros/opciones propias.
- `_custom_line_postprocessor(report, options, lines)` → `lines`.
- `_caret_options_initializer()`, `_custom_groupby_line_completer`,
  `_custom_unfold_all_batch_data_generator`,
  `_get_custom_display_config` (permite cambiar templates JS / componentes OWL),
  `_get_custom_groupby_map` (groupby custom no-AML), `_customize_warnings`.

## 4. Añadir un informe / línea / columna / expresión (data XML)

Se hace con data records; los atajos `*_formula` acortan mucho el XML. Ejemplo real
(`account_reports/data/balance_sheet.xml`, EE v18 líneas 1-70):

```xml
<record id="balance_sheet" model="account.report">
    <field name="name">Balance Sheet</field>
    <field name="filter_multi_company">selector</field>
    <field name="custom_handler_model_id" ref="model_account_balance_sheet_report_handler"/>
    <field name="column_ids">
        <record id="balance_sheet_balance" model="account.report.column">
            <field name="name">Balance</field>
            <field name="expression_label">balance</field>
        </record>
    </field>
    <field name="line_ids">
        <record id="account_financial_report_total_assets0" model="account.report.line">
            <field name="name">ASSETS</field>
            <field name="code">TA</field>
            <field name="aggregation_formula">CA.balance + FA.balance + PNCA.balance</field>
            <field name="children_ids">
                <record id="..." model="account.report.line">
                    <field name="name">Bank and Cash Accounts</field>
                    <field name="code">BA</field>
                    <field name="groupby">account_id</field>
                    <field name="foldable" eval="True"/>
                    <field name="domain_formula">sum([('account_id.account_type', '=', 'asset_cash')])</field>
                </record>
                ...
```

Notas verificadas:
- La cabecera `<odoo auto_sequence="1">` autoincrementa `sequence`.
- `column_ids` inline con `expression_label` que casa con la `label` de las expresiones de cada
  línea (aquí `'balance'`).
- Atajos por línea: `aggregation_formula` (referencia `code.label` de otras líneas),
  `domain_formula` (`sum([...])`), y análogos `account_codes_formula`, `tax_tags_formula`,
  `external_formula`. Cada uno crea automáticamente una `account.report.expression` con la label de
  la columna (`balance`) y el engine correspondiente (`_create_report_expression`).
- Para **expresión explícita** (varias labels, o engine custom) declara `expression_ids` con records
  `account.report.expression` fijando `label`, `engine`, `formula`, `subformula`, `date_scope`,
  `figure_type`.
- Para **engine custom**: `<field name="engine">custom</field>` +
  `<field name="formula">_report_custom_engine_xxx</field>`.

### Añadir un informe nuevo vs añadir una línea a uno existente
- **Informe nuevo**: record `account.report` completo como el de arriba + su
  `ir.actions.client` (ver §5).
- **Extender uno existente**: son data records, no vistas → no aplica `<xpath>`. Se **sobreescriben**
  campos del record existente por su xmlid, o se **añaden** records nuevos
  (`account.report.line` / `.column` / `.expression`) apuntando por `ref` a `report_id` /
  `parent_id`. Los informes *variant* usan `root_report_id`; los *compuestos* usan
  `section_report_ids`.
  > No verificado con cita: no abrí un archivo de fuente que **añada** una línea a un report definido
  > en otro módulo. El patrón descrito es el estándar de data-records de Odoo, no un recibo de fuente.

## 5. Acción cliente + componente OWL raíz (idéntico v18/v19)

- **Registro OWL**: `registry.category("actions").add("account_report", AccountReport)`
  (`account_reports/static/src/components/account_report/account_report.js::AccountReport`,
  v18 línea 130 / v19 línea 152). `class AccountReport extends Component`,
  `static template = "account_reports.AccountReport"`.
- **Componentes custom**: `registry.category("account_reports_custom_components")` — **diff interno**:
  v18 usa `customComponent.template` como key; **v19 usa `customComponent.name`** (línea ≈88/89).
- **Tag de la acción**: `ir.actions.client` con `<field name="tag">account_report</field>` y
  `context` con `{'report_id': ref('...')}` (`account_reports/data/account_report_actions.xml`,
  v18 líneas 1-18; v19 mismo tag). Ej.: Balance Sheet añade `<field name="path">balance-sheet</field>`.

Para extender el render sin tocar JS del core: `_get_custom_display_config` en el custom handler
apunta a tus propios templates / componentes registrados en `account_reports_custom_components`.

## 6. El `options` que recibe un custom engine (claves clave)

Lo construyen los `_init_options_*` de EE (`_init_options_date` / `_get_dates_period`,
EE v18 líneas 492-534 y 646-747). Claves verificadas por uso real en engines:
- `options['report_id']` — id del `account.report` (los engines hacen `.browse(options['report_id'])`).
- `options['date']` = dict de `_get_dates_period`: `date_from`, `date_to`, `string`, `period_type`,
  `mode` (+ `filter`, `period`). `mode` ∈ `{'range', 'single'}`.
- `options['comparison']` = `{'periods': [ {date_from, date_to, ...}, ... ], ...}` (+ `filter`; se
  hace `update` con `periods[0]`).
- `options['columns']` (columnas a pintar) y `options['column_groups']` (dict de grupos; el engine
  recibe ya el `column_group_options` partido por grupo).
- `options['journals']`, `options['unfold_all']` (bool), `options['unfolded_lines']` (ids),
  `options['currency_rates']` / `options['currency_table']` (multicurrency).

Para un custom engine típico basta leer `options['date']['date_from'/'date_to']`,
`options['report_id']` y respetar `current_groupby`/`next_groupby`/`offset`/`limit`.

> No verificado: la lista **completa** de claves de `options`. Confirmadas por uso real las de
> arriba; NO se leyeron enteros todos los `_init_options_*` (p.ej. `available_vat_fiscal_positions`,
> `buttons`, `variants`, `sections`, `analytic`, `all_entries` no verificadas).

## 7. Diffs v18 → v19 (resumen)

**Idénticos**: modelos, campo `engine` (6 valores), firma del custom engine, dispatch
`_compute_formula_batch_with_engine_*`, tag de acción `account_report`, componente OWL
`AccountReport` / template `account_reports.AccountReport`.

**Cambian**:
- `date_scope`: `previous_tax_period` (v18) → **`previous_return_period`** (v19), por el nuevo
  `account.return`.
- `_sql_constraints` (lista) v18 → **`models.Constraint`** v19 (p.ej.
  `_domain_engine_subformula_required`, `_line_label_uniq`) — coincide con la regla global v19.
- v19 añade modelos EE: `account.return`, `account.report.horizontal.group(.rule)`,
  `account_audit_account_status`, `kpi_provider`, y carpeta `static/src/views`.
  `account.report.custom.handler` sigue existiendo.
- Registro `account_reports_custom_components`: key `.template` (v18) → `.name` (v19).
- `chart_template.py` (v18) → `account_chart_template.py` (v19).

> No verificado: si v19 mantiene idénticos **todos** los `filter_*` de `account.report` (solo leí v18
> en detalle; de v19 CE verifiqué expression y line parcialmente).

## Lo que NO se pudo verificar (honesto)
- Cuerpo/return exacto de `_compute_formula_batch_with_engine_aggregation` (se procesa aparte del
  dispatch genérico, EE v18 ≈línea 3435): confirmada su existencia, no su implementación.
- Enumeración completa de `subformula` válidos por engine (`sum`, `sum_if_pos`, `-sum`,
  `cross_report`, `editable`, `rounding=N`, …): vistos parcialmente en `DOMAIN_REGEX` y
  `_create_report_expression`, no enumerados desde cada `_compute_formula_batch_with_engine_*`.
- Ejemplo de fuente que **extienda** un informe existente añadiéndole una línea desde otro módulo
  (§4): descrito por patrón, sin cita.
- Lista completa de claves de `options` (§6) y paridad total de `filter_*` en v19 (§7).
