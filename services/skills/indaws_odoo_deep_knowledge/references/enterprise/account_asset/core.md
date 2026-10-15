# Account Asset — Odoo 18.0 Enterprise — EE
<!-- verified: v18.0 @ 2026-06 -->

> **Enterprise (OEEL-1)**. Módulo `account_asset` en `odoo_enterprise/18/account_asset`. Depende de `account_reports` (v18) / `accountant` (v19). No existe en Community.

## 1. Model Map

| Model | Type | Purpose |
|---|---|---|
| `account.asset` | Regular (mail.thread, mail.activity.mixin, analytic.mixin) | Core asset/revenue recognition model. Used for assets, deferred revenues, deferred expenses, and asset models |
| `account.asset.group` | Regular | Grouping for assets (name + company). One2many to `account.asset` |
| `account.asset.report.handler` | AbstractModel (inherits `account.report.custom.handler`) | Depreciation Schedule report logic — SQL queries, grouping, line generation |
| `asset.modify` | TransientModel | Wizard for dispose, sell, re-evaluate, pause, resume actions |
| `account.move` (inherited) | - | Adds `asset_id`, `depreciation_value`, `asset_move_type`, depreciation cumulative values |
| `account.move.line` (inherited) | - | Adds `asset_ids` M2M, `non_deductible_tax_value`, `turn_as_asset()` action |
| `account.account` (inherited) | - | Adds `create_asset` selection, `asset_model_ids` M2M, `multiple_assets_per_line`, `can_create_asset` |
| `res.company` (inherited) | - | Adds `gain_account_id`, `loss_account_id` for disposal gain/loss |

**Bridge module**: `account_asset_fleet` adds `vehicle_id` to `account.asset` and propagates it to depreciation move lines.

**Dependency**: `account_reports` (Enterprise). Auto-installed.

## 2. account.asset — Fields & State Machine

### State Machine
```
draft --> open (validate) --> paused (pause) --> open (resume)
  |         |                                      |
  |         +--> close (set_to_close)              +--> close
  |         |
  |         +--> cancelled (set_to_cancelled)
  |
  +--> cancelled
  
model (special state for templates)
```

### Key Fields

**Identity & Classification:**
- `name` (Char, computed from original_move_line_ids[0].name)
- `company_id`, `currency_id` (related to company)
- `state` (model/draft/open/paused/close/cancelled)
- `active` (Boolean, constraint: cannot archive if not closed or model)
- `asset_group_id` (Many2one to account.asset.group)
- `country_code` (related to company fiscal country)

**Depreciation Parameters:**
- `method` (Selection: linear/degressive/degressive_then_linear)
- `method_number` (Integer, default 5 — number of periods)
- `method_period` (Selection: '1'=months, '12'=years)
- `method_progress_factor` (Float, default 0.3 — declining factor)
- `prorata_computation_type` (Selection: none/constant_periods/daily_computation)
- `prorata_date` (Date, computed from acquisition_date + prorata_computation_type)
- `paused_prorata_date` (Date, computed — prorata_date shifted by paused days)
- `asset_lifetime_days` (Float, computed — total days for depreciation)
- `asset_paused_days` (Float, copy=False — accumulated pause days)

**Accounts:**
- `account_asset_id` (Many2one — fixed asset account, e.g., 211000)
- `account_depreciation_id` (Many2one — accumulated depreciation, e.g., 281100)
- `account_depreciation_expense_id` (Many2one — expense account, e.g., 681100)
- `journal_id` (Many2one — general journal, computed from company)

**Values:**
- `original_value` (Monetary, computed from move lines or manual)
- `salvage_value` (Monetary — not depreciable, computed from model_id.salvage_value_pct)
- `salvage_value_pct` (Float — percentage for model templates)
- `total_depreciable_value` (computed: original_value - salvage_value)
- `value_residual` (computed: original - salvage - imported - posted depreciations)
- `book_value` (computed recursive: value_residual + salvage_value + children book values)
- `gross_increase_value` (computed: sum of children original_value)
- `non_deductible_tax_value` (computed from original move lines)
- `related_purchase_value` (computed: sum of original_move_line_ids.balance, divided by qty if multiple_assets_per_line)
- `already_depreciated_amount_import` (Monetary — for software migration)

**Dates:**
- `acquisition_date` (computed from original_move_line_ids or today)
- `disposal_date` (computed: max date of depreciation_move_ids when state=close)

**Links:**
- `depreciation_move_ids` (One2many account.move via asset_id)
- `original_move_line_ids` (Many2many account.move.line via asset_move_line_rel)
- `model_id` (Many2one account.asset — template model)
- `parent_id` / `children_ids` (self-referential — gross increases)
- `linked_assets_ids` (computed — other assets sharing same move lines)

**Properties:**
- `asset_properties_definition` (PropertiesDefinition — on models)
- `asset_properties` (Properties — on instances, defined by model_id)

### account.move Fields Added
- `asset_id` (Many2one, ondelete=cascade)
- `depreciation_value` (Monetary, computed from line balances)
- `asset_remaining_value` / `asset_depreciated_value` (computed cumulative)
- `asset_value_change` (Boolean — marks revaluation entries)
- `asset_number_days` (Integer, deprecated)
- `asset_depreciation_beginning_date` (Date)
- `asset_move_type` (Selection: depreciation/sale/purchase/disposal/negative_revaluation/positive_revaluation)
- `asset_ids` (One2many computed from line_ids.asset_ids)
- `count_asset`, `draft_asset_exists`, `asset_id_display_name` (computed)

## 3. Depreciation Methods

### Linear (Straight Line)
Formula: `amount = total_depreciable_value * days_in_period / asset_lifetime_days`

The computation uses cumulative expected amounts to avoid rounding drift:
```
amount_expected_at_end = total_depreciable * days_until_period_end / lifetime
amount_expected_before = total_depreciable * days_before_period / lifetime
period_amount = amount_expected_at_end - amount_expected_before
```

Value decreases are spread proportionally over remaining lifetime.

### Degressive (Declining)
Each fiscal year, computes: `residual * declining_factor * days_in_period / days_in_fiscal_year`

The implementation takes the max of degressive vs linear amount within each fiscal year period, so it never goes below what straight-line would produce for that period.

At each fiscal year boundary, `residual_declining` resets to the current residual amount.

### Degressive then Linear
Uses declining method but switches to linear when linear produces a higher amount. For child assets (gross increases), aligns the transition point with the parent asset's curve using proportional calculation.

### Constants
```python
DAYS_PER_MONTH = 30
DAYS_PER_YEAR = 360  # (30 * 12)
```

## 4. Depreciation Board Computation

### compute_depreciation_board(date=False) (`account_asset/models/account_asset.py:636`, `AccountAsset._compute_board_amount`)
1. Unlinks draft moves with `mv.date >= date` (or all drafts if `date` is falsy) — `:638`
2. Calls `_recompute_board(date)` for each asset (`:642`)
3. Creates new moves from returned values (`:644`)
4. `_post()`s only the moves whose `asset.state == 'open'` (`:645-647`)

### _recompute_board(start_depreciation_date=False)
1. Gathers posted moves (non-value-change), sorted by (date, id)
2. Computes residual from `value_residual - sum(draft moves)`
3. Iterates from start_date to final_depreciation_date:
   - Computes period end via `_get_end_period_date()`
   - Calls `_compute_board_amount()` for each period
   - Handles `already_depreciated_amount_import` by skipping initial moves
   - Creates move values via `_prepare_move_for_asset_depreciation()`
4. Returns list of move value dicts

### _get_delta_days(start, end)
Two modes:
- **daily_computation**: `(end - start).days + 1` (real calendar days)
- **constant_periods**: 30-day months formula with prorata for partial months

### _get_end_period_date(start)
- Yearly: end of fiscal year containing start date
- Monthly (`method_period='1'`): end of the calendar month, capped at fiscal year end

### _get_last_day_asset() (`account_asset/models/account_asset.py:750`, `AccountAsset._get_delta_days`)
Uses the parent's record (`this = self.parent_id or self`) if the asset has a parent (gross increase), then:
`this.paused_prorata_date + relativedelta(months=int(method_period)*method_number, days=-1)`

## 5. Asset Creation

### Manual
Create directly from Accounting > Assets menu. Fill name, accounts, method, value.

### From Invoice Line (automatic)
Triggered in `account.move._post()` -> `_auto_create_asset()`:
1. For each posted invoice line where `account.can_create_asset` and `create_asset != 'no'`:
2. Checks: not zero amount, not already linked, not tax line, positive price_total
3. If `multiple_assets_per_line`: creates `int(quantity)` separate assets
4. If account has `asset_model_ids`: applies model via `_onchange_model_id()`
5. If `create_asset == 'validate'`: auto-validates the asset
6. Posts chatter message linking asset to invoice

### From Journal Items
`account.move.line.turn_as_asset()` — opens asset form with selected lines as `original_move_line_ids`.

### From Credit Note
When an invoice with assets is reversed via `button_cancel()`, associated assets are deactivated (`active=False`).

## 6. Disposal & Sale of Assets

### Disposal Flow (via asset.modify wizard)
1. User selects "Dispose" action
2. Wizard computes gain/loss comparing book value to zero
3. Calls `asset.set_to_close(invoice_line_ids=empty, date=date)`
4. Creates depreciation up to disposal date via `_create_move_before_date()`
5. Creates disposal move with lines:
   - Reversal of original value on asset account
   - Reversal of accumulated depreciation
   - Difference to gain or loss account

### Sale Flow
1. User selects "Sell", attaches customer invoice(s)
2. Wizard computes gain/loss comparing book value to invoice amount
3. Calls `asset.set_to_close(invoice_line_ids=lines, date=date)`
4. Creates depreciation up to sale date
5. Creates sale move:
   - Reversal of original value on asset account
   - Reversal of accumulated depreciation
   - Invoice line amounts on their accounts
   - Difference to gain or loss account
6. Sets `net_gain_on_sale` on asset

### Cancellation
`set_to_cancelled()`: Reverses all posted moves, unlinks drafts, resets paused_days, logs detailed message.

### Key Constraint
Cannot sell an asset with running gross increases (children in draft/open state).

## 7. Asset Groups / Categories

`account.asset.group` is a simple grouping model:
- `name` (Char, trigram indexed)
- `company_id`
- `linked_asset_ids` (One2many)
- `count_linked_assets` (computed via _read_group)

Used in the Depreciation Schedule report for grouping (`assets_grouping_field` option).

**Asset Models** (state='model') serve as templates:
- Define default method, accounts, journal, prorata settings
- Linked to accounts via `account.account.asset_model_ids`
- Applied via `_onchange_model_id()` which copies all params
- Support `salvage_value_pct` for percentage-based salvage
- Support `asset_properties_definition` (Properties system)

## 8. Accounting Entries

### Depreciation Entry (periodic)
```
DR  Expense Account (account_depreciation_expense_id)     amount
CR  Depreciation Account (account_depreciation_id)        amount
```
- Partner preserved from original invoice (if single partner)
- Analytic distribution propagated from asset
- Currency conversion at depreciation date
- Ref: "{Asset Name}: Depreciation"

### Disposal Entry (no sale)
```
DR  Depreciation Account          accumulated_depreciation
DR  Gain/Loss Account             difference (if loss)
CR  Asset Account                  original_value
CR  Gain/Loss Account             difference (if gain)
```

### Sale Entry
```
DR  Depreciation Account          accumulated_depreciation
DR  Loss Account                   difference (if loss)
CR  Asset Account                  original_value
CR  Invoice Line Account(s)       invoice_amounts
CR  Gain Account                   difference (if gain)
```

### Gross Increase (Re-evaluation)
When value increases via asset.modify wizard:
```
DR  Gross Increase Account        increase_amount
CR  Counterpart Account           increase_amount
```
A child asset is created and validated with the increase amount.

### Value Decrease (Re-evaluation)
```
DR  Expense Account               decrease_amount
CR  Depreciation Account          decrease_amount
```
Marked with `asset_value_change=True`.

### Reversal Handling
When a depreciation move is reversed:
- Amount is redistributed to next draft move (if exists)
- Or a new move is created at last_date + period
- Chatter message logged on asset

## 9. Deferred Revenue/Expense

The same `account.asset` model handles deferred revenues and expenses. The distinction is purely in the account types used:
- **Deferred Revenue**: `account_asset_id` = revenue account, `account_depreciation_expense_id` = deferred revenue account
- **Deferred Expense**: `account_asset_id` = expense account type

The `can_create_asset` computed field on `account.account` returns True for `asset_fixed` and `asset_non_current` account types.

The `account_type` related field on the asset determines the form view behavior.

## 10. Tax Implications

### Non-Deductible Taxes
Special handling for taxes where `use_in_tax_closing=False` on repartition lines (e.g., Belgian 50% non-deductible car expenses):

1. `account.move.line._compute_non_deductible_tax_value()` queries tax details to find non-deductible portions
2. `account.asset._compute_non_deductible_tax_value()` aggregates these per asset
3. Added to `original_value` in `_compute_value()`
4. Logged in chatter via `_post_non_deductible_tax_value()`

### Tax on Depreciation Lines
When `move.asset_id` is set, `_get_computed_taxes()` returns `self.tax_ids` directly instead of computing from fiscal position, preventing tax recomputation on depreciation entries.

## Security

### Access Rules (ir.model.access.csv)
| Group | Read | Write | Create | Unlink |
|---|---|---|---|---|
| group_account_readonly | Yes | No | No | No |
| group_account_user | Yes | Yes | Yes | Yes |
| group_account_invoice | Yes | No | Yes | No |
| group_account_manager (groups only) | Yes | Yes | Yes | Yes |

### Record Rules
- Multi-company rule: `('company_id', 'parent_of', company_ids)` — global, applies to both account.asset and account.asset.group

### Deletion Constraints
- Cannot delete assets in open/paused/close state
- Cannot delete assets linked to posted entries
- Cannot reset to draft a move linked to a posted asset

## asset.modify Wizard

### Actions
| Action | Effect |
|---|---|
| `dispose` | Creates depreciation to date + disposal entry, closes asset |
| `sell` | Like dispose but uses invoice lines for sale amount |
| `modify` (Re-evaluate) | Changes duration/salvage/residual, creates gross increase child if value increased, creates decrease entry if decreased, recomputes board |
| `pause` | Creates depreciation to date, sets state to paused |
| `resume` | Adds paused days, recomputes board from resume date |

### Key Validations
- Cannot modify if posted future depreciation entries exist (must reverse first)
- Cannot modify/sell before fiscal lock date
- Cannot sell asset with running gross increases
- Unposted depreciations before operation date must be dealt with first
- Resume date must be after pause date

## Depreciation Schedule Report

`account.asset.report.handler` generates the depreciation schedule report with columns:
- **Characteristics**: Acquisition Date, First Depreciation, Method, Duration/Rate
- **Assets**: Opening, Additions, Disposals, Closing
- **Depreciation**: Opening, Additions, Disposals, Closing
- **Book Value**: Net book value

Features:
- Grouping by account or asset group (configurable)
- Account hierarchy support
- Prefix-based name grouping (unfoldable)
- Direct SQL query joining account_asset + account_move
- Supports draft entries option (`all_entries`)
- Analytic filtering
- Journal filtering
