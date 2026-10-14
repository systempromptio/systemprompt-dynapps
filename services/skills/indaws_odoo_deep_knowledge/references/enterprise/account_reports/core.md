<!-- verified: v18.0 @ 2026-06 -->
# Account Reports Engine — Odoo 18.0 Enterprise

> Source: `odoo_enterprise/18/account_reports/`
> Main engine: `models/account_report.py` (7658 lines)
> Depends: `accountant` (which depends on `account_accountant`)

## 1. Architecture Overview

The account_reports module is the **central financial reporting engine** for Odoo Enterprise. It provides a configurable, data-driven framework where reports are defined as data (XML records) rather than code.

### Core Design

```
account.report (CE base model in `account`)
    └── account.report (Enterprise _inherit in account_reports)
         ├── account.report.line (static lines, defined in XML)
         │    └── account.report.expression (formulas, engines)
         ├── account.report.column (column definitions)
         ├── account.report.custom.handler (AbstractModel for report-specific logic)
         └── account.report.horizontal.group (multi-dimensional grouping)
```

### Flow: From UI to Data

1. **JS Controller** calls `get_options(previous_options)` on the report
2. `get_options()` runs ~40 `_init_options_*` initializers in sequence (ordered by `_get_options_initializers_forced_sequence_map`)
3. If report has variants/sections, it **reroutes** to the correct sub-report
4. JS calls `_get_lines(options)` which:
   a. Calls `_compute_expression_totals_for_each_column_group()` for all static line expressions
   b. Calls `_get_dynamic_lines()` via the custom handler (if any)
   c. Merges static + dynamic lines by sequence
   d. Applies hierarchy, totals-below-sections, unfold-all
   e. Runs `_custom_line_postprocessor` and formats values

### Report Types

Reports are purely **data-driven** via XML records. The `custom_handler_model_id` field points to an AbstractModel that provides report-specific logic (dynamic lines, custom options, etc.).

**No custom handler needed** for simple reports that only use expressions (Balance Sheet, P&L use account_codes engine with pure XML definitions).

## 2. account.report Model

### Base Fields (defined in CE `account` module)

| Field | Type | Description |
|-------|------|-------------|
| `name` | Char | Report name (translatable) |
| `line_ids` | O2M | Static report lines |
| `column_ids` | O2M | Column definitions |
| `root_report_id` | M2O | Parent report (for variants) |
| `variant_report_ids` | O2M | Localized variants |
| `section_report_ids` | M2M | Sub-reports (composite reports) |
| `country_id` | M2O | Country (for localized variants) |
| `chart_template` | Selection | Chart of accounts filter |
| `load_more_limit` | Integer | Pagination limit for unfoldable lines |
| `search_bar` | Boolean | Enable search bar |
| `prefix_groups_threshold` | Integer | Auto-group lines by prefix (default: 4000) |
| `integer_rounding` | Selection | HALF-UP / UP / DOWN |

### Filter Fields (all Boolean/Selection, control UI menus)

| Filter | Default | Description |
|--------|---------|-------------|
| `filter_multi_company` | disabled | disabled / selector / tax_units |
| `filter_date_range` | True | Range vs. single date mode |
| `filter_show_draft` | True | Show draft entries toggle |
| `filter_unreconciled` | False | Unreconciled entries only |
| `filter_unfold_all` | False | Allow unfold-all button |
| `filter_hide_0_lines` | optional | by_default / optional / never |
| `filter_period_comparison` | True | Period comparison |
| `filter_growth_comparison` | True | Growth % column |
| `filter_journals` | False | Journal filter |
| `filter_analytic` | False | Analytic account filter |
| `filter_hierarchy` | optional | Account group hierarchy |
| `filter_account_type` | disabled | Trade/non-trade receivable/payable |
| `filter_partner` | False | Partner filter |
| `filter_fiscal_position` | False | Multi-VAT fiscal position |
| `filter_aml_ir_filters` | False | User-defined AML filters |
| `filter_budgets` | False | Budget comparison |

### Enterprise-added Fields

| Field | Type | Description |
|-------|------|-------------|
| `custom_handler_model_id` | M2O(ir.model) | Points to custom handler |
| `horizontal_group_ids` | M2M | Horizontal grouping rules |
| `annotations_ids` | O2M | Report annotations |
| `send_and_print_values` | Json | Async send & print state |
| `tax_closing_start_date` | Date | Company-dependent tax period start |

### account.report.line

| Field | Description |
|-------|-------------|
| `name` | Line label (translatable) |
| `code` | Short code for cross-references in aggregation formulas |
| `expression_ids` | O2M to `account.report.expression` |
| `sequence` | Display order |
| `hierarchy_level` | Indentation level (0-4) |
| `parent_id` | M2O to parent line |
| `foldable` | User can fold/unfold |
| `groupby` | Comma-separated groupby fields (e.g., `partner_id,account_id,id`) |
| `user_groupby` | User-configurable groupby |
| `hide_if_zero` | Hide when all values = 0 |
| `print_on_new_page` | Page break in PDF |
| `action_id` | Click action |
| `horizontal_split_side` | left/right for horizontal split layouts |

### account.report.column

| Field | Description |
|-------|-------------|
| `name` | Column header |
| `expression_label` | Links to expression label (e.g., `balance`, `debit`, `credit`) |
| `figure_type` | monetary / percentage / integer / float / date / boolean / string (`FIGURE_TYPE_SELECTION_VALUES`; column default `monetary`, required) |
| `sortable` | Allow column sorting |
| `blank_if_zero` | Show empty instead of 0 |

## 3. Expression Engine

Each `account.report.expression` defines how a cell value is computed.

### Fields

| Field | Description |
|-------|-------------|
| `label` | Identifier (e.g., `balance`). Links to column's `expression_label` |
| `engine` | One of: `domain`, `tax_tags`, `account_codes`, `aggregation`, `external`, `custom` |
| `formula` | Engine-specific formula |
| `subformula` | Modifier (e.g., `sum`, `sum_if_pos`, `editable`, `cross_report`) |
| `date_scope` | When to evaluate: `strict_range`, `from_beginning`, `from_fiscalyear`, `to_beginning_of_period`, `to_beginning_of_fiscalyear`, `previous_tax_period` |
| `figure_type` | Override column's figure_type |
| `green_on_positive` | Color coding direction |
| `blank_if_zero` | Override column's blank_if_zero |
| `auditable` | Allow drill-down audit |
| `carryover_target` | Target expression for tax carryover |

### Engine: `domain`

Formula is a Python domain on `account.move.line`.

```python
# Formula examples:
"[('account_id.account_type', '=', 'asset_receivable')]"
"[('account_id.code', '=like', '41%')]"
```

**Subformulas:** `sum` (default), `sum_if_pos`, `sum_if_neg`, `-sum_if_pos`, `-sum_if_neg`, `count_rows`

Sign policy: `sum_if_pos`/`sum_if_neg` check the **total** across all groupby keys, not per-key. If total is positive, `sum_if_pos` returns values; `sum_if_neg` returns 0 (and vice versa).

**Batching optimization:** Domains filtering on a single M2O field of AML are batched together into one SQL query with an extra GROUP BY on that field.

### Engine: `tax_tags`

Formula is a tag name (e.g., `+tag_name`). The engine finds `account.account.tag` records with names `+tag_name` and `-tag_name`, then sums AML balances weighted by the tag sign and `tax_tag_invert`.

```sql
SUM(balance * CASE WHEN tag.tax_negate THEN -1 ELSE 1 END
            * CASE WHEN aml.tax_tag_invert THEN -1 ELSE 1 END)
```

### Engine: `account_codes`

Formula targets account code prefixes with arithmetic.

```
# Syntax:
"123"           -- sum of all accounts starting with 123
"123 - 456"     -- difference
"123D"          -- only if account total is Debit (positive)
"123C"          -- only if account total is Credit (negative)
"123\(1234)"    -- exclude sub-prefix 1234
"123\(1234,1236)"  -- exclude multiple
"tag(l10n_xx.tag_ref)"  -- by account tag XML ID
"tag(42)"       -- by account tag database ID
```

**D/C evaluation**: checks the **entire account's total** (not per-groupby-key). If account 123 total is negative, `123D` evaluates to 0 for ALL groupby keys of that account.

### Engine: `aggregation`

Formula references other expressions by line code:

```
"LINE_CODE.expression_label"
"CODE1.balance + CODE2.balance"
"sum_children"  -- auto-sum children lines' same-label expression
"_expression:42" -- by expression database ID
```

**Subformulas:**
- `cross_report` -- reference expressions in OTHER reports
- `if_above(CUR(amount))` -- return 0 if value <= amount
- `if_below(CUR(amount))` -- return 0 if value >= amount
- `if_between(CUR(a), CUR(b))` -- clamp to bounds
- `round(N)` -- round to N decimal places
- `if_other_expr_above(CODE.label, CUR(amount))` -- conditional on another expression
- `ignore_zero_division` -- return 0 on ZeroDivisionError

### Engine: `external`

Uses `account.report.external.value` records (manual values, carryover).

**Formulas:** `sum`, `most_recent`

**Subformulas:** `editable` (allows manual editing), `rounding=N`

### Engine: `custom`

Delegates to a Python function named in the formula (must start with `_report_custom_engine_`). The function is called on the custom handler model if it exists.

```python
def _report_custom_engine_my_computation(self, expressions, options, date_scope, current_groupby, next_groupby, offset=0, limit=None, warnings=None):
    # Must return same format as other engines
```

## 4. Custom Handler System

A custom handler is an `AbstractModel` inheriting `account.report.custom.handler`. It is linked to a report via `custom_handler_model_id`.

### Hook Methods

| Method | Purpose |
|--------|---------|
| `_dynamic_lines_generator(report, options, totals, warnings)` | Generate lines that cannot be expressed as static XML. Returns `[(sequence, line_dict), ...]` |
| `_custom_options_initializer(report, options, previous_options)` | Add report-specific options (extra buttons, filters) |
| `_custom_line_postprocessor(report, options, lines)` | Modify lines after generation (add columns, reorder) |
| `_custom_groupby_line_completer(report, options, line_dict)` | Customize groupby line dicts |
| `_custom_unfold_all_batch_data_generator(report, options, lines_to_expand)` | Batch data for efficient unfold-all |
| `_caret_options_initializer()` | Define right-click menu options |
| `_get_custom_display_config()` | Override JS templates and OWL components |
| `_get_custom_groupby_map()` | Define custom groupby fields beyond AML fields |
| `_customize_warnings(report, options, totals, warnings)` | Add report-specific warnings |

### Dispatch System

`dispatch_report_action(options, action, action_param)` routes button clicks:
1. Checks if custom handler has the method
2. Falls back to the report model
3. Validates method is public via `get_public_method()`

## 5. Standard Reports Included

| Report | Handler Model | Engine | Key Features |
|--------|--------------|--------|--------------|
| **Balance Sheet** | None (pure XML) | `account_codes` | Horizontal split (Assets/Liabilities), hierarchy |
| **Profit & Loss** | None (pure XML) | `account_codes` | account_codes expressions |
| **General Ledger** | `account.general.ledger.report.handler` | dynamic | Account lines + AML details, initial balance, cumulative sum |
| **Partner Ledger** | `account.partner.ledger.report.handler` | dynamic | Partner groupby, initial balance, due date columns |
| **Aged Receivable/Payable** | `account.aged.partner.balance.report.handler` | dynamic | Aging buckets (0-30, 31-60, etc.) |
| **Trial Balance** | `account.trial.balance.report.handler` | dynamic | Initial balance + period columns, hierarchy |
| **Tax Report** | `account.generic.tax.report.handler` | `tax_tags` | Tax tag mapping, XML export, closing entries, carryover |
| **Cash Flow Statement** | `account.cash.flow.report.handler` | domain + custom | Indirect method, financing/investing/operating |
| **Executive Summary** | None — `_inherit='account.report'` extension in `executive_summary_report.py` (no handler model) | custom | KPIs, performance metrics |
| **Journal Audit** | `account.journal.report.handler` | dynamic | Journal-wise entry listing |
| **EC Sales List** | `account.ec.sales.report.handler` (NOT `account.sales.report.handler`) | dynamic | EC sales list, country-wise |
| **Bank Reconciliation** | `account.bank.reconciliation.report.handler` | custom | GL balance vs bank statement |
| **Multicurrency Revaluation** | `account.multicurrency.revaluation.report.handler` | custom | Unrealized gains/losses, revaluation wizard |
| **Deferred Revenue/Expense** | `account.deferred.report.handler` (+ `account.deferred.expense.report.handler`, `account.deferred.revenue.report.handler`) | custom | Deferred entry generation |
| **Customer Statement** | `account.customer.statement.report.handler` (inherits `account.partner.ledger.report.handler`) | domain | Partner-specific statement |
| **Follow-up Report** | `account.followup.report.handler` | custom | Collection follow-up |
| **Analytic Report** | None — `_inherit='account.report'` extension in `account_analytic_report.py` (no handler model; uses `analytic_temp_account_move_line` shadow table) | custom | Analytic account analysis |
| **Budget** | `account.report.budget` model | special | Shadow AML table, budget vs actual |

### File Sizes (lines of code per handler)

- `account_generic_tax_report.py`: 1228 lines (most complex)
- `account_journal_report.py`: 1417 lines
- `account_partner_ledger.py`: 816 lines
- `account_general_ledger.py`: 763 lines
- `account_cash_flow_report.py`: 712 lines
- `account_deferred_reports.py`: 663 lines
- `bank_reconciliation_report.py`: 620 lines
- `account_aged_partner_balance.py`: 461 lines
- `account_sales_report.py`: 421 lines
- `account_multicurrency_revaluation_report.py`: 387 lines
- `account_analytic_report.py`: 257 lines
- `account_trial_balance_report.py`: 197 lines
- `budget.py`: 116 lines
- `account_followup_report.py`: 109 lines
- `account_customer_statement.py`: 24 lines
- `executive_summary_report.py`: 15 lines
- `balance_sheet.py`: 11 lines

## 6. Report Options & Filters

### Options Initialization Sequence

Options are built by `_init_options_*` methods, ordered by `_get_options_initializers_forced_sequence_map()`:

```
10:  companies
15:  variants
16:  sections
17:  report_id          ← REROUTE CHECK HAPPENS HERE
20:  fiscal_position
30:  date
40:  horizontal_groups
50:  comparison
60:  export_mode
70:  integer_rounding
80:  journals
90:  journals_names
200: [default] — all_entries, analytic, partner, account_type, reconciled, etc.
990: column_headers
1000: columns
1010: column_percent_comparison
1020: order_column
1030: hierarchy
1040: prefix_groups_threshold
1050: custom            ← Custom handler's _custom_options_initializer
1055: currency_table
1060: section_buttons
1070: readonly_query
```

### Date Scopes

| Scope | Meaning |
|-------|---------|
| `strict_range` | Exact date_from to date_to |
| `from_beginning` | All time up to date_to |
| `from_fiscalyear` | Fiscal year start to date_to |
| `to_beginning_of_period` | All time up to (date_from - 1 day) |
| `to_beginning_of_fiscalyear` | All time up to (fiscal year start - 1 day) |
| `previous_tax_period` | Previous tax closing period |

### Column Groups

Each comparison period + horizontal group combination creates a **column group**. Each column group gets its own set of `forced_options` (different dates, domains). Expressions are evaluated independently per column group via `_split_options_per_column_group()`.

### Fiscal Position (Multi-VAT)

When `filter_fiscal_position=True` and the report has a `country_id`:
- `domestic` — only lines without foreign VAT fiscal position
- `all` — all lines regardless of fiscal position
- `<fpos_id>` — only lines with that specific fiscal position
- Tax tag lines bypass fiscal position filtering (they use `tax_tag_ids.country_id` + exclusion of other regions)

## 7. Export Formats

### PDF Export

1. Calls `_get_lines()` with `export_mode='print'`
2. Renders QWeb template `account_reports.pdf_export_main`
3. Uses wkhtmltopdf with landscape for reports with >5 columns
4. Supports custom templates via `_get_custom_display_config()` -> `pdf_export`
5. Multi-section reports render each section, then merge PDFs

### XLSX Export

1. Uses `xlsxwriter` library
2. Renders all lines (unfolds all for print mode)
3. Splits account code + name into separate columns for account lines
4. Level-based formatting (bold for headers, indentation for hierarchy)
5. Custom handlers can override `_inject_report_into_xlsx_sheet()`
6. Adds a separate "Options" sheet with applied filters

### XML Export (Tax Reports)

Handled by localization modules (e.g., `l10n_xx_reports`) via custom handler buttons.

## 8. Extension Points

### Creating a Custom Report

1. **XML Data**: Define `account.report`, `account.report.line`, and `account.report.expression` records
2. **Custom Handler** (optional): Create an AbstractModel inheriting `account.report.custom.handler`
3. **Menu Item**: Create `ir.actions.client` with `tag='account_report'` and `context={'report_id': report_id}`

### Adding Columns to Existing Reports

Override the report's XML to add new `account.report.column` records and corresponding `account.report.expression` on each line.

### Modifying Lines

- **Static lines**: Override XML data to modify expressions
- **Dynamic lines**: Override the custom handler's `_dynamic_lines_generator`
- **Post-processing**: Override `_custom_line_postprocessor`

### Custom Engine Functions

```python
class MyHandler(models.AbstractModel):
    _name = 'my.report.handler'
    _inherit = 'account.report.custom.handler'

    def _report_custom_engine_my_formula(self, expressions, options, date_scope,
                                          current_groupby, next_groupby, offset=0,
                                          limit=None, warnings=None):
        # Return format:
        # Without groupby: {(formula, expressions): {'result': value, 'has_sublines': bool}}
        # With groupby: {(formula, expressions): [(grouping_key, {'result': value}), ...]}
```

### Line ID System

Line IDs encode hierarchy: `markup~model~value|markup2~model2~value2`

- `|` is the hierarchy delimiter
- Each segment: `markup~model_name~record_id`
- Parse with `_parse_line_id()`, build with `_get_generic_line_id()`
- Markup can be a JSON dict (e.g., `{"groupby": "partner_id"}`)

### Groupby Expansion

When a line has `groupby='partner_id,account_id,id'`:
1. First unfold shows partners (groupby=`partner_id`)
2. Unfolding a partner shows accounts (groupby=`account_id`)
3. Unfolding an account shows individual AMLs (groupby=`id`)

Load more is only active on terminal groupby levels.

## 9. Security & Access

### Access Groups

| Group | Read | Write | Create | Delete |
|-------|------|-------|--------|--------|
| `group_account_readonly` | Yes (annotations) | No | No | No |
| `group_account_user` | Yes | Yes | Yes | Yes |
| `group_account_invoice` | Yes (annotations) | No | No | No |
| `group_account_manager` | Yes | Yes | Yes | Yes (budget, horizontal groups) |

### Key Security Mechanisms

1. **`_get_report_query()`** applies `_apply_ir_rules(query)` on `account.move.line` — respects company multi-rule
2. **Company domain**: `('company_id', 'in', report_company_ids)` always added
3. **`dispatch_report_action()`**: validates method is public via `get_public_method()`
4. **Custom function names**: must start with `_report_` prefix (validated by `_get_custom_report_function()`)
5. **Branch handling**: export buttons disabled when not all branches selected (unless `branch_allowed=True`)
6. **`check_access('read')`** called on `account.move.line` before query execution
7. **Journal access**: respects `active_test=False` for archived journals
8. **`@api.readonly`** on `get_options()` — safe for read replicas

### Budget Security

Budget items are inserted into a **temporary table** (`account_report_budget_temp_aml`) that inherits from `account_move_line` schema. This allows reusing the same query engine but with synthetic data. The temp table is dropped on commit (`ON COMMIT DROP`).
