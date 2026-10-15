<!-- DEPRECATED v17-only: el módulo account_consolidation fue ELIMINADO en Odoo 18 (no existe en v18/v19 CE ni EE). Este conocimiento solo aplica a instalaciones legacy v17. NO usar para proyectos Odoo 18+. Citas verificadas contra fuente v17 (presente en EE 16-17, eliminado en v18): `cite_audit --methods account_consolidation --roots <ee>/17` da 0 sím✗. Con los roots por defecto v18/v19 salen marcadas = artefacto de versión, NO deuda. -->

> ⚠️ **v17-only — módulo eliminado en Odoo 18.** No aplica a proyectos v18+; conservado solo para legacy v17.

# account_consolidation — Multi-company Financial Consolidation

> **EE only · v17 last available version.** All `path:line` citations refer to `odoo_enterprise/17/account_consolidation/`.

> **STATUS — REMOVED in v18 and v19.**
>
> Last available in Odoo 17.0 Enterprise (`odoo_enterprise/17/account_consolidation/`).
> No longer present in `odoo_enterprise/18/` or `odoo_enterprise/19/`.
>
> Customers running this in v17 must plan an alternative for v18+ migration:
> - Manual consolidation via `account_reports` (cross-company filters + intercompany journals).
> - Custom OCA-style multi-company aggregation modules.
> - Third-party / external consolidation tools (e.g. Lucanet, BoardWalk, BlackLine, Tagetik).
>
> This file documents the v17 implementation as reference for migration projects.

## Module (v17)

Path: `odoo_enterprise/17/account_consolidation/` (EE only).
`'application': True`, `OEEL-1`. Depends: `account_reports`, `web_grid`.

## Models (v17)

All `_name` values verified against v17 source.

| Model (`_name`) | Source | Purpose |
|-------|--------|---------|
| `consolidation.chart` | `account_consolidation/models/consolidation_coa.py:9`, `ConsolidationChart._name` | ROOT model. Consolidation chart of accounts (`currency_id` target `:13`, `period_ids` `:14`, `account_ids` `:16`, `group_ids` `:18`, `rate_ids` `:20`, `company_ids` `:23`, parent/children consolidations `:24-27`). NOTE: the chart is the root, NOT `consolidation.account`. |
| `consolidation.account` | `account_consolidation/models/consolidation_coa.py:148`, `ConsolidationAccount` | Mapped consolidation account (`currency_mode` end/avg/hist `:165`, `group_id` `:163`, `account_ids` M2M to `account.account` `:164`) |
| `consolidation.group` | `account_consolidation/models/consolidation_coa.py:291`, `ConsolidationGroup` | Hierarchy section, `_parent_store=True` `:295` (`parent_id` self-ref `:301`, `account_ids` `:304`, `sign`/`invert_sign` `:307-308`, `show_on_dashboard` `:300`) |
| `consolidation.period` | `account_consolidation/models/consolidation_period.py:13`, `ConsolidationPeriod` | Time period (`date_analysis_begin/end` `:28-29`, `state` draft/closed `:30`, `chart_id` `:23`, `company_period_ids` `:37`, `journal_ids` `:38`) — `_inherit=['mail.thread']` `:16` |
| `consolidation.company_period` | `account_consolidation/models/consolidation_period.py:524`, `ConsolidationCompanyPeriod` | One row per company in a period (`currency_company_id` `:538`, `currency_rate_avg/end` `:539-542`, `rate_consolidation` `:548`, `consolidation_method` `:550`, `exclude_journal_ids` `:555`) |
| `consolidation.period.composition` | `account_consolidation/models/consolidation_period.py:396`, `ConsolidationPeriodComposition` | Sub-consolidation linkage (`using_period_id` ↔ `composed_period_id` `:402-404`, `rate_consolidation` `:407`, `currency_rate` `:409`) |
| `consolidation.journal` | `account_consolidation/models/consolidation_journal.py:9`, `ConsolidationJournal` | Generated journal (`auto_generated` flag `:26`, `line_ids` `:24`, `currencies_are_different` `:32`, `composition_id`/`company_period_id` origins `:17-22`) |
| `consolidation.journal.line` | `account_consolidation/models/consolidation_journal.py:117`, `ConsolidationJournalLine` | Consolidated entry (`account_id` `:123`, `amount`+`currency_amount` `:129-132`, `move_line_ids` audit M2M `:136`) |
| `consolidation.rate` | `account_consolidation/models/consolidation_rate.py:7`, `ConsolidationRate` | FX rate per company+chart over a date range (`rate` `:12`, `date_start/end` `:13-14`, `get_rate_for()` `:16`) |

## Workflow (v17)

```
1. Create consolidation.chart (target currency, companies)
2. Map company account.account → consolidation.account
3. Create consolidation.period (Q1 2024)
4. Auto-generate company_period rows (one per company in chart.company_ids)
5. Set rate_consolidation per company_period (currency conversion factor)
6. action_generate_journals → creates consolidation.journal per company
   - Walks each company's account.move.line
   - Aggregates by mapped consolidation.account
   - Applies currency_mode + rate_consolidation
7. View consolidated balance on dashboard
8. action_close → state='closed', journals locked
```

`action_generate_journals` (`account_consolidation/models/consolidation_period.py:155`, `ConsolidationPeriod.action_generate_journals`): skips `state=='closed'` periods `:161`, checks write/unlink/create access then drops to `sudo()` `:164-168` (cross-company AML read needs superuser), unlinks only `auto_generated=True` journals `:170-176`, then regenerates one journal per company period (`_generate_company_periods_journals` `:180`) and one per sub-consolidation (`_generate_consolidations_journals` `:182`).

## Currency Modes (consolidation.account.currency_mode)

Selection defined at `account_consolidation/models/consolidation_coa.py:165`, `ConsolidationAccount.currency_mode`.

| Mode | Behaviour |
|------|-----------|
| `end` | Closing rate (`currency_rate_end`) — for balance sheet items |
| `avg` | Average rate (`currency_rate_avg`) — for income statement items |
| `hist` | Historical rate — per-move-line conversion |

Conversion logic: `consolidation.company_period._convert()` (`account_consolidation/models/consolidation_period.py:728`, `ConsolidationCompanyPeriod._convert`) reads `currency_rate_<mode>` via `getattr` and **divides** the amount by it (stored rate = "1 chart currency = N company currency", so reverse op `:744`). `hist` mode bypasses this and is converted line-by-line in `_apply_historical_rates()` `:670` using `consolidation.rate.get_rate_for(move_line.date,...)`, falling back to `currency._convert()` if no rate found.

## Sign Logic

`invert_sign` cascades MULTIPLICATIVELY: `consolidation.account.sign = (-1 if invert else 1) * (group_id or chart_id).sign` (`account_consolidation/models/consolidation_coa.py:257-260`, `ConsolidationAccount`); group sign similarly multiplies `parent_id or chart_id` (`:341-344`, `recursive=True`); chart sign is `-1 if invert_sign else 1` (`:56-59`). Used to flip presentation (e.g. revenue as positive instead of credit-negative). Affects DISPLAY only — `journal.line.amount` keeps raw sign.

## Sub-consolidation

`consolidation.chart.children_ids` (M2M self via custom rel `account_consolidation_inner_rel`, `account_consolidation/models/consolidation_coa.py:24-27`, `ConsolidationChart.children_ids`) — sub-consolidations rolled up into parent. Realised via `consolidation.period.composition` linking `using_period_id` (parent) to `composed_period_id` (child) `account_consolidation/models/consolidation_period.py:402-404`, `ConsolidationPeriodComposition.composed_period_id`. `consolidation.journal._check_unique_origin` (`account_consolidation/models/consolidation_journal.py:35-42`, `ConsolidationJournal._check_unique_origin`) constraint: a journal links EITHER to `company_period_id` OR `composition_id`, never both.

## Group Constraint

`consolidation.group._check_unique_type_of_descendant` (`account_consolidation/models/consolidation_coa.py:321-328`, `ConsolidationGroup._check_unique_type_of_descendant`): a group can have EITHER child groups OR child accounts, never mixed. Impact: tree must be strict hierarchy of section folders → account leaves.

## copy() overrides (non-trivial)

- `consolidation.chart.copy()` (`account_consolidation/models/consolidation_coa.py:61-73`, `ConsolidationChart.copy`): suffixes name " (copy)", bumps color, deep-copies parent groups (which cascade-copy child groups + accounts) and re-links them to the new chart.
- `consolidation.group.copy()` (`:310-318`): recursively copies `child_ids` and `account_ids`.
- `consolidation.account.copy()` (`:203-206`): only renames.
- `consolidation.period.copy()` (`account_consolidation/models/consolidation_period.py:126-130`, `ConsolidationPeriod.copy`): sets `date_analysis_begin = old.date_analysis_end + 1 day`, `date_analysis_end = None` — designed for rolling to the next period.

## Onboarding

Steps registered with `onboarding.onboarding.step`:
- `onboarding_onboarding_step_setup_consolidation` (create first chart)
- `onboarding_onboarding_step_setup_ccoa` (consolidated chart of accounts mapping)
- First period setup

## Dashboard

`consolidation.period._compute_dashboard_sections` returns JSON of group totals filtered by `group_id.show_on_dashboard=True`. Rendered in kanban with `account_consolidation_kanban.scss` and OWL components.

## Reports

`account_consolidation/data/consolidated_balance_report.xml:4`, `consolidated_balance_report` defines an `account.report` record ("Consolidated Balance") wired to `custom_handler_model_id ref="model_consolidation_trial_balance_report_handler"` (`:11`). The handler is the AbstractModel `consolidation.trial.balance.report.handler` which `_inherit = 'account.report.custom.handler'` (`account_consolidation/report/trial_balance.py:11-13`, `TrialBalanceCustomHandler`) — it is NOT a standard report engine, it builds lines dynamically in `_dynamic_lines_generator` `:27` via `report/builder/` (Default + Comparison) and `report/handler/` (Periods + Journals). Shows hierarchical balance grouped by `consolidation.group`. See `integrations.md` for the full `account_reports` coupling.

## Deprecation Reasoning

Removal between v17 and v18 likely due to:
1. Low adoption (multi-company groups typically use specialised tools).
2. `account_reports` cross-company filters cover ~80% of use cases.
3. Maintenance cost of FX rate handling + currency_mode logic.
4. v18 introduced more powerful "Tax Reports" engine with custom expressions covering some consolidation scenarios.

## Migration Path (v17 → v18 / v19)

For projects upgrading from v17 with active consolidations:

1. **Export consolidation chart structure** as CSV before upgrade (consolidation.account, consolidation.group, mappings).
2. **Snapshot period data** as `account.move` entries in a "consolidation" company (target currency).
3. **Recreate as ACCOUNT.REPORT custom engine**: define a custom `account.report` model with computation expressions aggregating cross-company AML.
4. **For ongoing periods**: use multi-company filter in standard `account.report` views, optionally with intercompany journals (`account_inter_company_rules`) to model FX adjustments.
5. **Heavy users**: integrate external consolidation tool via `account.export_csv` + `account_saft`.

## Gotchas

**ACG1. account_consolidation REMOVED in v18+** — Module simply doesn't exist past v17.0. Impact: any v17 → v18 upgrade involving this module REQUIRES external migration plan; standard upgrade.odoo.com leaves orphan tables and broken references.

**ACG2. consolidation.* tables persist after upgrade** — When upgrading v17 → v18, the `consolidation_*` tables remain in PostgreSQL but no module owns them. Impact: data still queryable directly via SQL during migration window, but ORM access broken.

**ACG3. action_generate_journals deletes auto_generated entries** — `auto_generated=True` journals are unlinked then recreated. Manual edits LOST. Impact: customizations to consolidated balances must use `auto_generated=False` journals or be re-applied each regeneration.

**ACG4. Closed periods FORBID new journals** — `_check_not_locked_period` raises if journal added to `state='closed'` period. Impact: late adjustments require reopening period (toggle state), losing audit trail.

**ACG5. Group with mixed children type forbidden** — `_check_unique_type_of_descendant`: children must be all groups OR all accounts. Impact: cannot have a section with both a sub-section AND direct accounts at same level — must wrap accounts in nested group.

**ACG6. period.copy() shifts dates by +1 day, end=None** — `copy()` sets `date_analysis_begin = old.date_analysis_end + 1`, end empty. Impact: rapid duplication for next quarter requires manual end_date entry, missing it produces ambiguous period.

**ACG7. consolidation.account.account_ids M2M smart-write** — Special write logic preserves accounts from companies the user doesn't have access to: only adds/removes within `env.companies` scope. Impact: scripted bulk update may produce unexpected merges.

**ACG8. Currency modes don't auto-apply** — Setting `end`/`avg`/`hist` is metadata only; the actual rate value comes from `consolidation.rate` records. Missing rate at period end → falls back to 1.0 silently. Impact: missing rate produces wrong consolidated balance, no error.

**ACG9. show_on_dashboard at GROUP level only** — Account-level visibility on dashboard not supported. Impact: granular account display requires creating one-account groups (workaround clutter).

**ACG10. `using_ids`/`used_in_ids` M2M poorly named** — Source comment: `# TODO I've no idea what this is for...`. Impact: module reads "account A is used in account B's calculation" but actual semantics unclear; treated as informational metadata.

**ACG11. _name_search supports code prefix shortcut** — Typing `100` in account dropdown matches `code = '100xx'` AND name. Impact: numeric searches behave differently from typical ilike — may surprise migration scripts.

**ACG12. Export to FY rotation requires manual period chain** — No "auto-roll" of consolidation period at fiscal year end. Each period created/copied manually. Impact: high-touch maintenance for monthly close cadence.
