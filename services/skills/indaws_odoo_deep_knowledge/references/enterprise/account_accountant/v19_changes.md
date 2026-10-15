# Account Accountant v19 Changes (vs v18)

> Generated 2026-04-01 by comparing `odoo_enterprise/18/account_accountant` vs `odoo_enterprise/19/account_accountant`

---

## Breaking Changes (HIGH)

### 1. `bank.rec.widget` and `bank.rec.widget.line` models REMOVED
- **Files removed**: `models/bank_rec_widget.py` (1791 lines), `models/bank_rec_widget_line.py` (503 lines)
- **Model removed**: `ir_model.py` (`_is_valid_for_model_selector` exclusion no longer needed)
- **Security removed**: `access_bank_rec_widget` and `access_bank_rec_widget_line` ACL rows deleted from `ir.model.access.csv`
- **Impact**: Any module inheriting `bank.rec.widget` or `bank.rec.widget.line` will CRASH on v19. The reconciliation widget logic has been completely rewritten and moved to `account.bank.statement.line` (see below).
- **Migration**: All `_inherit = 'bank.rec.widget'` / `bank.rec.widget.line` must be refactored to work with the new `account.bank.statement.line` methods.

### 2. Bank reconciliation logic moved to `account.bank.statement.line`
- `account_bank_statement.py` expanded from ~240 lines to ~1868 lines. It now contains ALL reconciliation logic that was previously in `bank.rec.widget`:
  - `_try_auto_reconcile_statement_lines()` - full auto-reconciliation engine with SQL-based matching
  - `set_line_bank_statement_line()`, `delete_reconciled_line()`, `edit_reconcile_line()` - manual reconciliation operations
  - `_reconcile_payments()` - payment reconciliation
  - `_apply_early_payment_discount()` - EPD handling
  - `_recompute_tax_lines()`, `_create_tax_lines()`, `_edit_tax_lines()`, `_delete_tax_lines()` - tax line management
  - `_lines_get_account_balance_exchange_diff()` - exchange difference calculations
  - `_retrieve_partner()` - partner detection from statement line data
  - `_create_automatic_reconciliation_model()` - auto-creation of reconciliation rules
  - `_create_account_model_fee()` - fee model auto-creation
  - `create_document_from_attachment()` - document creation from attachments
  - `action_unreconcile_entry()` - un-reconciliation
- `AccountBankStatementLine` now also inherits from `mail.thread.main.attachment` (was only `account.bank.statement.line` before)

### 3. `_apply_rules()` method REMOVED from `account.reconcile.model`
- The entire `_apply_rules()` method and all its helper methods are gone:
  - `_apply_rules()` - main entry point for rule-based reconciliation
  - `_is_applicable_for()` - rule applicability check
  - `_get_invoice_matching_amls_domain()` - AML domain building
  - `_get_st_line_text_values_for_matching()` - text extraction
  - `_get_invoice_matching_st_line_tokens()` - token parsing
  - `_get_invoice_matching_amls_candidates()` - candidate fetching
  - `_get_invoice_matching_rules_map()` - rules map
  - `_get_partner_from_mapping()` - partner mapping
  - `_get_invoice_matching_amls_result()` - result building
  - `_check_rule_propositions()` - proposition validation
  - `run_auto_reconciliation()` - auto-reconciliation trigger
- **Replaced by**: SQL-based matching in `account.bank.statement.line._try_auto_reconcile_statement_lines()` and `account.reconcile.model._apply_reconcile_models()`
- **Impact**: Any module calling `_apply_rules()` or overriding any of these methods will break.

### 4. `_apply_lines_for_bank_widget()` signature changed
- **v18**: `_apply_lines_for_bank_widget(self, residual_amount_currency, partner, st_line)`
- **v19**: `_apply_lines_for_bank_widget(self, residual_amount_currency, residual_balance, partner, st_line)`
- New `residual_balance` parameter (company currency balance) is now required.

### 5. `_apply_in_bank_widget()` and `_apply_in_manual_widget()` signatures changed (reconcile model line)
- **v18**: `_apply_in_bank_widget(self, residual_amount_currency, partner, st_line)`
- **v19**: `_apply_in_bank_widget(self, residual_amount_currency, residual_balance, partner, st_line)`
- **v18**: `_apply_in_manual_widget(self, residual_amount_currency, partner, currency)`
- **v19**: `_apply_in_manual_widget(self, residual_amount_currency, residual_balance, partner, st_line)`
- Both methods now require `residual_balance` and `_apply_in_manual_widget` takes `st_line` instead of `currency`.

### 6. `action_post()` override REMOVED from `account.move`
- v18 had an `action_post()` override that triggered the bank statement line CRON. This is removed in v19.
- The CRON trigger mechanism has been rearchitected.

### 7. `_read_group_groupby()` signature changed
- **v18**: `_read_group_groupby(self, groupby_spec, query)`
- **v19**: `_read_group_groupby(self, alias, groupby_spec, query)`
- New `alias` parameter added (ORM-level change).

### 8. `_predicted_field()` signature changed to `@api.model`
- **v18**: Instance method `_predicted_field(self, name, partner_id, field, query=None, additional_queries=None)`
- **v19**: Class method `_predicted_field(self, move_id, name, partner_id, field, query=None, additional_queries=None)`
- Now `@api.model` with explicit `move_id` parameter instead of reading from `self.move_id` / context.

### 9. Security group restructuring
- `res.groups` for `account.group_account_basic` now uses `privilege_id` instead of `category_id`
- `group_fiscal_year` no longer has `category_id` set to hidden
- Module category description block removed from security XML

### 10. `account_reconcile_model_views.xml` and `reconcile_model_wizard.xml` REMOVED
- View file `views/account_reconcile_model_views.xml` removed from v19 manifest
- Wizard file `wizard/reconcile_model_wizard.xml` removed from v19 manifest

---

## Behavioral Changes (MEDIUM)

### 11. Deferred entries now support miscellaneous journal entries
- `deferred_entry_type` field gains new selection value `'misc'` (Deferred Miscellaneous)
- `_generate_deferred_entries()` now iterates over both `'expense'` and `'revenue'` types separately, allowing mixed entries
- `_get_deferred_entries_method()` now handles `is_entry()` moves by checking `internal_group` of accounts
- `_has_deferred_compatible_account()` now uses `internal_group` instead of `account_type` and supports `is_entry()` moves
- Previously raised `UserError` for miscellaneous journal entries; now processes them

### 12. Reconciliation models auto-apply on create/write/archive
- New `write()`, `create()`, and `action_archive()` overrides on `account.reconcile.model`
- When a reconciliation model is created, modified, or archived, it automatically re-evaluates all unreconciled statement lines
- New field `created_automatically` (Boolean) to track auto-created models

### 13. CRON auto-reconciliation enhanced
- `_cron_try_auto_reconcile_statement_lines()` gains `company_id` parameter for single-company processing
- New time limit handling with fallback to 180 seconds
- Supports commit between batches (not in test mode)
- Domain simplified: only looks for `cron_last_check = False` lines (was using 3-month date filter)
- New constants: `AUTO_STATEMENT_PROCESSING_NUMBER = 80`, `AUTO_STATEMENT_PROCESSING_BATCH_SIZE = 100`

### 14. Reconcile model matching now SQL-based
- `get_available_reconcile_model_per_statement_line()` - new SQL-based method for bulk model matching
- `_apply_reconcile_models()` - new SQL-based method for applying models to statement lines in batch
- Both use `LATERAL JOIN` for efficient per-statement-line filtering
- `match_label` filtering now also searches `transaction_details` field (not just `payment_ref`)

### 15. `_compute_payments_widget_to_reconcile_info()` - NEW override
- Shows outstanding bank statement line credits/debits in the invoice payment widget
- Finds unreconciled statement lines with matching partner and suspense account balance

### 16. `js_assign_outstanding_line()` and `js_remove_outstanding_partial()` - NEW overrides
- `js_assign_outstanding_line()`: When assigning an outstanding line from a bank statement, routes through `set_line_bank_statement_line()`
- `js_remove_outstanding_partial()`: When removing a partial from a statement line, uses `delete_reconciled_line()`

### 17. Account domain changes: `deprecated` filter removed
- `account_reconcile_wizard.py`: Domain `('deprecated', '=', False)` removed from account field
- `account_auto_reconcile_wizard.py`: Same removal
- `_predict_account()`: Same removal
- Odoo v19 likely deprecated the `deprecated` field on accounts

### 18. `_where_calc` replaced with `_search` + `bypass_access`
- Multiple occurrences in predictive billing: `_where_calc([...])` -> `_search([...], bypass_access=True)`
- ORM-level change for proper access right handling

### 19. Lock date wizard enhancements
- `exception_needed` field REMOVED (marked as TODO remove in v18)
- `_compute_exception_needed()` method REMOVED
- New `show_posted_tax_closing_warning` field and `_compute_show_posted_tax_closing_warning()` method
- New `action_show_posted_tax_closing_in_locked_period()` action
- Domain building migrated from `expression.OR` to `Domain.OR`

### 20. Tax handling in reconcile model lines
- `_prepare_aml_vals()` now falls back to `self.account_id.tax_ids` when `self.tax_ids` is empty
- `tax_ids` key only added to vals dict when taxes actually exist (conditional instead of always set)

### 21. Regex amount extraction enhanced
- New `_get_amount_currency_by_regex()` method on `account.reconcile.model.line`
- Searches across `payment_ref`, `transaction_details`, and `narration` (v18 only searched `payment_ref`)
- Supports two-group regex (integer + decimal parts)
- Raises `RedirectWarning` with link to reconcile model on bad regex

### 22. `_compute_reco_model_autocomplete_ids()` filter enhanced (reconcile wizard)
- v18: Simple filter `rule_type = writeoff_button` + `counterpart_type not in (sale, purchase)`
- v19: Filters by matching partners and matching amounts (lower/greater/between), uses `_search` instead of `_where_calc`

### 23. `display_type` filter updated
- `account_auto_reconcile_wizard.py`: `'line_subsection'` added to `display_type` exclusion list

### 24. `ir.ui.menu` visibility refactored
- New `_get_account_readonly_menu_ids()` method for extensibility
- `account_reports.menu_action_account_report_multicurrency_revaluation` removed from this module's hidden list
- Uses `_xmlid_to_res_id` instead of `env.ref` for safer lookup

### 25. Statement line renamed from "Bank Reconciliation" to "Bank Matching"
- `_action_open_bank_reconciliation_widget()` default name changed: `"Bank Reconciliation"` -> `"Bank Matching"`
- Context now includes `bank_statements_source` and `auto_statement_processing` flags

### 26. `full_amount_switch_html` - NEW computed field on `account.move.line`
- Shows inline HTML with links to switch between full/partial payment reconciliation
- Provides contextual messages for invoices vs other documents

---

## New Features (LOW)

### 27. New model: `analytic.mixin` extension
- **File**: `models/analytic_mixin.py` (NEW)
- Adds `_read_group_for_accrual()` method to `analytic.mixin` for computing non-stored computed field aggregates
- Used by accrual views for `purchase.order.line` and `sale.order.line`
- Fields: `qty_invoiced_at_date`, `amount_to_invoice_at_date`

### 28. New `ir.config_parameter`: `account_accountant.bank_rec_payment_tolerance`
- **File**: `data/ir_config_parameters.xml` (NEW)
- Default value: `0.00`
- Controls payment tolerance in bank reconciliation

### 29. New setting: `module_account_auto_transfer`
- **File**: `res_config_settings.py`
- New Boolean field `module_account_auto_transfer` with label "Enable Auto Transfer"

### 30. Bank statement auto-generates PDF attachment on create
- `AccountBankStatement.create()` now auto-generates PDF attachment for complete statements without PDF
- Skippable via context `skip_pdf_attachment_generation`

### 31. New actions on bank statements
- `action_open_journal_invalid_statements()` - opens invalid statements view for a journal
- `action_button_draft()` - reset statement line's move to draft
- `bank_statement_attachment_ids` - computed field for statement-level attachments on lines

### 32. Journal dashboard enhancements
- `get_total_journal_amount()` - new method returning formatted balance + invalid statements flag
- `open_invalid_statements_action()` override - routes to bank reco widget with invalid statement filter
- `open_action()` simplified - no longer passes `extra_domain` with account filter for bank/cash/credit journals

### 33. Chart template auto-installs SEPA module
- `_post_load_data()` now auto-installs `account_iso20022` module for SEPA zone countries

### 34. Predictive billing: deductible amount prediction
- New `_predict_deductible_amount()` method on `account.move.line`
- Requires `account.group_partial_purchase_deductibility` group
- Called during `_onchange_name_predictive()`

### 35. Auto-reconciliation creates rules automatically
- `_create_automatic_reconciliation_model()` - creates reconciliation models from patterns
- `_check_and_create_reconciliation_rule()` - checks if a rule should be created
- `_prepare_reconciliation_rule_data()` - prepares rule data from statement lines
- `_create_account_model_fee()` - creates fee reconciliation models
- `_get_common_substring()` - finds common label patterns for rule creation

### 36. Payment reconciliation from bank widget
- `_get_amls_for_payment_without_move()` - NEW on `account.payment`: gets AMLs for draft payments without moves
- `_get_aml_amount_in_payment_currency()` - NEW: currency conversion helper
- `_get_amls_for_reconciliation()` - NEW: comprehensive AML fetching for bank reconciliation context
- Handles early payment discounts, payment cancellation/modification during reconciliation

### 37. `_compute_attachment` optimized
- Uses `_read_group` with grouped aggregation instead of per-record `search()`
- New helper `_get_attachment_by_record()`

### 38. `_onchange_deferred_start_date` / `_onchange_deferred_end_date` improvements
- Now also triggered by `account_id` changes
- Auto-sets `deferred_end_date = deferred_start_date` when end date is missing

### 39. Deferred date validation relaxed
- `has_abnormal_deferred_dates` now allows `deferred_start_date == deferred_end_date` (was flagged as abnormal before)

### 40. `_compute_has_abnormal_deferred_dates` fix
- Added check `deferred_start_date != deferred_end_date` before computing fractional month check

---

## Migration Checklist

### Must Fix (will crash)
- [ ] Remove all `_inherit = 'bank.rec.widget'` and `_inherit = 'bank.rec.widget.line'` references
- [ ] Remove all `access_bank_rec_widget*` ACL entries
- [ ] Update calls to `_apply_rules()` -> use new SQL-based matching or `_apply_reconcile_models()`
- [ ] Update `_apply_lines_for_bank_widget()` calls: add `residual_balance` parameter
- [ ] Update `_apply_in_bank_widget()` calls: add `residual_balance` parameter
- [ ] Update `_apply_in_manual_widget()` calls: change `currency` to `st_line`, add `residual_balance`
- [ ] Update `_read_group_groupby()` overrides: add `alias` parameter
- [ ] Update `_predicted_field()` calls: add `move_id` as first argument, remove context hack
- [ ] Remove references to `account_reconcile_model_views.xml` and `reconcile_model_wizard.xml`

### Should Fix (behavioral changes)
- [ ] Review any `action_post()` override that expected CRON trigger from `account.move`
- [ ] Review deferred entry logic if module handles miscellaneous entries
- [ ] Update `deprecated` account domain filters (field likely removed)
- [ ] Replace `_where_calc()` with `_search(bypass_access=True)` where applicable
- [ ] Review `exception_needed` field usage in lock date wizard customizations
- [ ] Update `display_type` filters to include `'line_subsection'`
- [ ] Test reconciliation model auto-apply behavior (may cause unexpected side effects)

### Nice to Have (new features to leverage)
- [ ] Consider using `bank_rec_payment_tolerance` config parameter
- [ ] Leverage `_compute_payments_widget_to_reconcile_info()` for bank statement credit display
- [ ] Use `_get_amount_currency_by_regex()` for enhanced regex matching
- [ ] Consider `module_account_auto_transfer` setting
- [ ] Use `analytic.mixin._read_group_for_accrual()` for accrual reporting

---

## File Inventory

### Files REMOVED in v19
| File | Content |
|------|---------|
| `models/bank_rec_widget.py` | `bank.rec.widget` model (1791 lines) |
| `models/bank_rec_widget_line.py` | `bank.rec.widget.line` model (503 lines) |
| `models/ir_model.py` | `_is_valid_for_model_selector` exclusion |
| `views/account_reconcile_model_views.xml` | Reconcile model views |
| `wizard/reconcile_model_wizard.xml` | Reconcile model wizard |

### Files ADDED in v19
| File | Content |
|------|---------|
| `models/analytic_mixin.py` | `analytic.mixin` accrual `_read_group` override |
| `data/ir_config_parameters.xml` | `bank_rec_payment_tolerance` parameter |

### Files with MAJOR changes
| File | Nature of change |
|------|-----------------|
| `models/account_bank_statement.py` | Expanded from ~240 to ~1868 lines; absorbs all bank rec widget logic |
| `models/account_reconcile_model.py` | Rewritten from ~560 to ~325 lines; Python matching replaced with SQL |
| `models/account_move.py` | Deferred entries rework, new payment widget, predictive billing changes |
| `models/account_payment.py` | New payment-without-move reconciliation methods |
| `models/account_reconcile_model_line.py` | Signature changes, regex enhancement |

### Manifest changes
- New data file: `data/ir_config_parameters.xml`
- New `author` field: `'Odoo S.A.'` added
- Removed: `views/account_reconcile_model_views.xml`, `wizard/reconcile_model_wizard.xml`
