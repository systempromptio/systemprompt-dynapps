# Account Reports — Integrations

Bridges del motor de reportes contables Enterprise (`account_reports`) con account (AML aggregation engines), analytic (multi-dimensión), tax_tags, multi-company consolidation, audit drill-down, variants/sections (declaraciones país), return types (303/111/347/etc.), multi-currency, export PDF/XLSX/CSV y external values manuales. Ubicación: `odoo_enterprise/18/account_reports/`.

Para gotchas del core ver `gotchas.md` (ARG1-37). Aquí solo bridges. Continúa: **ARG38+**.

Convención IDs: **ARG<N><suf>** (A=account/aml, AN=analytic, T=tax_tags, MC=multi_company, AU=audit, V=variants_sections, RT=return_type, FX=multi_currency, EX=export, EV=external_values).

Cross-link: detalles del motor 2 (Tax Reports Enterprise tag-based) en `ecosystem/l10n_es/` y memory project_aeat_*. Aquí cubrimos el engine genérico, no implementación AEAT.

---

## 1. account_reports ↔ account (move_line aggregation)

Núcleo del motor. Todas las expresiones (excepto `external` y algunas `custom`) terminan reduciendo SQL sobre `account_move_line` filtrado por opciones del informe.

### Modelos y campos

- `account.report`: contenedor (line_ids, column_ids, country_id, filter_*, custom_handler_model_id, only_tax_exigible).
- `account.report.line`: name, code (cross-ref), expression_ids, hierarchy_level, groupby (CSV `partner_id,account_id,id`), foldable, hide_if_zero.
- `account.report.expression`: label, engine (`domain`, `sum`/`sum_if_pos`/`sum_if_neg`, `tax_tags`, `account_codes`, `aggregation`, `external`, `custom`), formula, subformula, date_scope, auditable.
- `account.report.column`: name, expression_label, figure_type (monetary/percentage/integer/float/date/string), sortable, blank_if_zero.
- `account.move.line` (inherit Enterprise): añade `exclude_bank_lines` (computed sobre `journal_id.default_account_id`), constraint `_check_taxes_on_closing_entries` (no taxes en líneas con `move_id.tax_closing_report_id`), `_prepare_aml_shadowing_for_report()` para temp tables (budgets).

### Pipeline

```
JS controller → report.get_options(previous_options)
  → ~40 _init_options_* en secuencia (_get_options_initializers_forced_sequence_map)
  → posible reroute por variant/section (_init_options_report_id, sequence 17)
report._get_lines(options)
  → _compute_expression_totals_for_each_column_group()
    → _split_options_per_column_group() — cada period × horizontal group = un column group
    → _compute_formula_batch_with_engine_<engine>() por engine + date_scope
      → domain  → _get_report_query(options, date_scope) → SQL SUM(balance) GROUP BY groupby
      → tax_tags → JOIN account_account_tag_account_move_line_rel + SUM con tax_negate + tax_tag_invert
      → account_codes → parseo de prefijos D/C, exclusiones \(…), tag(xmlid)
      → aggregation → DAG topo sort, resuelve referencias entre líneas/reportes
      → external → SELECT FROM account_report_external_value (sin AML)
      → custom → invoca `_report_custom_engine_<name>` en custom_handler_model
  → _get_dynamic_lines() vía custom handler (si existe)
  → merge static + dynamic por sequence, hierarchy, prefix groups, totals-below-sections
  → _custom_line_postprocessor → _format_column_values
```

### Configuración

- Engine `domain` SIEMPRE filtra por `parent_state` (`'posted'` salvo `all_entries=True` → excluye `cancel`).
- Domain base aplicado en TODA query: `display_type NOT IN ('line_section','line_note')`, `company_id IN report_company_ids`, opciones de fecha/journal/partner/etc. + `_apply_ir_rules(query)` sobre AML (record rules multi-company respetadas).
- `only_tax_exigible=True` en report → añade `move_line._get_tax_exigible_domain()` (excluye `tax_exigibility='on_payment'` no pagado).
- `compute_budget` en options → swap de `account_move_line` por `account_report_budget_temp_aml` (temp table heredada con `ON COMMIT DROP`).

### Gotchas

**ARG38A.** `sum_if_pos`/`sum_if_neg` evalúan el signo del **total agregado** de TODOS los groupby keys, NO por key. Si el total es positivo, `sum_if_pos` devuelve todos los grouping_keys con sus valores tal cual, `sum_if_neg` devuelve 0 (y viceversa). Caso típico: 5 partners +1000€ y 5 partners -1100€ → total -100€ → `sum_if_pos`=0 para los 10 partners. Ref `account_reports/models/account_report.py:3938-3962`, `AccountReport._compute_formula_batch_with_engine_domain`.

**ARG39A.** Batching de domains: si todos los términos del domain filtran por UN solo M2O field de AML, varias expresiones se ejecutan en UNA query con `GROUP BY` extra sobre ese M2O. Si tu domain mezcla `partner_id` y `account_id` → NO se batchea (`non_batchable_domains_data`). `count_rows` o `offset/limit` también desactivan batching. Ref `account_reports/models/account_report.py:3840-3863`, `AccountReport._compute_formula_batch_with_engine_tax_tags`.

**ARG40A.** `count_rows` cuenta `COUNT(DISTINCT next_groupby_first_field)`. Si la línea NO tiene groupby → cuenta AMLs distintos por `id`. Si tiene groupby `partner_id,account_id` → cuenta partners distintos al primer unfold. Confuso: cambiar el groupby cambia el count. Ref `account_reports/models/account_report.py:3873`, `AccountReport._compute_formula_batch_with_engine_domain` y `account_reports/models/account_report.py:3804-3806`, `AccountReport._compute_formula_batch_with_engine_tax_tags`.

**ARG41A.** `display_type IN ('line_section','line_note')` EXCLUIDAS en todo query (`_get_options_domain`:2090). Si tienes líneas de sección con saldo (raro pero ocurre en imports legacy) → no aparecen en reports. Para forzar inclusión hay que override `_get_options_domain`.

**ARG42A.** Engine `domain` con term `('account_id.account_type', '=', 'asset_receivable')`: el batching detecta `aml_root_fields={'account_id'}`, comodel `account.account`, batchea pero requiere un `search(domain).ids` previo en `account.account` (línea 3915). Para 50k accounts → search lento, no se cachea entre column groups.

**ARG43A.** `parent_state='posted'` por defecto: borradores excluidos siempre salvo `all_entries=True` (toggle "Include Unposted Entries"). Reports comparativos con un periodo cerrado y otro abierto: el abierto puede mostrar 0 si el usuario no activó borrador. Confusión común con `account_followup_report` que SÍ los muestra. Ref `_get_options_all_entries_domain:914`.

**ARG44A.** Budget shadow table (`account_report_budget_temp_aml`): hereda DDL de `account_move_line` pero **NO** sus constraints (`NO INHERIT account_move_line` + DROP NOT NULL). Insert SOLO desde `account_report_budget_item`. Si tu custom handler genera SQL que joinea contra `account_move`/`account_journal` → falla porque `move_id`/`journal_id` son NULL. Ref `_create_report_budget_temp_table:2140-2217`.

**ARG45A.** `exclude_bank_lines` (campo computed stored) se calcula vía `account_id != journal_id.default_account_id`. Si un usuario crea AML manual en diario bancario apuntando a otra cuenta → `exclude_bank_lines=True` y la contrapartida tampoco se considera bank. Bank Reconciliation Report puede mostrar inconsistencias. Ref `account_reports/models/account_move_line.py:13-18`, `class AccountMoveLine`.

**ARG46A.** Constraint `_check_taxes_on_closing_entries`: bloquea añadir `tax_ids`/`tax_tag_ids` a AMLs de moves con `tax_closing_report_id`. Si haces import legacy de cierres antiguos con tags → import explota. Workaround: limpiar tags pre-import o `with_context(_force_skip=True)` no funciona (no hay bypass).

---

## 2. account_reports ↔ analytic (analytic_account dimension)

### Modelos y campos

- `account.report`: `filter_analytic` (Boolean, default False) — habilita menú filtro.
- `account.report` options: `analytic_accounts` (lista IDs), `display_analytic`, `selected_analytic_account_names`, `display_analytic_groupby`, `display_analytic_plan_groupby`.
- `account.move.line`: `analytic_distribution` (JSON `{plan_account_id: percent}`).
- `account.analytic.account`: filtro principal.
- `account.analytic.plan` y `account.analytic.line` no se cruzan directamente con AML (sólo distribution).

### Pipeline

```
options['analytic_accounts'] = previous_options.get('analytic_accounts', [])
  → _init_options_analytic [account_report.py:862-874]
  → require group `analytic.group_analytic_accounting`
  → NO inyecta domain en _get_options_domain automáticamente

drill-down audit:
  → _get_audit_line_domain añade [('analytic_distribution', 'in', analytic_accounts)] [4634-4639]

custom handlers (Analytic Report):
  → leen options['analytic_accounts'] y construyen su propio query sobre account.analytic.line
```

### Configuración

- `filter_analytic=True` activa menú.
- Requiere grupo `analytic.group_analytic_accounting`.
- Para reports estándar (Balance Sheet, P&L): el filtro NO altera AML query, sólo el drill-down. Lo que filtra de verdad es `account_analytic_report` que usa `account.analytic.line`.

### Gotchas

**ARG47AN.** Filtro `analytic_accounts` NO afecta el cómputo del Balance Sheet ni P&L estándar — sólo el drill-down. Esto es porque `analytic_distribution` es un JSON con porcentajes y no se puede sumar como AML balance. Para usar de verdad analytic en reports financieros necesitas activar "analytic" en el report custom handler. Ref `account_reports/models/account_report.py:862-874`, `AccountReport._get_date_bounds_info` (no aplica filtro a `_get_options_domain`).

**ARG48AN.** El operator `'in'` sobre `analytic_distribution` (JSON) en domain — funciona porque Odoo lo trata como JSON key match, pero PostgreSQL usa `->>` con cast. Performance pobre con muchos analytic_accounts (no index). Ref `account_reports/models/account_report.py:4638`, `AccountReport.action_audit_cell`.

**ARG49AN.** Si activas `filter_analytic=True` pero el usuario NO tiene `analytic.group_analytic_accounting` → options['analytic_accounts'] queda en `[]` sin error pero el menú no aparece. Confusión típica: admin ve el filtro, accountant junior no. Ref `account_reports/models/account_report.py:867`, `AccountReport._get_date_bounds_info`.

**ARG50AN.** Analytic distribution con porcentajes (`{plan_acc_id: 60.0, plan_acc_id_2: 40.0}`) NO se aplica como factor al balance en drill-down. El AML completo aparece, no su 60%. Para ver el 60% real hay que ir al Analytic Report (`account.analytic.line`). Confusión muy común.

**ARG51AN.** Multi-plan analytic (v17+): un AML puede tener múltiples planes simultáneos. El filtro `analytic_accounts` agrupa SIN distinguir plan — si filtras por "Marketing" (plan A) y "Project X" (plan B), AMLs con AMBOS aparecen una sola vez, no doble. `_get_audit_line_domain` no de-duplica explícitamente.

**ARG52AN.** `account_analytic_report` (handler) opera sobre `account.analytic.line` (no AML). Esto significa que entries SIN AML (analytic puro, e.g. timesheet) aparecen en el Analytic Report pero NUNCA en Balance Sheet aunque tengan importe. Disonancia esperada por design pero rompe expectativa de "todo se ve en BS".

---

## 3. account_reports ↔ tax_tags

### Modelos y campos

- `account.account.tag` (applicability=`'taxes'`): tags `+xxx` y `-xxx` creadas automáticamente al crear expresiones engine=`tax_tags`. Campos clave: `tax_negate` (Bool), `country_id` (M2O), `applicability='taxes'`.
- `account.report.expression`: engine=`'tax_tags'`, formula=tag name SIN signo (e.g. `casilla_01`).
- `account.move.line`: `tax_tag_ids` (M2M tag), `tax_tag_invert` (Bool, computed por `_compute_tax_tag_invert`).
- `account.tax.repartition.line`: `tag_ids` (M2M) que se propagan a AMLs al crear factura.

### Pipeline

```
crear expresión engine='tax_tags', formula='casilla_01', report.country_id=ES
  → _get_tags_create_vals genera +casilla_01 (tax_negate=False) y -casilla_01 (tax_negate=True)
    con country_id=ES, applicability='taxes'
  → ambas tags creadas/recuperadas

invoice posted con tax con repartition_line.tag_ids = [+casilla_01]
  → AML hereda tax_tag_ids = [+casilla_01]
  → tax_tag_invert se computa según tipo move (refund invierte)

report computation (_compute_formula_batch_with_engine_tax_tags:3714):
  → all_expressions._get_matching_tags() resuelve tags por (formula, country_id)
  → SQL JOIN account_account_tag_account_move_line_rel:
    SUM(balance * (tag.tax_negate ? -1 : 1) * (aml.tax_tag_invert ? -1 : 1))
  → SUBSTRING(tag.name, 2) → recupera formula sin el ±
```

### Configuración

- `report.country_id` debe coincidir con `tag.country_id`. Sin country_id → tags globales (problemático en multi-país).
- Tags creadas con prefix `+` y `-` automáticamente al guardar el report (`_create_tags()` en `account.report.expression`).
- `tax_tag_invert` recalculado al posted/cancel de moves.

### Gotchas

**ARG53T.** Engine `tax_tags` NO soporta subformula. Si pones `subformula='sum_if_pos'` → se ignora silenciosamente, no error. Para condicionales en tax tags hay que usar `aggregation` por encima con `if_above`. Ref `account_reports/models/account_report.py:3722`, `AccountReport._aggregation_apply_bounds` ("This engine does not support any subformula").

**ARG54T.** Cambiar el `country_id` de un report con expresiones tax_tags existentes → tags antiguas (con country viejo) quedan huérfanas; nuevas tags se crean con country nuevo. AMLs históricos siguen apuntando a tags viejas → report nuevo da 0. Ref `_get_matching_tags:808-820` filtra por country.

**ARG55T.** Una tag puede vivir en `account.account` (applicability='accounts') Y en taxes; pero el engine `tax_tags` SOLO mira applicability='taxes' implícitamente vía la creación (`_get_tags_create_vals`). Si alguien crea manualmente una tag '+casilla_01' con applicability='accounts' → invisible al engine.

**ARG56T.** `tax_tag_invert` se computa pero NO siempre se recalcula al cambiar tax_repartition. Reformar un move (cancel + duplicate + post) puede dejar `tax_tag_invert` desincronizado. Bug histórico en CE; vigilar tras reformas masivas (migrations).

**ARG57T.** SQL del engine usa `SUBSTRING(name, 2)` para quitar el ±. Si alguien crea tag con name `++casilla_01` (doble prefijo, error humano) → SUBSTRING devuelve `+casilla_01` → emparejaría falsamente con otra tag. Validador no protege contra esto. Ref `account_reports/models/account_report.py:3738`, `AccountReport._aggregation_apply_bounds`.

**ARG58T.** Tax tags NO respetan `parent_state='posted'` por sí solas — heredan del domain general de `_get_options_domain`. Por tanto: incluir borradores en report tax → tags se suman desde AMLs borrador también. AEAT puede pedir solo `posted` → toggle siempre off para legal. Cross-ref ARG43A.

**ARG59T.** Multi-VAT (fiscal_position): el filtro tax_tag se aplica con `('tax_tag_ids.country_id', '=', report.country_id)` + exclusión de fiscal_positions de otras regiones del mismo país. Casos India con múltiples states → AMLs con tag de un state pueden aparecer en report de otro si no se excluyó bien `fp_ids_to_exclude`. Ref `_get_options_fiscal_position_domain:1250-1289`.

---

## 4. account_reports ↔ multi-company (consolidation)

### Modelos y campos

- `account.report`: `filter_multi_company` selection (`disabled`/`selector`/`tax_units`).
- `account.tax.unit`: agrupación de companies para reporting consolidado por VAT. Campos: `company_ids`, `country_id`, `main_company_id`.
- options: `companies` (lista dicts con id/name/currency_id), `tax_unit` (id o `'company_only'`), `forced_companies`, `available_tax_units`.
- `res.company`: `_accessible_branches()` (multi-branch roll-up), `account_fiscal_country_id`, `_get_branches_with_same_vat()`.

### Pipeline

```
_init_options_companies [1295-1307]:
  filter_multi_company='disabled' → env.company._accessible_branches() (solo branches)
  filter_multi_company='selector' → env.companies (selector global de Odoo)
  filter_multi_company='tax_units' → _multi_company_tax_units_init_options (mismo país, mismo VAT)
  previous_options['forced_companies'] → bypass

_multi_company_tax_units_init_options [1309-1361]:
  tax_units_domain = [('company_ids', 'in', env.company.id), ('country_id', '=', report.country_id.id)]
  filtra units con TODAS las companies accesibles al user (sudo() para chequear)
  default: si una sola tax_unit y su companies == env.companies → seleccionarla
  else: 'company_only' (branches del current company)

todos los queries: WHERE company_id IN report.get_report_company_ids(options)
  + _apply_ir_rules(query) sobre account.move.line (record rules multi-company)
```

### Configuración

- `filter_multi_company='tax_units'` requiere country_id en report.
- `account.tax.unit` se crea manualmente en Configuration → Fiscal Localization.
- Branches: company padre con `child_ids` activos.

### Gotchas

**ARG60MC.** `filter_multi_company='disabled'` no significa "ignora multi-company" — significa "solo branches del current company". Si tienes 3 companies independientes activas en el switcher pero report disabled → solo se considera env.company y sus branches. Sorpresa común tras activar nuevo company. Ref `account_reports/models/account_report.py:1304-1305`, `AccountReport.get_foreign_vat_tax_tag_extra_domain`.

**ARG61MC.** `tax_units` filtra companies "filtradas". Si user no tiene acceso a TODAS las companies de la unit (`sudo().company_ids` no subset de `env.user.company_ids`) → la tax_unit no aparece en `available_tax_units`. Admin sí la ve, accountant junior no. Diagnóstico: chequear `res.users.company_ids`. Ref `account_reports/models/account_report.py:1320-1323`, `AccountReport.get_foreign_vat_tax_tag_extra_domain`.

**ARG62MC.** Currency conversion en multi-company opera vía `currency_table` (ver bridge 8). Si companies tienen `chart_template` distintas → `account_codes` engine con `'123'` puede no match accounts en una company (numeración local distinta). Balance Sheet consolidado da resultados parciales sin warning explícito.

**ARG63MC.** Branches: `_accessible_branches()` retorna company + descendientes activos. Si tienes branch archived con AMLs históricos → NO aparece en report current pero AMLs siguen ahí. Drill-down audit muestra esos AMLs filtrados por record rule → puede dar 0 líneas. Ref `account_reports/models/account_report.py:1305`, `AccountReport.get_foreign_vat_tax_tag_extra_domain`.

**ARG64MC.** Cambiar `tax_unit` en UI dispara hack JS que recarga companies (`A js hack ensures the page is reloaded`) — durante esa recarga options['companies'] cambia y reports cacheados se invalidan parcialmente. Bugs visuales reportados con widgets que dependen del previous state. Ref `account_reports/models/account_report.py:1331-1336`, `AccountReport._get_options_fiscal_position_domain`.

**ARG65MC.** Export de tax report en multi-company con varias VAT (branches con `vat` distinto): export buttons deshabilitados salvo `branch_allowed=True` (caso PDF/XLSX). XML/BoE generalmente bloqueado para evitar mezclar VATs. `_enable_export_buttons_for_common_vat_groups_in_branches` permite override pero requiere mismo VAT. Ref `account_reports/models/account_report.py:7649`, `class AccountReportHorizontalGroupRule`.

**ARG66MC.** Multi-company + journals filter: opciones `available_journals` se calculan POR company. Si un journal está solo en company A y user filtra dos companies → journal aparece, pero AMLs solo de A. Drill-down audit puede mostrar columna company vacía para esa journal en B.

---

## 5. account_reports ↔ audit (drill-down)

### Modelos y campos

- `account.report.expression`: `auditable` (Boolean, default True para `domain`/`tax_tags`/`account_codes`; False para `aggregation` salvo expansión).
- `account.report.column`: `custom_audit_action_id` (M2O `ir.actions.act_window`) — override del audit action por columna.
- caret options: menú contextual sobre líneas de detalle.

### Pipeline

```
click "Audit" en celda → action_audit_cell(options, params) [4533]
  params = {report_line_id, expression_label, column_group_key, calling_line_dict_id}
  
  if expression.engine == 'external':
    return act_window account.report.external.value
      domain = [('target_report_expression_id','=', expression.id), ('date','<=', date_to), ...]
      if formula=='most_recent': sólo registros del max(date)
  
  else (engines AML):
    action = column.custom_audit_action_id or default act_window account.move.line
    domain = _get_audit_line_domain(column_group_options, expression, params)
      → _expand_aggregations() — resuelve aggregation hasta engines base
      → _get_expression_audit_aml_domain por cada expresión base (domain/tax_tags/account_codes)
      → date_scope: respeta date_scope de cada subexpresión EXCEPTO si subformula='cross_report'
      → AND con _get_options_domain(column_group_options, date_scope)
      → AND con _get_audit_line_groupby_domain(calling_line_dict_id) — extrae groupby keys
      → + analytic filter si options['analytic_accounts']
```

### Gotchas

**ARG67AU.** Engine `account_codes` con sufijo `D`/`C` NO puede traducirse a domain AML (depende del signo del TOTAL de la cuenta, no per-AML). El audit muestra **TODAS** las líneas del prefix sin filtro D/C, no las que contribuyeron. Comentario explícito en `_get_expression_audit_aml_domain:4660-4663`. Para mitigar: aplicar group_by en list view manualmente.

**ARG68AU.** `aggregation` con `cross_report` usa `expression.date_scope` (la principal) y NO el de las subexpresiones. Si tu aggregation suma `R1.balance + R2.balance` con date_scope distintos → audit usa SOLO el date_scope de la aggregation, perdiendo precisión de las hijas. Ref `account_reports/models/account_report.py:4611`, `AccountReport.action_audit_cell`.

**ARG69AU.** `_get_audit_line_groupby_domain` parsea el `line_dict_id` para extraer groupby keys (`partner_id`, `account_id`, `id`). Si tu custom handler usa custom groupby field (registered en `_get_custom_groupby_map`) → debe proveer `domain_builder` o el audit NO aplicará ese groupby y mostrará líneas de otros groupings. Ref `account_reports/models/account_report.py:4650-4651`, `AccountReport.action_audit_cell`.

**ARG70AU.** Custom audit action (`column.custom_audit_action_id`) reemplaza el action default pero NO elimina el domain calculado — el domain se asigna a `action['domain']` SIEMPRE, sobreescribiendo el del action custom. Si tu action custom tenía un domain pre-establecido → se pierde. Ref `account_reports/models/account_report.py:4582-4583`, `AccountReport.execute_action`.

**ARG71AU.** Audit de columna con `figure_type='string'` (text external value) abre list de `account.report.external.value` (no AML). Si el usuario espera ver moves → confusión. No hay tooltip que avise. Ref `account_reports/models/account_report.py:4540-4566`, `AccountReport._create_carryover_for_company`.

**ARG72AU.** El domain del audit incluye `_get_options_domain(column_group_options, date_scope)` que aplica `parent_state='posted'` POR DEFECTO. Si user activó `all_entries=True` en report → audit muestra borradores; si lo desactivó luego pero la URL es vieja → audit muestra sólo posted, no concuerda con visión del report. Cross-ref ARG43A.

---

## 6. account_reports ↔ variants/sections

### Modelos y campos

- `account.report`: `root_report_id` (M2O self), `variant_report_ids` (O2M self inverse `root_report_id`), `country_id`, `use_sections` (Bool computed).
- `account.report`: `section_report_ids` (M2M, sub-reports composición), `section_main_report_ids` (M2M inverse).
- options: `variants_source_id`, `available_variants`, `selected_variant_id`, `sections_source_id`, `sections`, `selected_section_id`, `has_inactive_variants`, `has_inactive_sections`.

### Pipeline

```
_init_options_variants [1803-1848]:
  source = previous_options['sections_source_id'].root_report_id OR root_report_id OR self
  all_variants = source + source.with_context(active_test=False).variant_report_ids
  for each variant: variant._is_available_for(options) — filtra por country/company
  default selected: variant matching env.company.account_fiscal_country_id

_init_options_sections [1860-1881]:
  source = browse(selected_variant_id)
  if source.use_sections: sections = source.section_report_ids
  default: primer section

_init_options_report_id [1886-1891]:
  options['report_id'] = selected_section_id OR selected_variant_id OR self.id
  (← REROUTE: subsequent _init_options_* corren con report=options['report_id'])
```

### Configuración

- Root report (genérico): `country_id=False`, `root_report_id=False`, varias variants apuntan a él.
- Variant local: `root_report_id=<root>`, `country_id=<ES/FR/...>`.
- Section composite: `use_sections=True`, `section_report_ids` con sub-reports.
- Ejemplo: l10n_es `mod_303`, `mod_390`, `mod_347` como variants del `account.generic_tax_report`.

### Gotchas

**ARG73V.** El reroute via `_init_options_report_id` (sequence=17) ocurre TARDE en la secuencia (después de companies, variants, sections). Los `_init_options_*` que corren ANTES (companies, variants, sections) usan SELF, no el report rerouteado. Si companies depende de variant.country_id (típico en tax_units) → bug porque country_id se evalúa del root, no del variant. Workaround: usar `previous_options['selected_variant_id']` explícitamente. Ref `account_reports/models/account_report.py:2113`, `AccountReport._get_options_initializers_forced_sequence_map` (el `sequence=17`); el reroute en sí, `account_reports/models/account_report.py:1940-1945`, `AccountReport._init_options_report_id`.

**ARG74V.** `_is_available_for(options)` filtra variants por country (default) — si tu variant heredada quiere mostrarse SOLO bajo cierto fiscal_position, hay que override del método. Caso India (varios estados con mismo report): difícil de filtrar. Ref `account_reports/models/account_report.py:5418`, `AccountReport.get_report_information` (TODO el filtro).

**ARG75V.** Sections con sub-reports de countries distintos: el PDF/XLSX export consolida pero el filtro `fiscal_position` se aplica al MAIN report, no a cada sub-section. Si secciones tienen distinto `country_id` → fiscal_position puede ser inválido para algunas. Sin warning.

**ARG76V.** `with_context(active_test=False)` SE USA al cargar variants para mostrar incluso archived. Pero `_is_available_for` filtra los inactive del default selection (`if variant.active`). `has_inactive_variants` flag → JS muestra botón "View All Variants" para incluirlos. Confusión: archived NO desaparece del menú sino aparece bajo "View All". Ref `account_reports/models/account_report.py:1820`, `AccountReport.open_account_report_file_download_error_wizard` y `account_reports/models/account_report.py:1826-1828`, `AccountReport.open_account_report_file_download_error_wizard`.

**ARG77V.** Sections heredan `selected_section_id` SOLO si está en `available_sections.ids`. Si cambias de variant y la section seleccionada no existe en la nueva → falla silenciosamente a la primera. Sorpresa común al navegar entre tax reports país. Ref `account_reports/models/account_report.py:1872-1874`, `AccountReport._init_options_variants`.

**ARG78V.** `section_report_ids` es M2M pero el orden depende del `sequence` de cada sub-report (no de la M2M order). Si dos sub-reports tienen mismo sequence → orden indeterminado. PDF rendering los apila en ese orden no-determinístico. Ref `_init_options_sections:1868-1869`.

---

## 7. account_reports ↔ return_types (declarations)

**v18**: declaraciones generadas vía `account.generic.tax.report.handler` + extensiones por país (l10n_es, l10n_fr...). **v19**: nuevo modelo `account.return` formaliza el ciclo.

### Modelos y campos

**v18:**
- `account.move`: `tax_closing_report_id` (M2O `account.report`), `tax_closing_end_date` — flag de move como cierre fiscal.
- `account.tax.repartition.line`: `use_in_tax_closing` (Bool) — si la línea cuenta para el cierre.
- `res.company`: `_get_tax_closing_period_boundaries(date_to, report)`, `_get_and_update_tax_closing_moves`, `tax_periodicity`, `account_tax_periodicity_journal_id`, `account_tax_next_activity_type`.

**v19 (nuevo):**
- `account.return.type`: name, category (`account_return`/`audit`), report_id (M2O), states_workflow (`generic_state_review`/`_submit`/`_tax_report`/`only_pay`), deadline_periodicity, deadline_start_date, deadline_days_delay, payment_partner_bank_id.
- `account.return`: una instancia por periodo+company. type_id, period_start, period_end, state, closing_move_id.
- `account.return.check`: validaciones del return (move_id, status, type).

### Pipeline (v18)

```
action_periodic_vat_entries(options) [account_generic_tax_report.py:90]
  → multi-company warning si len(companies) > 1
  → _get_periodic_vat_entries → _get_tax_closing_entries_for_closed_period (recupera moves draft existentes)
  → _generate_tax_closing_entries para companies sin move:
    → company._get_and_update_tax_closing_moves(end_date, report, fiscal_positions, include_domestic)
    → crea draft account.move con tax_closing_report_id=report.id
  → para cada closing_move:
    → _compute_vat_closing_entry(company, options) → SELECT por tax_line_id + tax_group_id
    → SUMa balance por (tax_id, account_id) WHERE use_in_tax_closing=True
    → _postprocess_vat_closing_entry_results (hook redondeo/ajuste)
    → _vat_closing_entry_results_rounding aplica diff vs report total
    → _add_tax_group_closing_items (líneas de tax_receivable/tax_payable accounts del tax_group)
  → write line_ids en move

post manual del closing move → AML con tax_closing_report_id != False
  → constraint _check_taxes_on_closing_entries IMPIDE añadir tax_ids/tax_tag_ids
```

### Gotchas

**ARG79RT.** Cierre VAT requiere `tax_group.tax_receivable_account_id` Y `tax_group.tax_payable_account_id` configurados. Si faltan en algún tax_group → ese grupo se IGNORA (continue), sin warning explícito. Resultado: closing entry incompleto, balance no cero. Ref `account_reports/models/account_generic_tax_report.py:339-340`, `AccountTaxReportHandler._compute_vat_closing_entry`.

**ARG80RT.** `_get_vat_closing_entry_additional_domain()` es hook para override por l10n_*. Sin override devuelve []. Localizations que filtran (e.g. excluir tax_groups específicos) deben override; si sub-localization re-extiende sin super() → pierde filtros. Patrón frágil. Ref `account_reports/models/account_generic_tax_report.py:394-395`, `class AccountTaxReportHandler`.

**ARG81RT.** Reabrir un closing move ya posted (cancel + edit + repost): SQL `_compute_vat_closing_entry` se re-ejecuta con `all_entries=False` y `parent_state='posted'`. **Pero** el closing move propio NO se excluye → sus AMLs (con `tax_closing_report_id`) entran al WHERE (no tienen tax_line_id por la constraint, pero balance afecta tag-based reports cuando se reincluyen tras cancel). Re-cálculo puede dar números distintos. Mitigación: forzar `('tax_closing_report_id','=',False)` en domain — no está aplicado por defecto.

**ARG82RT.** Multi-company tax_closing: si `len(env.companies) > 1` y no se pasa `override_tax_closing_warning=True` → warning bloqueante con UserError. Wizard de confirmación necesario. Devs que automatizan via scripts olvidan el context → falla. Ref `account_reports/models/account_generic_tax_report.py:94-104`, `AccountTaxReportHandler.action_periodic_vat_entries`.

**ARG83RT.** Periodo del cierre = `company._get_tax_closing_period_boundaries(date_to, report)`. Si el user filtra fechas custom NO alineadas con tax_periodicity (e.g. del 1 al 20 con periodicidad mensual) → cierre se genera para el MES COMPLETO, no para el rango. Wizard avisa pero usuarios saltan el warning. Ref `account_reports/models/account_generic_tax_report.py:101-102`, `AccountTaxReportHandler.action_periodic_vat_entries`, `:279-281`.

**ARG84RT.** `repartition_line.use_in_tax_closing=False` por defecto en algunas taxes (e.g. reverse charge en líneas no-imponibles). Si l10n del país no lo configura → tax aparece en report Modelo 303 pero NO entra en closing entry → balance descuadrado entre report y move. Diagnosis difícil. Ref `account_reports/models/account_generic_tax_report.py:309`, `AccountTaxReportHandler._compute_vat_closing_entry`  (filtra por use_in_tax_closing)`.

**ARG85RT.** v18 → v19 migration: `account.return` nuevo en v19 reemplaza la lógica embebida de v18. Modules custom que hooks `_generate_tax_closing_entries` en v18 deben migrar a `account.return._action_post_for_period()` en v19. Cambio de API significativo. Ref `odoo_enterprise/19/account_reports/models/account_return.py::AccountReturn+`.

**ARG86RT.** Carryover en tax reports (Modelo 303 casilla 110 → 87 del siguiente periodo): se gestiona via `account.report.external.value` con `carryover_origin_expression_label` + `carryover_origin_report_line_id`. Si borras un closing move post-carryover sin limpiar el external.value asociado → próximo periodo recibe carryover duplicado o desincronizado. Ref `_generate_carryover_external_values:4308`.

---

## 8. account_reports ↔ multi-currency

### Modelos y campos

- `account.report`: `currency_translation` selection (`'current'` rate at date_to / `'cta'` Cumulative Translation Adjustment), default 'cta'.
- options: `multi_currency` (Bool), `currency_table` ({type: monocurrency/current/cta, periods: {key: {from, to}}}).
- `res.currency`: `_check_currency_table_monocurrency(companies)`, `_get_monocurrency_currency_table_sql`, `_create_currency_table`.
- Multicurrency Revaluation Report (handler `account.multicurrency.revaluation.report.handler`): genera ajustes FX, modelo `account.multicurrency.revaluation.wizard`.

### Pipeline

```
_init_options_multi_currency [1366]: True si hay >1 currency entre companies O cols/expressions no-monetary
_init_options_currency_table [1376]:
  type = 'monocurrency' if companies single-currency else report.currency_translation
  periods extraídos de column_groups (cada col_group puede tener forced_options con date diferente)
  options['currency_table'] = {type, periods}

_init_currency_table [1471]: crea tabla temporal account_currency_table si type != monocurrency
  → res.currency._create_currency_table(companies, periods, use_cta_rates)

en cada query SQL:
  _currency_table_apply_rate(value) = value * COALESCE(account_currency_table.rate, 1)
  _currency_table_aml_join(options, aml_alias):
    if type=='cta':
      JOIN account_account aml_ct_account ON aml_ct_account.id = aml.account_id
      LEFT JOIN currency_table ON
        rate_type = CASE
          account_type LIKE 'equity%' THEN 'historical'
          account_type LIKE 'income%'/'expense%'/'equity_unaffected' THEN 'average'
          ELSE 'current'
        END
        AND date_from <= aml.date AND (date_next IS NULL OR date_next > aml.date)
        AND period_key = options['date']['currency_table_period_key']
    else:
      JOIN currency_table ON period_key
```

### Configuración

- `currency_translation='cta'`: usa CTA rates (equity historical, P&L average, BS current).
- `currency_translation='current'`: usa rate más reciente al date_to.
- Multicurrency Revaluation Report disponible si ≥2 currencies activas.

### Gotchas

**ARG87FX.** `currency_translation='cta'` usa LEFT JOIN — si no hay rate en `account_currency_table` para la fecha del AML, `COALESCE(rate, 1)` aplica rate=1 (sin conversión). Resultado: BS consolidado con AMLs sin rate calculado da números mezclados (algunos convertidos, otros no). Debería ser INNER JOIN o warning. Ref `account_reports/models/account_report.py:1429-1448`, `class AccountReport`.

**ARG88FX.** Rate type por account_type: la rule `account_type LIKE 'equity%'` matchea TODOS los equity (equity, equity_unaffected). Pero `equity_unaffected` está EXPLÍCITAMENTE en la rama `'average'` arriba (`LIKE ANY (ARRAY[..., 'equity_unaffected'])`). El CASE se evalúa ANTES de los hijos → equity_unaffected entra en `'historical'` no en `'average'` por orden. **Bug sutil** en BS consolidado. Verificar Ref `account_reports/models/account_report.py:1432-1435`, `AccountReport._init_options_currency_table`.

**ARG89FX.** Multicurrency Revaluation Report requiere `len(active_currencies) >= 2` (incluyendo la de la company). Solo USD activado en BD USD-company → UserError "You need to activate more than one currency". Confunde a usuarios que solo tienen una currency real. Ref `account_reports/models/account_multicurrency_revaluation_report.py:38-40`, `MulticurrencyRevaluationReportCustomHandler._custom_options_initializer`.

**ARG90FX.** El user puede override rates en options (`options['currency_rates']` mutable). Si modifica rate en UI → `custom_rate=True` → warning permanente. PERO el ajuste real (`action_multi_currency_revaluation_open_revaluation_wizard`) usa estos custom rates SIN volver a chequear si son sensatos. Posible distorsión enorme si user mete rate=0.0001 por error. Ref `account_reports/models/account_multicurrency_revaluation_report.py:60`, `MulticurrencyRevaluationReportCustomHandler._custom_options_initializer`.

**ARG91FX.** Comparison period (e.g. Year-over-Year): cada col_group tiene su propio period en `currency_table`. Si el período comparativo era pre-CTA-activation y el current es post → rates entre periods incongruentes. Reports consolidados Y-o-Y pueden mostrar growth% distorsionado. Ref `account_reports/models/account_report.py:1380-1408`, `AccountReport._multi_company_tax_units_init_options`.

**ARG92FX.** `no_impact_on_currency_table=True` en col_group (típico para Initial Balance del Trial Balance): excluye ese group del cálculo de periods → la initial balance NO afecta los rates aplicados a otras columnas. Pero el initial balance SE convierte usando rates de otra columna por proxy → off-by-one en períodos al borde. Diagnóstico complejo. Ref `account_reports/models/account_report.py:1382-1386`, `AccountReport._multi_company_tax_units_init_options`.

---

## 9. account_reports ↔ pdf/xlsx/csv export

### Modelos y campos

- options: `export_mode` (`'file'`/`'print'`/None), `force_landscape_printing` (context).
- buttons: array dicts con `{action: 'export_file', action_param: 'export_to_pdf'/'export_to_xlsx'/'export_to_xml'/'export_to_csv', file_export_type, branch_allowed, always_show}`.
- ir.actions: `ir_actions_account_report_download` (handler client-side download).
- `account.report.send` (wizard email + print).

### Pipeline

```
click PDF/XLSX → export_file(options, file_generator) [5889]
  returns {'type': 'ir_actions_account_report_download', 'data': {options json, file_generator}}
  → JS frontend POSTea a /web/dataset/call_kw/account.report/<file_generator>

export_to_pdf(options) [5908]:
  → recompute options con export_mode='print'
  → sections: cada sub-report rendered separadamente
  → custom_handler._get_pdf_export_html() si existe, else default
  → wkhtmltopdf con landscape si len(columns)>5 or horizontal_split
  → _merge_pdfs si múltiples sub-reports

export_to_xlsx(options) [6053]:
  → xlsxwriter workbook
  → _inject_report_into_xlsx_sheet (custom handlers override)
  → split account.code|account.name en cols separadas
  → level-based formatting (bold/indent)
  → "Options" sheet con filters aplicados (_add_options_xlsx_sheet)

export_to_csv → similar, csv writer

export_to_xml (tax reports): delegado a custom_handler con función con prefix `_l10n_<country>_export_to_xml` etc.
```

### Gotchas

**ARG93EX.** `export_mode='print'` fuerza `unfold_all=True` para PDF/XLSX. Reports con miles de líneas (e.g. Partner Ledger con 5k partners y AMLs cada uno) → render lento o timeout HTTP. No hay paginación PDF nativa. Mitigation: aplicar filtros pre-export. Ref `account_reports/models/account_report.py:5918-5926`, `class AccountReport`.

**ARG94EX.** PDF rendering usa wkhtmltopdf con `landscape=True` si `len(columns)>5 or horizontal_split`. Pero detecta `force_landscape_printing` en context (no en options). Tests automatizados que pasan options no triggean landscape. Ref `account_reports/models/account_report.py:5957`, `AccountReport._format_value`.

**ARG95EX.** `_get_pdf_export_html` rinde QWeb template `account_reports.pdf_export_main`. Custom handlers que override deben respetar el wrapper layout — si retornan HTML sin `<html>` válido → wkhtmltopdf da output truncado sin error claro. Ref `account_reports/models/account_report.py:5982`, `AccountReport._is_value_zero`.

**ARG96EX.** XLSX export inyecta una "Options" sheet con filtros aplicados (`_add_options_xlsx_sheet:6426`). Si tu custom handler añade filters propios pero NO los lista en `_inject_report_options_into_xlsx_sheet` → no aparecen en la Options sheet → auditoría incompleta. Ref `account_reports/models/account_report.py:6472`, `AccountReport._inject_report_into_xlsx_sheet`.

**ARG97EX.** Multi-section export combina PDFs vía `_merge_pdfs` (PyPDF). Si una sub-section da error en wkhtmltopdf → silenciado, el PDF resultante salta esa section sin warning. Diagnóstico requiere `loglevel=debug`. Ref `account_reports/models/account_report.py:5966-5973`, `AccountReport._format_value`.

**ARG98EX.** Branches+VAT: export buttons mostradas según `branch_allowed`. PDF/XLSX siempre `branch_allowed=True`. XML BoE/AEAT requiere mismo VAT en todas las branches o `_enable_export_buttons_for_common_vat_groups_in_branches`. Mezcla de VATs → botón "Export XML" hidden silenciosamente. Ref `account_reports/models/account_report.py:7649-7653`, `class AccountReportHorizontalGroupRule`.

**ARG99EX.** `account.report.send` wizard (email + print) usa `_get_report_send_recipients` del custom handler. Si el handler no define → devuelve `self.env['res.partner']` (vacío). Wizard se abre sin destinatarios → user piensa que es bug. Default debería ser `partner_id` del filter. Ref `account_reports/models/account_report.py:5902-5906`, `AccountReport._get_prefix_groups_matched_prefix_from_line_id`.

---

## 10. account_reports ↔ external_values

### Modelos y campos

- `account.report.external.value`: name, value (Float), text_value (Char), date (required), target_report_expression_id (M2O), company_id, foreign_vat_fiscal_position_id.
- Campos carryover: `carryover_origin_expression_label`, `carryover_origin_report_line_id` (M2O `account.report.line`).
- expression engine='external' con formula `'sum'` o `'most_recent'`.

### Pipeline

```
_compute_formula_batch_with_engine_external [4165]:
  domain = [('date','<=',date_to)]
  if date_from: domain += [('date','>=',date_from)]
  + ('company_id','in', report_company_ids)
  + fiscal_position filter (foreign_vat_fiscal_position_id)
  
  3 query types según figure_type:
    string → SELECT text_value
    monetary → SELECT SUM(CAST(value AS numeric) * rate) JOIN currency_table
    numeric → SELECT SUM(value)
  
  most_recent → GROUP BY date ORDER BY date DESC LIMIT 1
  
  NO soporta groupby/limit/offset (UserError if any)

editing manual (subformula='editable'):
  action_modify_manual_external_value [4969]:
    → create/update account.report.external.value para current column_group_options['date']
    → si existe value en el período: UPDATE
    → si no: CREATE con date=date_to del column_group

carryover generation [4308]:
  al posted del closing move → _generate_carryover_external_values
  recorre expresiones con carryover_target → crea external.value para próximo periodo
  multi-company: una external.value por company (split para auditoría)
```

### Gotchas

**ARG100EV.** Engine `external` NO soporta groupby, limit, ni offset (`if current_groupby or next_groupby or offset or limit: raise UserError`). Si pones `groupby` en la report.line con expression external → crash al unfold. Diagnóstico: el line da error solo al expandir, no al ver collapsed. Ref `account_reports/models/account_report.py:4178-4179`, `AccountReport._compute_formula_batch_with_engine_account_codes`.

**ARG101EV.** `most_recent` con figure_type='string' devuelve `text_value`; con figure_type numérico devuelve `value` agregado del último date. Si mezclas figure_types entre external.values del mismo expression (e.g. user editó valor a "N/A" string en BD numérica) → query monetary intenta CAST → PostgreSQL error o devuelve 0. Sin validación pre-edit. Ref `account_reports/models/account_report.py:4216-4234`, `AccountReport._compute_formula_batch_with_engine_account_codes`.

**ARG102EV.** `foreign_vat_fiscal_position_id` en external.value: debe coincidir con `target_report_expression_id.report_line_id.report_id.country_id` (constraint `_check_fiscal_position`:914-918). Si modificas country del report después de crear external.values → constraint violated en próximo save. No auto-fix.

**ARG103EV.** Carryover external_values se ondelete=cascade vía expression. Si borras la expression (config update / module uninstall) → carryover history SE PIERDE. No hay archive intermedio. Cross-ref ARG86RT.

**ARG104EV.** Multi-company carryover: `_generate_carryover_external_values` split por company. Si tax_unit cambia composición (company sale del unit) → carryover de esa company queda "huérfano" (target_report_expression válido pero company no en current unit) → invisible en current report. Mitigation: re-import o move manual. Ref `account_reports/models/account_report.py:4375-4422`, `AccountReport._generate_carryover_external_values`.

**ARG105EV.** Editable external values (`subformula='editable'`): el UPDATE/CREATE usa `date_to` del column_group_options. Si el user edita en una vista comparativa Y-o-Y mostrando current + previous year → la edición se guarda en current period. Para editar previous → cambiar period filter primero. Confuso, sin tooltip. Ref `_action_modify_manual_external_value:4969`.

**ARG106EV.** `name` field es required pero el engine NO lo usa (solo para list view display). Auto-generación al crear via UI: name = expression.label + date. Imports masivos sin name → falla con required violation, sin auto-fill desde target_report_expression_id. Ref `account_reports/models/account_report.py:890`, `class AccountReport` (required=True).

---

## Matriz de interacción cross-bridge

| Bridge A | Bridge B | Interacción |
|---|---|---|
| **account/AML** | **tax_tags** | Tax tag SQL no excluye `tax_closing_report_id` AMLs (ARG46A) ni borradores por defecto (ARG58T). |
| **account/AML** | **audit** | Engine `account_codes` con D/C imposible auditar exactamente (ARG67AU); cross-ref ARG43A. |
| **tax_tags** | **multi_company** | Multi-VAT: tag.country_id filtra; `fp_ids_to_exclude` puede dejar leaks entre regions del mismo país (ARG59T). |
| **variants_sections** | **multi_company** | `tax_units` requires country_id pero options se evalúan antes del reroute → desincronización (ARG73V). |
| **return_type** | **external_values** | Carryover: tax closing genera external.value para próximo period (ARG86RT, ARG103EV). |
| **return_type** | **multi_currency** | Closing entry usa company.currency_id; currency_table NO se aplica al SELECT del cierre → desfase en multi-currency consolidado. |
| **multi_currency** | **export** | XLSX exporta currency code column si `_add_xlsx_currency_codes_columns` activo; PDF NO incluye conversion rate explícito → auditoría externa difícil. |
| **audit** | **analytic** | `_get_audit_line_domain` añade analytic_distribution filter (ARG47AN), pero el report base no lo aplicó → audit y línea pueden no cuadrar. |
| **external_values** | **multi_company** | foreign_vat_fiscal_position_id constraint vs report.country_id (ARG102EV); split por company en carryover (ARG104EV). |
| **return_type** | **variants_sections** | Modelo 303/347/390 son variants del root `account.generic_tax_report`. Reroute via `selected_variant_id` antes de generate_tax_closing_entries. |

---

## v19 — Cambios bridges

Cobertura core en `v19_changes.md`. Bridges v19 specific:

- **account_return v19**: nuevo modelo `account.return` formaliza el ciclo declaración (review/submit/pay). `account.return.type` define el tipo (categoría `account_return`/`audit`, states_workflow). `account.return.check` valida pre-submit. Reemplaza la lógica embebida v18 de `_generate_tax_closing_entries` con un workflow stateful. Ref `odoo_enterprise/19/account_reports/models/account_return.py`.
- **account_analytic_line v19**: nuevo modelo bridge — exposición directa al reports engine para analytic puro. Ref `odoo_enterprise/19/account_reports/models/account_analytic_line.py`.
- **account_audit_account_status v19**: nuevo modelo para tracking de audit por cuenta (estado revisión por accountant). Ref `odoo_enterprise/19/account_reports/models/account_audit_account_status.py`.
- **kpi_provider v19**: nuevo modelo para inyectar KPIs en dashboards desde reports (sustituye dashboard.kpi parcial).
- **mail_scheduled_message v19**: integración con account.report.send para programar envíos diferidos.
- **ir_module v19**: hooks para auto-link de reports al instalar localizations (auto-create de variants).
- **currency_table CTA v19**: refinamientos al rate type per account_type (revisar bug ARG88FX en v19).

---

## Configuración mínima recomendada por caso de uso

### Modelo 303 trimestral España (Tax Return v19)

```yaml
account.report:
  country_id: ES
  root_report_id: account.generic_tax_report
  filter_multi_company: tax_units
  filter_fiscal_position: True
  custom_handler_model_id: l10n_es_reports.aeat.tax.report.handler

account.return.type (v19):
  report_id: <mod_303 report>
  category: account_return
  states_workflow: generic_state_tax_report  # Review, Submit, Pay
  deadline_periodicity: trimester
  deadline_days_delay: 20  # día 20 del mes siguiente

account.tax.unit (multi-company VAT consolidado):
  company_ids: [parent_ES, branch_Madrid, branch_Barcelona]
  country_id: ES
  main_company_id: parent_ES
```

### Balance Sheet consolidado multi-currency

```yaml
account.report:
  root_report_id: account.balance_sheet_report
  filter_multi_company: selector
  currency_translation: cta  # equity historical, P&L average, BS current

# Activar ≥2 currencies en res.currency
# Configurar rates en res.currency.rate con date_from periodicidad
```

### Custom report con engine domain + groupby partner_id + audit

```yaml
account.report:
  name: "Sales per Customer Region"
  line_ids:
    - name: "Receivable per Region"
      code: REGION_REC
      groupby: partner_id.country_id,partner_id,id
      expression_ids:
        - label: balance
          engine: domain
          formula: "[('account_id.account_type', '=', 'asset_receivable')]"
          subformula: sum
          auditable: True
          date_scope: strict_range
  column_ids:
    - name: Balance
      expression_label: balance
      figure_type: monetary
```

---

## Cross-refs activos

- `enterprise/account_reports/core.md` — engine principal, modelos base.
- `enterprise/account_reports/gotchas.md` — ARG1-37 core gotchas (no bridges).
- `enterprise/account_reports/v19_changes.md` — refactor v19, `account.return`.
- `functional/account/integrations.md` — AML, taxes, fiscal_position desde la perspectiva de account.
- `functional/account/gotchas.md` — taxes, repartition_lines, use_in_tax_closing.
- `ecosystem/l10n_es/` — implementación concreta Modelo 303/347/349/390 (BoE export, AEAT).
- `enterprise/account_consolidation/` — bridge para consolidación contable formal (no abordado aquí).
- Memory `project_aeat_submit_native.md`, `reference_odoo_native_aeat_engine.md` — motor AEAT en Odoo native.
