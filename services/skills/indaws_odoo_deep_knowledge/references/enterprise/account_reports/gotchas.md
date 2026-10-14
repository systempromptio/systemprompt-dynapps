<!-- verified: v18.0 @ 2026-06 -->
# Account Reports Gotchas — Odoo 18.0 Enterprise

## Expression Engine

**ARG1.** `sum_if_pos` / `sum_if_neg` evaluates the sign on the **total across ALL groupby keys**, not per key. If a line groups by partner and the grand total is positive, `sum_if_pos` returns values for ALL partners (even those with negative individual balances). This is intentional but confusing. _ES: En un informe agrupado por cliente, salen importes positivos para clientes que en realidad tienen saldo negativo._

**ARG2.** `account_codes` D/C suffix checks the **entire account total**, not the per-groupby-key amount. If account 411 has a total of -500, then `411D` evaluates to 0 for every groupby key under that account, even if some individual partners have positive balances. _ES: En el informe la columna de debe/haber de una cuenta sale en cero aunque algunos clientes de esa cuenta tengan saldo positivo._

**ARG3.** Aggregation engine evaluates **last** — after all other engines. If you reference a `domain` or `tax_tags` expression from an `aggregation` expression, the referenced expression must be evaluated first. This works by design, but if you create a circular dependency between aggregation expressions, you get an infinite loop (the `to_treat` list never empties). _ES: El informe se queda colgado y no termina de calcular una línea que suma otras._

**ARG4.** `cross_report` subformula forces the aggregation to evaluate expressions from OTHER reports using the **forced date scope** of the original expression. If you mix `cross_report` with bounds (`if_above`, `if_below`), bounds are applied to the final result, NOT per-expression. Split into two expressions as a workaround. _ES: Una línea que combina datos de otro informe con límites máximos o mínimos aplica el tope al total, no a cada parte, y da un resultado inesperado._

**ARG5.** The `_default` prefix on expression labels (e.g., `_default_balance`) is special: these expressions are filtered out of normal computation. They provide default values and are only included when `include_default_vals=True`. Naming an expression `_default_anything` will silently exclude it from results. _ES: Una línea del informe no aparece nunca en los resultados aunque la haya configurado bien._

**ARG6.** `_carryover_` and `_applied_carryover_` prefixed expression labels are special. They trigger carryover logic (info popups, external value generation). Do not use these prefixes unless implementing tax carryover. _ES: Al usar cierto nombre en una fórmula del informe se disparan popups y valores raros de arrastre de impuestos que no había pedido._

**ARG7.** `_currency_` prefixed expression labels (e.g., `_currency_balance`) are NOT a forced-currency mechanism. They drive the **XLSX currency-code columns**: `_add_xlsx_currency_codes_columns` (`account_report.py → _add_xlsx_currency_codes_columns`) strips the `_currency_` prefix and, for each matching real column (e.g., `balance`), injects an extra `_xlsx_currency_code_<label>` column in XLSX exports showing the currency code. Only relevant for XLSX export; ignored on screen/PDF. (Earlier docs claimed this forced a `res.currency` ID display — that is false.) _ES: Al exportar el informe a Excel aparecen columnas extra con el código de moneda que no salen en pantalla ni en el PDF._

## Performance

**ARG8.** The `domain` engine batches queries when all domain terms reference a single M2O field on AML. If your domain mixes fields (e.g., `[('account_id.code', '=like', '41%'), ('partner_id', '!=', False)]`), it falls back to a non-batched individual query per expression. This can cause N+1 query patterns on reports with many lines. _ES: El informe va muy lento y hace muchísimas consultas cuando el filtro mezcla varios campos distintos._

**ARG9.** `count_rows` subformula always forces non-batched execution. If one expression on a line uses `count_rows`, ALL expressions sharing that formula lose batching benefits. _ES: Al añadir un conteo de filas a una línea, todo el informe se vuelve mucho más lento._

**ARG10.** `_compute_expression_totals_for_each_column_group` iterates over ALL column groups independently. With 12 monthly comparison columns and horizontal groups, you get `12 * N_horizontal_groups` independent evaluation passes. Each pass runs ALL engine queries. This is the #1 performance bottleneck for complex reports. _ES: El informe con muchas columnas comparativas mensuales tarda muchísimo en abrirse._

**ARG11.** The `NO_NEXT_GROUPBY_ENGINES` set (`tax_tags`, `account_codes`) forces `next_groupby=None`. This enables better batching but means these engines cannot produce per-subline results when expanding groupby lines. They always return aggregated values. _ES: Al desplegar líneas de impuestos o de códigos de cuenta no se ve el detalle por cliente, solo el total agrupado._

**ARG12.** `unfold_all` with many lines triggers `_fully_unfold_lines_if_needed`, which iterates and recursively expands. Without `_custom_unfold_all_batch_data_generator` in the custom handler, each line expansion runs its own queries. General Ledger implements this optimization; custom reports often don't. _ES: Desplegar todas las líneas de un informe personalizado va lentísimo, mientras que el Libro Mayor estándar va rápido._

**ARG13.** The currency table is created as a temporary table (`account_currency_table`) for multi-currency reports. If `currency_table.type == 'monocurrency'`, it's replaced by an inline SQL subquery instead. Check `_init_currency_table()` — it only creates the temp table when needed, but **must** be called before any query using it. Forgetting this in custom code causes "relation does not exist" errors. _ES: El informe multidivisa da un error de tabla inexistente al calcular._

**ARG14.** `prefix_groups_threshold` (default: 4000) triggers automatic line grouping by name prefix when a report has too many lines. This happens transparently and can confuse users who expect to see all lines. Set to 0 to disable. _ES: El informe agrupa automáticamente las líneas por prefijo del nombre y el usuario no ve todas las líneas que esperaba._

## Tax Report Specifics

**ARG15.** Tax tag matching uses `SUBSTRING(tag_name, 2)` to strip the `+`/`-` prefix, then groups by the remaining string. If two tags share the same name after stripping the sign prefix (which should always be the case since tags are created in pairs), they are grouped together. Creating a tag manually without its counterpart will produce incorrect results. _ES: Un informe de impuestos da importes incorrectos porque se creó una etiqueta de impuesto suelta sin su pareja de signo contrario._

**ARG16.** `tax_tag_invert` on AML is an often-overlooked field. It flips the sign of the tag contribution. This is set by the tax computation when a tax line's sign differs from the base line's sign (e.g., credit notes). Not understanding this field leads to sign errors in custom tax reports. _ES: En un informe de impuestos personalizado los signos salen cambiados, sobre todo en facturas rectificativas/abonos._

**ARG17.** Tax closing (`_generate_carryover_external_values`) handles multi-company by: _ES: En el cierre de impuestos con varias empresas, el arrastre por empresa no cuadra con el total combinado._
   1. Computing carryover for all companies combined
   2. Computing carryover per company individually
   3. Adding an **adjustment** on the main company to reconcile the difference (due to bounds like `if_between`)
   This means carryover values per company may not add up to the total. This is by design.

**ARG18.** Fiscal position filtering on tax reports uses an OR between the fiscal position domain AND a tax-tag-based domain. This means AMLs wearing a tax tag matching the report's country are included regardless of their fiscal position, UNLESS they belong to another region's fiscal position. This is the `get_foreign_vat_tax_tag_extra_domain` logic and is extremely non-obvious. _ES: En el informe de impuestos aparecen apuntes que no deberían por la posición fiscal, o al revés, dependiendo del país._

## Multi-Company & Currency

**ARG19.** `filter_multi_company='tax_units'` does NOT simply aggregate all companies. It uses `account.tax.unit` records to determine which companies to combine. The companies must all be accessible to the user (checked via `self.env.user.company_ids`). If a user doesn't have access to one company in a tax unit, the entire tax unit is excluded. _ES: Al consolidar empresas por unidad fiscal, si el usuario no tiene acceso a una de las empresas desaparece toda la unidad fiscal del informe._

**ARG20.** The currency table supports three modes: `monocurrency` (no conversion), `current` (latest rate at report date), and `cta` (Current Temporal Approach with historical/average/current rates depending on account type). CTA mode joins differently — it uses `rate_type` matched to account type patterns: _ES: La conversión de moneda del informe sale mal porque aplica tipo de cambio histórico, medio o actual según el tipo de cuenta._
   - `equity%` accounts → `historical` rate
   - `income%` / `expense%` / `equity_unaffected` → `average` rate
   - Everything else → `current` rate
   Getting this wrong produces silently incorrect currency conversions.

**ARG21.** `_get_sender_company_for_export()` determines the "main" company for exports. In a tax unit, it's the tax unit's main company. Otherwise, it's the first company in options. This affects VAT number display, carryover adjustments, and XML export sender identification. _ES: En la exportación (XML, NIF, arrastres) el informe usa una empresa 'principal' que no es la que esperaba._

## Options & Filters

**ARG22.** The `is_opening_report` key in previous_options resets the journal filter to the first journal group. This happens every time the report is opened from the menu. Setting `not_reset_journals_filter=True` prevents this but must be done explicitly (e.g., when navigating from one report to another via caret options). _ES: Cada vez que abro el informe desde el menú se me resetea el filtro de diarios al primer grupo._

**ARG23.** `_get_options_journals_domain()` returns an empty list (no filter) when NO journals are specifically selected. This means "none selected = all selected". If you expect empty selection to mean "no data", you'll get all data instead. _ES: Si no selecciono ningún diario el informe muestra todos los datos en vez de ninguno._

**ARG24.** `forced_domain` in options is additive — it's appended to `_get_options_domain()`. But `forced_domain` in column group data is ALSO appended. If you set both, you get both. This is used by horizontal groups to filter by specific dimension values per column. _ES: Al configurar filtros forzados por columna y por informe a la vez, se aplican los dos y el resultado sale doblemente filtrado._

**ARG25.** Options comparison's `period_order` can be `'ascending'` or `'descending'`. When ascending, column headers are reversed but the data computation order stays the same. The reversal happens in `_init_options_column_headers`. This only affects display, not computation. _ES: Al cambiar el orden ascendente/descendente de los periodos comparativos solo cambian las cabeceras, no los datos._

**ARG26.** Budget columns use a **shadow AML table** (`account_report_budget_temp_aml`) that inherits from `account_move_line` but fills most fields with NULL/defaults. This means any domain that filters on fields not populated by the budget (e.g., `partner_id`, `journal_id`) will exclude all budget lines. Budget items only have `account_id`, `date`, `balance`, `company_id`. _ES: Las columnas de presupuesto desaparecen del informe en cuanto filtro por cliente, diario u otro campo que el presupuesto no tiene._

## Line ID & Hierarchy

**ARG27.** Line IDs are strings with `|` as hierarchy delimiter and `~` as field delimiter within each segment. The `|` character is the module constant `LINE_ID_HIERARCHY_DELIMITER = '|'` (`account_reports/models/account_report.py:47`, `NUMBER_FIGURE_TYPES`, defined in the EE class). Build with `_build_line_id` (`LINE_ID_HIERARCHY_DELIMITER.join(f'{markup}~{model}~{value}')`), parse with `_parse_line_id` which does `key.rsplit('~', 2)` per segment. Because it's `rsplit('~', 2)`, a `~` inside the **markup** is tolerated (only the last two `~` split model/value), but a `|` anywhere in any field breaks parsing. JSON-encoded markups (dicts) are safe because `json.dumps` doesn't produce a bare `|`. _ES: El informe da error al procesar líneas cuando un valor de la línea contiene un carácter de barra vertical._

**ARG28.** `_create_hierarchy()` only processes lines whose last segment has `model='account.account'`. Lines for other models (partners, AMLs) are treated as children of the last account line. If your dynamic lines don't follow this convention, hierarchy mode will misplace them. _ES: En modo jerárquico las líneas personalizadas (clientes, apuntes) se colocan mal, colgando de la cuenta equivocada._

**ARG29.** Totals below sections (`_add_totals_below_sections`) are first gated by `self.env.company.totals_below_sections` (a company setting) and skipped entirely if `options['ignore_totals_below_sections']`. When active, a line gets a total below if it is `unfoldable`, OR (`unfolded` AND has an `expand_function`), OR is the `parent_id` of another line — AND `any(col.get('no_format') is not None for col in line_dict['columns'])` (i.e. at least one column carries a value). A line that is expandable but whose columns all have `no_format is None` will NOT get a total line. (`account_report.py → _add_totals_below_sections`.) _ES: Algunas líneas expandibles del informe no muestran su línea de total por debajo aunque debería aparecer._

## Export & Display

**ARG30.** PDF export uses `export_mode='print'` which changes behavior: unfold_all becomes True (unless specific lines are unfolded), load_more_limit is ignored, and `hide_0_lines` is applied via `_filter_out_0_lines()`. The print output can differ significantly from the screen display. _ES: El PDF impreso del informe sale distinto de lo que se ve en pantalla, con todo desplegado y sin las líneas en cero._

**ARG31.** XLSX export calls `_filter_out_folded_children()` which removes children of folded lines. If you want all data in XLSX, ensure `unfold_all=True` in options. The default behavior when printing is to auto-unfold. _ES: Al exportar a Excel faltan las líneas hijas de las secciones plegadas._

**ARG32.** `_format_column_values()` skips columns that already have a `name` key. Custom handlers can pre-format columns by setting `name` in the column dict, which prevents the engine from reformatting. But this also means `rounding_unit` changes won't apply to pre-formatted columns. _ES: Al cambiar la unidad de redondeo, algunas columnas del informe no se actualizan porque venían preformateadas._

**ARG33.** The `show_debug_column` option requires FOUR conditions together (`account_report.py → _init_options_columns`, near line 1662): `export_mode != 'print'` AND `self.env.user.has_group('base.group_no_one')` (developer mode) AND `len(options['column_groups']) == 1` (single column group) AND `len(self.line_ids) > 0` (has static lines). So even an admin won't see the debug column without developer mode, and fully dynamic reports (like GL, no static `line_ids`) never show it by default. _ES: La columna de depuración del informe no aparece aunque sea administrador._

**ARG34.** `integer_rounding_enabled` defaults to True in options but is only active when `self.integer_rounding` is set on the report. When `export_mode='file'`, integer rounding is forced on. This means file exports can differ from screen display if the user toggled rounding off. _ES: El fichero exportado del informe muestra importes redondeados a entero distintos de lo que se ve en pantalla._

## Annotations

**ARG35.** Report annotations are stored with the **line_id** (string) and a date. The `_remove_tax_grouping_from_line_id` strips `account.group` segments from the line_id before storing. This means annotations survive hierarchy toggling, but if the report structure changes (different markup, different model in the ID), annotations become orphaned. _ES: Las anotaciones del informe se pierden o quedan huérfanas cuando cambia la estructura del informe._

## Variant & Section System

**ARG36.** `get_options()` performs a reroute check after computing `report_id`. If `report_id != self.id`, it calls `get_options()` on the target report recursively. This means the first ~17 option initializers run on the root report, then ALL initializers run again on the variant. Any side effects in the first 17 initializers run twice. _ES: Al abrir una variante del informe se ejecutan dos veces ciertos cálculos iniciales, con efectos secundarios duplicados._

**ARG37.** The `no_report_reroute` key in options prevents the reroute and forces options to be computed for the current report. This is used internally for export operations that need to work on the exact report they're called on. If you see unexpected report switching during exports, check this flag. _ES: Durante una exportación el informe cambia inesperadamente a otro informe distinto._

## Engine Edge Cases (added)

**ARG38.** In the `account_codes` engine, `D`/`C` are reserved as debit/credit total selectors placed right after a prefix (`411D`, `411C`). If an actual account prefix legitimately ends in `C` or `D`, you must disambiguate with an **empty exclusion** `\()`: e.g. write `411D\()` to mean "prefix `411D`" rather than "prefix `411`, debit only". Documented in `account_report.py → _compute_formula_batch_with_engine_account_codes` docstring. Easy to get silently wrong on charts where account codes use trailing letters. _ES: En códigos de cuenta que terminan en letra C o D el informe interpreta mal el prefijo y saca solo debe o haber sin querer._

**ARG39.** `currency_translation` (the report field, CE `account/models/account_report.py`) only offers two values: `current` ("most recent rate at report date") and `cta` ("Use CTA"). There is NO `monocurrency` selection. `monocurrency` is decided at runtime by `res.currency._check_currency_table_monocurrency(companies)` in `_init_options_currency_table`: if all involved companies share the report currency, `options['currency_table']['type']` is forced to `'monocurrency'` regardless of the field. So you cannot pin a report to monocurrency via the field — it's auto-detected. _ES: No consigo fijar el informe a moneda única desde la configuración; se decide solo según las empresas implicadas._

**ARG40.** The `external` engine's `most_recent` formula returns the value of the **single latest** `account.report.external.value` in the period (`GROUP BY date ORDER BY date DESC LIMIT 1`), and that value may be **text**, not a number (used for string KPIs). Code consuming external-engine results must not assume a numeric `value`. `sum` instead adds all external values in the period. (`account_report.py → _compute_formula_batch_with_engine_external`.) _ES: Un valor externo del informe (KPI) puede ser texto en vez de número y rompe cálculos que esperan un importe._

**ARG41.** `sum_children` (aggregation engine) is resolved by expanding it into `_expression:<child_expr_id>` terms for each child line sharing the **same expression label** (`account_report.py → _compute_expression_totals_for_each_column_group`, near line 3151). It is NOT a SQL sum: it depends on the children's expressions being evaluated first, which is why it lives in the deferred `_compute_totals_no_batch_aggregation` pass. A child line without an expression of the same label contributes nothing (silently), not an error. _ES: Una línea que suma sus hijas da menos de lo esperado porque alguna hija no tiene la misma expresión y no suma, sin avisar._
