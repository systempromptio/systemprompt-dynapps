<!-- verified: v18.0 @ 2026-06 -->
# Account Accountant — Odoo 18.0 Enterprise

Full accounting module. Activates bank reconciliation, follow-ups, fiscal closing, and advanced accounting features.
Source: `odoo_enterprise/18/account_accountant/`. Method/field anchors below resolve under `models/`, `wizard/`, `data/`, `security/` of that module; line numbers are v18.0 hints.

**Module chain**: `account` (CE) -> `account_accountant` (EE, auto_install) -> `accountant` (EE, application=True) -> `account_reports`, `account_asset`, etc.

Note: `account_accountant` is auto_install=True (installs when `account` + `mail_enterprise` + `web_tour` are present). The `accountant` module depends on `account_accountant` and is the actual "Accounting" application entry point.

---

## 1. What account_accountant Activates

### Features NOT available in base `account` (CE Invoicing):
| Feature | CE `account` | EE `account_accountant` |
|---|---|---|
| Bank reconciliation widget | No (basic matching only) | Full interactive widget (`bank.rec.widget`) |
| Auto-reconciliation CRON | No | Yes — `_cron_try_auto_reconcile_statement_lines()` |
| Reconcile models (full) | Basic model definition | Full rule engine: invoice_matching, writeoff_suggestion, writeoff_button with auto-reconcile |
| Fiscal years | No model | `account.fiscal.year` model + `compute_fiscalyear_dates()` on res.company |
| Lock date wizard | No wizard | `account.change.lock.date` wizard with exceptions, hard lock, per-type locks |
| Deferred entries | No | Full deferred revenue/expense management (start/end dates on invoice lines, auto-generation) |
| Manual reconciliation wizard | No | `account.reconcile.wizard` (write-off, transfer between accounts, partial) |
| Auto-reconcile wizard | No | `account.auto.reconcile.wizard` (one-to-one perfect match, zero-balance clearing) |
| Predictive bill coding | No | `_predicted_field()`, `_predict_account()`, `_predict_taxes()`, `_predict_product()` on AML |
| Invoice signing | No | `signing_user`, `signature` fields on account.move |
| Invoicing switch threshold | No | `invoicing_switch_threshold` — hides old entries when migrating from Invoicing to Accounting |
| Digest KPI | No | `kpi_account_bank_cash` on digest.digest |
| Journal dashboard (enhanced) | Basic | Enhanced actions: `action_open_reconcile`, `action_open_to_check`, bank rec widget integration |

### Groups added:
- **`group_fiscal_year`** — Hidden group, allows defining fiscal years != 12 months. Toggled via Settings.
- **`account.group_account_basic`** renamed to "Invoicing & Banks" (was just "Invoicing"). Implies removal of `group_account_readonly`.

### Settings exposed (res.config.settings):
- `fiscalyear_last_day` / `fiscalyear_last_month`
- `use_anglo_saxon` (Anglo-Saxon accounting toggle)
- `invoicing_switch_threshold`
- `predict_bill_product`
- `sign_invoice` / `signing_user`
- Deferred expense: `deferred_expense_journal_id`, `deferred_expense_account_id`, `generate_deferred_expense_entries_method`, `deferred_expense_amount_computation_method`
- Deferred revenue: same four fields with `revenue` prefix

### CRON jobs:
- `account_accountant.auto_reconcile_bank_statement_line` — Runs `_cron_try_auto_reconcile_statement_lines()`, processes unreconciled statement lines in batches using reconcile models with `auto_reconcile=True`.

---

## 2. Bank Reconciliation Widget

### Architecture: In-memory transient models

The reconciliation widget uses **two models that are NEVER persisted to the database** (`_auto = False`, `_table_query = "0"`):

#### `bank.rec.widget` — Main wizard (one per statement line)
- **Key fields**:
  - `st_line_id` (Many2one -> account.bank.statement.line) — The statement line being reconciled
  - `move_id` (related -> st_line_id.move_id) — The underlying journal entry
  - `partner_id` (computed, store=True, readonly=False) — Auto-detected partner via `_retrieve_partner()`
  - `line_ids` (One2many -> bank.rec.widget.line) — All reconciliation lines
  - `state` (Selection: invalid/valid/reconciled) — Invalid if suspense account still present
  - `available_reco_model_ids` — Write-off button reconcile models for the current journal
  - `selected_reco_model_id` — Currently applied reconcile model
  - `matching_rules_allow_auto_reconcile` (Boolean) — Set by matching rules when confidence is high enough
  - `todo_command` / `return_todo_command` (Json, store=False) — JS<->Python communication bridge
  - `form_index` (Char) — Which line is currently in edit mode

- **State logic** (`bank_rec_widget.py → _compute_state`, `:198`):
  ```
  if not st_line_id           -> 'invalid'
  elif st_line.is_reconciled  -> 'reconciled'
  elif suspense_account in line_ids.account_id -> 'invalid'
  else                        -> 'valid'
  ```

#### `bank.rec.widget.line` — Individual line in the widget
- **Key field: `flag`** (Selection) — Determines the line's role:
  | Flag | Description |
  |---|---|
  | `liquidity` | The bank/cash journal item (always first line) |
  | `new_aml` | A matched existing journal item (invoice, payment, etc.) |
  | `aml` | An already-reconciled journal item (shown in reconciled state) |
  | `exchange_diff` | Auto-computed exchange difference line |
  | `tax_line` | Tax line from manual write-off with taxes |
  | `manual` | Manual write-off line |
  | `early_payment` | Early payment discount adjustment |
  | `auto_balance` | Auto-balancing line (suspense or receivable/payable) |

- **Key fields**: `account_id`, `partner_id`, `currency_id`, `amount_currency`, `balance`, `source_aml_id`, `source_amount_currency`, `source_balance`, `source_rate`, `tax_ids`, `tax_tag_ids`, `reconcile_model_id`, `manually_modified`
- **Suggestion system**: `suggestion_html`, `suggestion_amount_currency`, `suggestion_balance` — Shows contextual hints ("fully paid", "partial payment", etc.)
- **Partner info**: `partner_receivable_amount`, `partner_payable_amount` — Shows current balance for the partner
- **`_get_aml_values(**kwargs)`** — Converts widget line to journal item creation dict

### Reconciliation Flow

1. **Mount statement line**: `_action_mount_st_line(st_line)` sets `st_line_id`, computes `line_ids`, triggers matching rules
2. **Auto-matching**: `_action_trigger_matching_rules()` runs all non-button reconcile models via `_apply_rules()`
3. **User interaction** (via `todo_command` onchange -> `_js_action_*` methods):
   - `_js_action_mount_st_line(st_line_id)` — Switch to a different statement line
   - `_js_action_validate()` — Confirm reconciliation
   - `_js_action_reset()` — Undo reconciliation (checks hash integrity)
   - `_js_action_to_check()` — Mark as "to check" and optionally validate
   - `_js_action_set_as_checked()` — Mark as checked
   - `_js_action_remove_line(line_index)` — Remove a matched line
   - `_js_action_select_reconcile_model(reco_model_id)` — Apply a write-off model
   - `_js_action_mount_line_in_edit(line_index)` — Open line edit form
4. **Validation**: `_action_validate()`:
   - Collects all line values via `_validation_lines_vals()`
   - Determines partner (single partner across non-liquidity lines)
   - Writes to `move.line_ids` with `Command.clear()` + `Command.create()` for each line
   - Creates exchange difference moves if needed
   - Reconciles `new_aml` lines with their `source_aml_id` counterparts via `_reconcile_plan()`
   - Creates/links partner bank account if `account_number` present
   - Refreshes analytic lines

### Key Internal Methods

**Line preparation**:
- `_lines_prepare_liquidity_line()` — Creates the liquidity (bank account) line
- `_lines_prepare_auto_balance_line()` — Creates auto-balance (suspense or receivable/payable)
- `_lines_prepare_new_aml_line(aml)` — Creates counterpart for an existing journal item
- `_lines_prepare_reco_model_write_off_vals(reco_model, vals)` — Creates manual write-off from reconcile model

**Balance management**:
- `_lines_add_auto_balance_line()` — Ensures debit/credit balance, drops and recreates auto_balance
- `_lines_check_partial_amount(line)` — Checks if partial matching is possible
- `_lines_check_apply_early_payment_discount()` — Applies early payment discount if all conditions met
- `_lines_check_apply_partial_matching()` — Applies partial on the last new_aml line

**Exchange differences**:
- `_lines_get_account_balance_exchange_diff(currency, balance, amount_currency)` — Computes exchange diff amount and account (income/expense)
- `_lines_get_exchange_diff_values(line)` — Creates exchange_diff line dict
- `_lines_recompute_exchange_diff(lines)` — Recomputes exchange diffs for given lines

**Tax handling**:
- `_lines_recompute_taxes()` — Full tax recomputation using `account.tax._prepare_tax_lines()`

**Value change handlers** (called when user edits a line field):
- `_line_value_changed_account_id(line)` — Recomputes taxes if needed
- `_line_value_changed_amount_currency(line)` — Handles amount changes, partial capping, exchange diff recompute
- `_line_value_changed_balance(line)` — Similar but for company currency balance
- `_line_value_changed_partner_id(line)` — Updates account to receivable/payable based on partner type
- `_line_value_changed_tax_ids(line)` — Manages price_included mode transitions
- `_line_value_changed_name(line)` — If liquidity line, updates statement line payment_ref
- `_line_value_changed_date(line)` — If liquidity line, updates statement line date

**Reconcile model integration**:
- `_action_select_reconcile_model(reco_model)` — Applies a reconcile model (write-off or invoice creation)
- `_create_invoice_from_write_off_values(reco_model, vals_list)` — Creates an invoice/bill from a sale/purchase type reconcile model

### Auto-Balance Logic

The `auto_balance` line determines whether the widget shows "valid" or "invalid":
- If partner is set: uses receivable/payable account based on `customer_rank`/`supplier_rank` -> **valid**
- If no partner: uses `journal.suspense_account_id` -> **invalid** (suspense account detected)

---

## 3. Bank Statement Processing

### Statement Model Extensions (`account.bank.statement`)
- Inherits `mail.thread.main.attachment`
- `action_open_bank_reconcile_widget()` — Opens reconciliation widget filtered by statement
- `action_generate_attachment()` — Generates PDF attachment of the statement

### Statement Line Extensions (`account.bank.statement.line`)
- **New field**: `cron_last_check` (Datetime) — Tracks when CRON last tried auto-reconciliation
- `action_save_close()` / `action_save_new()` — Navigation actions in the reconciliation widget

### Auto-Reconciliation CRON (`_cron_try_auto_reconcile_statement_lines`)
**Algorithm**:
1. Find companies with at least one reconcile model having `auto_reconcile=True`
2. Search unreconciled statement lines created in the last 3 months, ordered by `cron_last_check ASC NULLS FIRST` (prioritizes never-processed lines)
3. For each line:
   a. Create a transient `bank.rec.widget` with the statement line
   b. Run `_action_trigger_matching_rules()` — applies all reconcile models
   c. If state becomes `valid` AND `matching_rules_allow_auto_reconcile`:
      - Call `_action_validate()` to reconcile
      - Post a message on the move noting which model was used
   d. On `UserError`, log and continue
4. Write `cron_last_check = now()` on all processed lines
5. If batch_size reached and more lines exist, trigger another CRON run

**Gotcha**: The CRON uses `FOR UPDATE` lock on processed statement lines to prevent concurrent processing.

### Partner Retrieval (`_retrieve_partner`)
Priority order:
1. `st_line.partner_id` (if already set)
2. Bank account number matching (`sanitized_acc_number ilike`)
3. Partner name matching (`complete_name =ilike` then `ilike`, with company filtering)
4. Reconcile model partner mappings (`_get_partner_from_mapping`)

### Text Matching for Invoice Matching
`_get_st_line_strings_for_matching(allowed_fields)` collects text from:
- `payment_ref` (label)
- `narration` (notes, converted from HTML to plain text)
- `ref` (reference)

### Reconcile Model Rule Engine

#### `_apply_rules(st_line, partner)` — Main entry point
Iterates through non-button reconcile models (sorted by sequence):
1. Checks `_is_applicable_for()` — filters on journal, amount nature/range, partner, label/note/transaction_type regex
2. For `invoice_matching` type:
   - Gets rules map via `_get_invoice_matching_rules_map()` (extensible, priority-ordered)
   - Default rule at priority 10: `_get_invoice_matching_amls_candidates()`
   - Returns matched AMLs + auto_reconcile flag
3. For `writeoff_suggestion` type:
   - Returns the model itself as a write-off

#### Invoice Matching Candidates (`_get_invoice_matching_amls_candidates`)
**Domain construction** (`_get_invoice_matching_amls_domain`):
- Default AML matching domain from statement line
- Filtered by amount sign (positive/negative)
- Optional: same currency filter
- Optional: partner filter
- Optional: past months limit

**Token-based matching** (SQL):
1. Extract numerical and exact tokens from statement line text
2. Build CTE with AML names, move names, move refs
3. Match tokens against CTE using `UNNEST(REGEXP_SPLIT_TO_ARRAY(...))` for numerical matching
4. Rank by `nb_match DESC`, then by date order (configurable: `new_first` or `old_first`)
5. If tokens found candidates -> `allow_auto_reconcile = True`

**Fallback** (no text match):
- If no partner: match by exact `amount_residual` or `amount_residual_currency`
- If partner: return all matching AMLs sorted by date

#### Result Validation (`_get_invoice_matching_amls_result`)
- Attempts batch matching: perfect (sum = 0) or partial
- Tries early payment discount matching first (only perfect match accepted)
- Calls `_check_rule_propositions()` for tolerance checks:
  - `payment_tolerance_param = 0` -> reject if not perfect
  - `payment_tolerance_type = 'fixed_amount'` -> check residual <= tolerance
  - `payment_tolerance_type = 'percentage'` -> check residual % <= tolerance
  - Returns status set: `{rejected}`, `{allow_write_off, allow_auto_reconcile}`, or `{allow_auto_reconcile}`

#### Write-off Application (`_apply_lines_for_bank_widget`)
For each reconcile model line, calls `_apply_in_bank_widget()` which handles:
- `amount_type = 'percentage_st_line'` — Percentage of statement line amount
- `amount_type = 'regex'` — Extract amount from payment_ref using regex
- `amount_type = 'percentage'` — Percentage of residual
- `amount_type = 'fixed'` — Fixed amount

---

## 4. Fiscal Year Operations

### `account.fiscal.year` Model
- Fields: `name`, `date_from`, `date_to`, `company_id`
- **Constraint**: No overlapping fiscal years for the same company. No fiscal years on child companies.
- Simple model — the heavy lifting is in `res.company.compute_fiscalyear_dates()`

### `res.company.compute_fiscalyear_dates(current_date)`
1. Search for a fiscal year record containing the date -> return it
2. Otherwise compute from `fiscalyear_last_day` / `fiscalyear_last_month`
3. Handles gaps between fiscal year records (adjusts date_from/date_to)

### Lock Date Wizard (`account.change.lock.date`)
**Lock date types** (all on res.company):
| Lock Type | Field | Scope |
|---|---|---|
| Lock Everything | `fiscalyear_lock_date` | All entries up to date |
| Lock Tax Return | `tax_lock_date` | Entries with taxes up to date |
| Lock Sales | `sale_lock_date` | Sale journal entries up to date |
| Lock Purchases | `purchase_lock_date` | Purchase journal entries up to date |
| Hard Lock | `hard_lock_date` | **Irreversible**, no exceptions allowed |

**Exception system** (`account.lock_exception` — defined in base `account`):
- Exceptions can be "for me" (current user) or "for everyone"
- Duration: 5min, 15min, 1h, 24h, or forever
- Requires a reason
- Exception for everyone + forever = just changing the lock date

**Gotchas**:
- Cannot set lock dates in the future
- Cannot decrease hard_lock_date
- Warns about draft entries in locked period
- Only `group_account_manager` can change lock dates
- `_create_default_report_external_values()` hook for tax/fiscal lock changes (overridden by account_reports)

---

## 5. Deferred Entries (Accruals)

### Fields on `account.move.line`
- `deferred_start_date` / `deferred_end_date` — Define the deferral period
- `has_deferred_moves` (computed) — Whether the parent move has deferred entries
- `has_abnormal_deferred_dates` (computed) — Warns if dates might cause odd amounts (e.g., Jan 1 -> Jan 1 next year instead of Dec 31)

### Fields on `account.move`
- `deferred_move_ids` (M2M) — Generated deferred entries
- `deferred_original_move_ids` (M2M, inverse) — Original invoices
- `deferred_entry_type` (computed: expense/revenue)
- `payment_state_before_switch` — Stores previous state during invoicing switch

### Company Configuration
- `generate_deferred_expense_entries_method`: `'on_validation'` or `'manual'` (Manually & Grouped)
- `deferred_expense_amount_computation_method`: `'day'`, `'month'`, or `'full_months'`
- Same pair for revenue
- `deferred_expense_journal_id` / `deferred_expense_account_id`
- `deferred_revenue_journal_id` / `deferred_revenue_account_id`

### Deferred Entry Generation (`_generate_deferred_entries()`)
Called during `_post()` when method is `'on_validation'`:
1. For each line with `deferred_start_date` and `deferred_end_date`:
   - Skip if start, end, and accounting date are all in the same month
   - Compute periods via `_get_deferred_periods()` (list of (start, end) per month)
2. Create a **full deferral move** (reversal): debit original account, credit deferred account
3. Create **period deferral moves**: for each month period, debit deferred account, credit original account
4. Last period gets forced balance to avoid rounding errors
5. All moves posted with `auto_post='at_date'`

### Amount Computation Methods
- **`day`**: `balance / total_days * period_days`
- **`month`**: Uses `_get_deferred_diff_dates()` which normalizes months to 30 days (Feb=Mar=Apr)
- **`full_months`**: Like `month` but resets all dates to the 1st of the month first

### Deferred-aware Tax Computation
`account.tax` is extended to include `deferred_start_date` / `deferred_end_date` in:
- `_prepare_base_line_for_taxes_computation()`
- `_prepare_base_line_grouping_key()`
- `_prepare_base_line_tax_repartition_grouping_key()` — Only propagates deferred dates to tax lines when the account is deferred-compatible AND `use_in_tax_closing=False`

### Compatible Accounts
`_has_deferred_compatible_account()`:
- Purchase documents: `expense`, `expense_depreciation`, `expense_direct_cost`
- Sale documents: `income`, `income_other`

### Gotchas
- Resetting a deferred invoice to draft will reverse (not unlink) deferred moves if audit trail is active
- Cannot change `account_id` on a line that already has deferred moves
- Grouped deferral (`manual` method) prevents reset-to-draft of individual invoices

---

## 6. Payment Follow-ups

Payment follow-ups are NOT in `account_accountant` directly — they live in `account_followup` which depends on `account_reports` (which depends on `accountant` -> `account_accountant`).

The `account_accountant` module does NOT add follow-up fields or logic. It provides the foundation (bank reconciliation, lock dates, etc.) that the follow-up system builds upon.

See section 7 for `account_followup` overview.

---

## 7. Related Enterprise Modules

### `accountant` (depends: account_accountant)
The actual "Accounting" application (`application=True`). Mostly a thin wrapper:
- Adds security groups for full accounting
- Provides the application menu entry
- Depends on `account_accountant` for all actual functionality

### `account_reports` (depends: accountant)
**What it adds**: All financial reports — Balance Sheet, P&L, Cash Flow, Trial Balance, General Ledger, Partner Ledger, Aged Partner Balance, Tax Report, Multicurrency Revaluation, Deferred Reports, Journal Report, Customer Statement, Follow-up Report.
- Core engine: `account.report` model with dynamic columns, grouping, filters
- PDF export with dedicated asset pipeline
- Bank reconciliation report
- Tax closing integration (automatic `tax_lock_date` updates)

### `account_asset` (depends: account_reports)
**What it adds**: Asset management — depreciation schedules, automatic journal entries.
- `account.asset` model with configurable depreciation methods (linear, degressive, etc.)
- Asset groups for common configurations
- Linked to accounts via `account.account` extensions
- Depreciation report in accounting reports

### `account_budget` (depends: accountant, purchase)
**What it adds**: Budget management with analytic accounts.
- `budget.line` model for budget allocations by analytic account/period
- Budget vs actual comparison
- Split wizard for distributing budgets across periods
- Purchase order budget integration

### `account_followup` (depends: mail, sms, account_reports)
**What it adds**: Automated payment follow-up system.
- Multi-level follow-up definitions (`account.followup.line`)
- Actions per level: email, letter (PDF), SMS, activity
- CRON-based automatic sending
- Manual reminder wizard
- Missing information wizard (when partner data is incomplete)
- Follow-up report integrated into account_reports

### `account_batch_payment` (depends: account)
**What it adds**: Group payments into batches for bank deposits.
- `account.batch.payment` model
- Batch types: inbound (deposit) and outbound
- PDF batch report
- Integration with journal views
- Reconciliation: select a batch to reconcile all its payments at once

---

## 8. Security & Groups

### Groups Modified by account_accountant

```xml
<!-- Renames group_account_basic to "Invoicing & Banks" -->
<record id="account.group_account_basic" model="res.groups">
    <field name="name">Invoicing &amp; Banks</field>
    <!-- Removes implied group_account_readonly -->
    <field name="implied_ids" eval="[(3, ref('account.group_account_readonly'))]"/>
</record>

<!-- group_account_manager now implies group_account_basic -->
<record id="account.group_account_manager" model="res.groups">
    <field name="implied_ids" eval="[(4, ref('account.group_account_basic'))]"/>
</record>

<!-- New hidden group for custom fiscal years -->
<record id="group_fiscal_year" model="res.groups"/>
```

### Access Rights (ir.model.access.csv)

| Model | Group | R | W | C | D |
|---|---|---|---|---|---|
| account.change.lock.date | group_account_manager | Y | Y | Y | N |
| account.secure.entries.wizard | group_account_user | Y | Y | Y | N |
| account.auto.reconcile.wizard | group_account_user | Y | Y | Y | N |
| account.reconcile.wizard | group_account_user | Y | Y | Y | N |
| account.fiscal.year | group_account_readonly | Y | N | N | N |
| account.fiscal.year | group_account_basic | Y | N | N | N |
| account.fiscal.year | group_account_manager | Y | Y | Y | Y |
| bank.rec.widget | group_account_user | Y | Y | Y | Y |
| bank.rec.widget.line | group_account_user | Y | Y | Y | Y |

### Menu Visibility
`ir.ui.menu._visible_menu_ids()` is overridden to hide these menus from non-accountants (users without `group_account_readonly`):
- `account_accountant.account_tag_menu`
- `account_accountant.menu_account_group`
- `account_reports.menu_action_account_report_multicurrency_revaluation`

---

## 9. Key Methods & Hooks

### Post-init Hook
- `_account_accountant_post_init` — Called on module install
- `uninstall_hook` — Called on module uninstall

### Methods Developers Commonly Override

#### Bank Reconciliation
- **`bank.rec.widget._action_trigger_matching_rules()`** (`account_accountant/models/bank_rec_widget.py:1185`, `BankRecWidget._action_trigger_matching_rules`) — Override to add custom matching logic before/after standard rules
- **`account.reconcile.model._get_invoice_matching_rules_map()`** (`account_accountant/models/account_reconcile_model.py:356`, `AccountReconcileModel._get_invoice_matching_rules_map`) — Add custom matching rules at specific priorities (default rule is at priority 10)
- **`account.reconcile.model._is_applicable_for(st_line, partner)`** (`account_accountant/models/account_reconcile_model.py:96`, `AccountReconcileModel._is_applicable_for`) — Add custom applicability conditions
- **`account.bank.statement.line._retrieve_partner()`** (`account_accountant/models/account_bank_statement.py:169`, `AccountBankStatementLine._retrieve_partner`) — Add custom partner detection logic
- **`bank.rec.widget._do_amounts_apply_for_early_payment()`** (`bank_rec_widget.py`) — Customize early payment discount threshold
- **`bank.rec.widget._validation_lines_vals()`** (`bank_rec_widget.py`) — Modify line values before reconciliation is committed
- **`bank.rec.widget._get_key_mapping_aml_and_exchange_diff(line)`** (`account_accountant/models/bank_rec_widget.py:808`, `BankRecWidget._get_key_mapping_aml_and_exchange_diff`) — Customize exchange diff grouping

#### Deferred Entries
- **`account.move._get_deferred_entries_method()`** — Override to customize when deferrals are generated
- **`account.move._get_deferred_period_amount(method, ...)`** — Override for custom amount computation
- **`account.move.line._get_deferred_periods()`** — Override to customize period splitting
- **`account.move.line._has_deferred_compatible_account()`** — Override to allow/disallow deferral on specific account types

#### Lock Dates
- **`account.change.lock.date._create_default_report_external_values(lock_date_field)`** — Hook for creating report external values when lock dates change (used by account_reports)
- **`account.change.lock.date._prepare_lock_date_values()`** — Override to add custom validation

#### Fiscal Year
- **`res.company.compute_fiscalyear_dates(current_date)`** — Override for custom fiscal year computation

#### Predictive Coding
- **`account.move.line._predict_account()`** — Override to customize account prediction
- **`account.move.line._predict_taxes()`** — Override to customize tax prediction
- **`account.move.line._build_predictive_query(move_id)`** — Override to customize the base query for predictions

### Important Context Keys
- `skip_statement_line_cron_trigger` — Prevents triggering auto-reconcile CRON on post
- `skip_readonly_check` — Bypasses readonly field checks (used in reconciliation validation)
- `force_delete` — Allows clearing move lines during reconciliation
- `skip_account_move_synchronization` — Prevents sync between move and statement line
- `no_exchange_difference_no_recursive` — Prevents recursive exchange diff creation during reconciliation
- `disable_onchange_name_predictive` — Prevents predictive coding during EDI creation
- `preferred_aml_value` / `preferred_aml_currency_id` — Custom ordering for AML search (exact amount match first)
- `move_reverse_cancel` — Preserves deferred dates during reversal copy

### Utility: `ir.model._is_valid_for_model_selector()`
Excludes `bank.rec.widget` and `bank.rec.widget.line` from model selectors since they have no real database table.
