# Account Asset v19 Changes — vs Odoo 18.0 Enterprise

## Dependency Change (BREAKING)

| Aspect | v18 | v19 |
|---|---|---|
| `depends` | `['account_reports']` | `['accountant']` |

The `accountant` module is the new v19 umbrella module for Enterprise accounting. If you have custom modules depending on `account_asset`, they may need to adjust their dependency chain.

## New Files in v19

### models/account_chart_template.py (NEW)
```python
class AccountChartTemplate(models.AbstractModel):
    _inherit = 'account.chart.template'

    @template(model='account.asset')
    def _get_account_asset(self, template_code):
        ...
```
Asset models can now be defined in chart template CSV data. The `post_init_hook` loads asset models and account configurations from chart template data on module install.

### models/account_return.py (NEW)
```python
class AccountReturn(models.Model):
    _inherit = 'account.return'

    def _check_suite_annual_closing(self, check_codes_to_ignore):
        ...
```
Adds a "Fixed Assets" check to the annual closing checklist. Warns if no depreciation was recorded for running assets in the period. Check code: `check_fixed_assets`.

### controller/asset_template_controller.py (NEW)
Full XLSX import template generator at `/web/binary/download_asset_template/<company_id>`. Creates a 2-sheet Excel file:
- Sheet 1: "Assets" with localized example rows (accounts from company's CoA)
- Sheet 2: "Instructions" with field descriptions and requirements

This is a significant UX improvement for bulk asset import.

### data/account_return_check_template.xml (NEW)
Template data for the annual closing check.

### post_init_hook (NEW)
Loads chart template data for `account.asset` and `account.account` (create_asset, asset_model_ids) on module installation. Ensures pre-defined asset models are created from CoA data.

## Model Changes in account.asset

### parent_id now indexed
```python
# v18
parent_id = fields.Many2one('account.asset', help="...")
# v19
parent_id = fields.Many2one('account.asset', index=True, help="...")
```
Performance improvement for queries on parent-child relationships.

### Deprecated filter removed from account domains
```python
# v18
account_depreciation_id domain: "[(...), ('deprecated', '=', False)]"
account_depreciation_expense_id domain: "[(...), ('deprecated', '=', False)]"
# v19
account_depreciation_id domain: "[(...)]"  # deprecated filter REMOVED
account_depreciation_expense_id domain: "[(...)]"
```
This applies to the asset model AND the wizard AND res.company gain/loss accounts. Deprecated accounts can now be selected. **Migration note**: no data migration needed, just behavioral change.

### _compute_related_purchase_value uses deductible_amount
```python
# v18
related_purchase_value = sum(asset.original_move_line_ids.mapped('balance'))
# v19
related_purchase_value = sum(line.balance * line.deductible_amount / 100 for line in asset.original_move_line_ids)
```
**BREAKING**: v19 introduces `deductible_amount` field on `account.move.line`. Asset original value now factors in deductibility percentage. This affects non-deductible tax calculations too.

### _compute_value_residual optimized with _read_group
```python
# v18: filters and maps posted moves per asset individually
posted_depreciation_moves = record.depreciation_move_ids.filtered(lambda mv: mv.state == 'posted')
sum(posted_depreciation_moves.mapped('depreciation_value'))

# v19: single SQL query via _read_group for all assets at once
grouped_moves = self.env['account.move']._read_group(
    domain=[('asset_id', 'in', self.ids), ('state', '=', 'posted')],
    groupby=['asset_id'],
    aggregates=['depreciation_value:sum'],
)
```
Significant performance improvement for batch operations.

### Non-deductible tax uses deductible_amount
```python
# v19 addition in _compute_non_deductible_tax_value:
converted_non_deductible_tax_value *= line.deductible_amount / 100
```

### self._context replaced with self.env.context
Three occurrences in `create()`, `open_entries()`, `open_asset()` changed from `self._context` to `self.env.context`. Style cleanup, no behavioral change.

### Disposal moves support storno accounting
```python
# v19 addition in _get_disposal_moves:
def get_line(name, asset, amount, account, is_sale):  # added is_sale param
    ...
    'is_storno': asset.company_id.account_storno and is_sale and (
        account not in (asset.company_id.gain_account_id, asset.company_id.loss_account_id)
    ),
```
When storno accounting is enabled on the company, sale disposal lines (except gain/loss accounts) are marked as storno entries. **BREAKING** for custom code overriding `_get_disposal_moves` or `get_line`.

## Changes in account.move

### asset_ids and related fields now compute_sudo=True
```python
# v18
asset_ids = fields.One2many('account.asset', ..., compute="_compute_asset_ids")
# v19
asset_ids = fields.One2many('account.asset', ..., compute="_compute_asset_ids", compute_sudo=True)
```
Same for `asset_id_display_name`, `count_asset`, `draft_asset_exists`. Fixes potential access right issues when invoice users view moves with assets they can't directly access.

### Depreciation value computation simplified
```python
# v18
move.line_ids[1].debit if asset.original_value > 0 else move.line_ids[1].credit
# v19
abs(move.line_ids[1].balance)
```
Simpler and handles edge cases better.

### self._cr replaced with self.env.cr
In `_compute_non_deductible_tax_value()`. Style cleanup.

## Changes in account.account

### create_asset becomes computed field
```python
# v18
create_asset = fields.Selection([...], required=True, default='no', tracking=True)

# v19
create_asset = fields.Selection(
    selection=[...],
    compute='_compute_create_asset', readonly=False, store=True,
    required=True, precompute=True, tracking=True,
)

@api.depends('asset_model_ids')
def _compute_create_asset(self):
    for account in self:
        if not account.create_asset or account.create_asset == 'no':
            account.create_asset = 'draft' if account.asset_model_ids else 'no'
```
**BEHAVIORAL CHANGE**: Setting `asset_model_ids` on an account now auto-sets `create_asset='draft'` if it was 'no'. This makes the workflow more intuitive but changes existing behavior where you could have models linked without auto-creation.

## Changes in account.asset.report.handler

### Class renamed
```python
# v18: AssetsReportCustomHandler
# v19: AccountAssetReportHandler
```

### _inherit format changed
```python
# v18: _inherit = 'account.report.custom.handler'
# v19: _inherit = ['account.report.custom.handler']
```

### _get_custom_display_config moved to _custom_options_initializer
```python
# v18: separate method _get_custom_display_config() returning dict
# v19: display config set directly in options['custom_display_config'] inside _custom_options_initializer
```

### "First Depreciation" column removed from report
```python
# v18 had: "first_depreciation": al["asset_date"] and format_date(...)
# v19: this column is REMOVED from _query_lines output
```
The Characteristics subheader colspan changed from 4 to 3 accordingly.

### self._cr replaced with self.env.cr
In `_query_values()`. Style cleanup.

### AssetsReport class renamed to AccountReport
```python
# v18: class AssetsReport(models.Model):
# v19: class AccountReport(models.Model):
```

## Changes in asset.modify Wizard

### Deprecated filter removed from all account domains
All six account Many2one fields in the wizard had `domain="[('deprecated', '=', False)]"` removed.

## Migration Checklist

### Must-Do
- [ ] Update dependency from `account_reports` to `accountant` in any module depending on `account_asset`
- [ ] If overriding `_get_disposal_moves()`: update `get_line` signature to include `is_sale` parameter
- [ ] If overriding `_compute_related_purchase_value()`: account for new `deductible_amount` field
- [ ] If overriding `_get_custom_display_config()`: migrate to `options['custom_display_config']` in `_custom_options_initializer`
- [ ] If referencing `AssetsReportCustomHandler` class name: rename to `AccountAssetReportHandler`
- [ ] If referencing `AssetsReport` class name: rename to `AccountReport`

### Should-Do
- [ ] Review any custom code using `self._context` on asset models (now `self.env.context`)
- [ ] Review any custom code using `self._cr` (now `self.env.cr`)
- [ ] Test assets with `account.account.create_asset` — auto-setting behavior changed
- [ ] Verify depreciation schedule report if you used "First Depreciation" column data
- [ ] Test disposal flows with storno accounting enabled

### Nice-to-Have
- [ ] Leverage new XLSX import template controller for bulk asset imports
- [ ] Use chart template data for pre-defined asset models
- [ ] Leverage annual closing check for fixed assets
- [ ] Benefit from `parent_id` index for faster queries
- [ ] Benefit from `_compute_value_residual` optimization (automatic)
