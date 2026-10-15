# Account Reports v19 Changes — vs Odoo 18.0 Enterprise

> Comparison: `odoo_enterprise/18/account_reports/` vs `odoo_enterprise/19/account_reports/`

## Major New Feature: Account Returns System

The biggest v19 addition is a complete **Tax Return / Audit workflow system** via new models:

### New Models

| Model | Description |
|-------|-------------|
| `account.return.type` | Template for return types (tax return, audit). Links to `account.report`. Has configurable periodicity, workflow states, deadlines. |
| `account.return` | Individual return instances. State machine with workflow: New → Reviewed → Submitted → Paid. Supports attachments, checks, payment tracking. |
| `account.return.check` | Checklist items within a return (review checkpoints). |
| `account.return.check.template` | Reusable templates for checks. |
| `account.audit.account.status` | Per-account audit status within an audit return (todo/reviewed/supervised/anomaly). |

### Return Type Workflows

Four configurable state machines:
- `generic_state_review` — Review only
- `generic_state_review_submit` — Review, then Submit
- `generic_state_tax_report` — Review, Submit, Pay (full tax return flow)
- `generic_state_only_pay` — Pay only

### Return Periodicity

`account.return.type` has company-dependent periodicity fields:
- `deadline_periodicity`: monthly / 2_months / trimester / 4_months / semester / year / fiscalyear
- `deadline_start_date`: company-dependent start date
- `deadline_days_delay`: deadline offset in days

Auto-generation creates return instances automatically based on periodicity.

### New Files in v19

| File | Purpose |
|------|---------|
| `models/account_return.py` | **3000+ lines** — `account.return.type`, `account.return`, `account.return.check`, `account.return.check.template` |
| `models/account_audit_account_status.py` | Account audit status per return |
| `models/kpi_provider.py` | KPI dashboard integration for returns |
| `models/ir_module.py` | Module-level hooks for returns |
| `models/ir_ui_menu.py` | Menu management for returns |
| `models/mail_message.py` | Mail message customization |
| `models/mail_scheduled_message.py` | Scheduled message support |
| `models/mail_tracking_value.py` | Tracking value customization |
| `models/res_users.py` | User-level settings for returns |
| `wizard/return_creation_wizard.xml` | Return creation wizard |
| `wizard/return_submission_wizard.xml` | Return submission wizard |
| `wizard/return_generic_payment_wizard.xml` | Return payment wizard |
| `data/annual_statements.xml` | Annual statement composite report data |
| `data/account_return_data.xml` | Default return type data |
| `data/account_return_check_template.xml` | Default check templates |
| `security/account_reports_security.xml` | New security rules for returns |

### Removed Files in v19

| File | Status |
|------|--------|
| `models/mail_activity.py` | Replaced by return workflow |
| `models/mail_activity_type.py` | Removed |
| `data/mail_activity_type_data.xml` | Removed |
| `views/mail_activity_views.xml` | Removed |
| `wizard/mail_activity_schedule_views.xml` | Removed |
| `views/account_activity.xml` | Removed |

## Dependency Change

- **v18**: `depends: ['accountant']`
- **v19**: `depends: ['account_accountant']`

The `accountant` module was renamed/restructured to `account_accountant` in v19.

## Report Engine Changes

### New Options Initializers in v19

| Method | Purpose |
|--------|---------|
| `_init_options_audit` | Account audit status integration |
| `_init_options_return_periodicity` | Tax return periodicity for date filter |
| `_init_options_consolidation` | Multi-company consolidation groupby |
| `_init_options_loading_call` | JS loading state tracking |
| `_init_options_filters` | Centralized filter visibility flags |
| `_init_options_user_groups` | Cache user group memberships in options |
| `_init_options_export` | Report title for exports |

### Changed Options Behavior

**`_init_options_readonly_query`**: v18 also checked budget selection; v19 only checks monocurrency:
```python
# v18:
options['readonly_query'] = (
    options['currency_table']['type'] == 'monocurrency'
    and not any(budget_opt['selected'] for budget_opt in options.get('budgets', []))
)

# v19:
options['readonly_query'] = options['currency_table']['type'] == 'monocurrency'
```

### New `_init_options_filters` (v19)

Centralizes filter visibility into a single `options['filters']` dict:
```python
options['filters'] = {
    'show_all': self.filter_unfold_all,
    'show_analytic': ...,
    'show_analytic_groupby': ...,
    'show_analytic_plan_groupby': ...,
    'show_draft': self.filter_show_draft,
    'show_hierarchy': ...,
    'show_period_comparison': self.filter_period_comparison,
    'show_totals': ...,
    'show_unreconciled': self.filter_unreconciled,
    'show_hide_0_lines': self.filter_hide_0_lines,
}
```

### New `_init_options_user_groups` (v19)

Caches user group memberships to avoid repeated `has_group()` calls on the client:
```python
options['user_groups'] = {
    'analytic_accounting': ...,
    'account_readonly': ...,
    'account_user': ...,
}
```

### Consolidation Option (v19)

New `_init_options_consolidation` adds a consolidation toggle when:
- Multiple companies selected
- At least one line has `account_id` in its user_groupby

This enables consolidated financial reporting across companies.

## Model Changes

### AccountReport (v19)

- **New field**: `return_type_ids` (O2M to `account.return.type`) — links reports to return types
- **New field**: `allow_account_audit_status_on_lines` (Boolean, computed) — enables audit status tracking
- **Removed field**: `annotations_ids` (O2M) — annotations now use `mail.message` instead of custom model
- **Changed field**: `AccountReportAnnotation.message_id` (M2O to `mail.message`) replaces the old text/date/report_id fields

### AccountReportAnnotation Rewrite

**v18**: Self-contained model with `report_id`, `line_id` (Char), `text`, `date`, `fiscal_position_id`

**v19**: Thin wrapper around `mail.message`:
```python
class AccountReportAnnotation(models.Model):
    _name = 'account.report.annotation'
    message_id = fields.Many2one('mail.message', required=True)
    date = fields.Date(required=True)
```

**API Break**: Any code directly accessing `annotation.text`, `annotation.line_id`, or `annotation.report_id` will break in v19. The annotation content is now in the linked `mail.message`.

### Annual Statements (v19)

New composite report concept: `annual_statements` is a root composite report that automatically collects localized report variants as sections. The `create()` method on `account.report` now auto-links new reports to annual statements via `_asr_sections_domain()` and `_link_annual_statements()`.

## Expression Engine

The five core engines (`domain`, `tax_tags`, `account_codes`, `aggregation`, `external`, `custom`) remain structurally identical. Method signatures and return formats are unchanged. The expression evaluation pipeline in `_compute_expression_totals_for_each_column_group` is the same.

## Security Changes

- **v19 adds**: `security/account_reports_security.xml` (new file for return-specific security rules)
- Return models have their own access rights (not just the existing report ACLs)

## Asset Changes

- **v18**: `'web.qunit_suite_tests'` and legacy test assets included
- **v19**: Legacy test assets removed; `views/` path added for OWL components (`static/src/views/**/*`)
- **v19 adds**: `account_return.scss` stylesheet

## Migration Notes for Custom Code

### Breaking Changes

1. **Annotations API completely changed** — if you read/write annotations, migrate to mail.message-based approach
2. **`accountant` dependency renamed to `account_accountant`** — update manifests
3. **Mail activity system replaced by returns** — custom follow-up flows using `mail.activity` on reports need rewrite
4. **New options keys** — code reading `options` dict should handle new keys: `audit`, `return_periodicity`, `consolidation`, `filters`, `user_groups`, `loading_call_number`

### Non-Breaking Additions

1. Custom handlers still work identically
2. Expression engines unchanged
3. `_get_lines()` flow unchanged
4. Export methods (PDF/XLSX) unchanged
5. Line ID system unchanged
6. All existing option initializers preserved (new ones added alongside)

### Recommended Migration Path

1. Update `__manifest__.py` dependency from `accountant` to `account_accountant`
2. If using annotations: refactor to use `mail.message` as backing store
3. If using mail activities on reports: migrate to `account.return` workflow
4. Test that `options` dict handling is robust against new keys
5. No changes needed for expression formulas or custom engines
