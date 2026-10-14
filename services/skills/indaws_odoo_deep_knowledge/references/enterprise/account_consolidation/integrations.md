<!-- DEPRECATED v17-only: el módulo account_consolidation fue ELIMINADO en Odoo 18 (no existe en v18/v19 CE ni EE). Este conocimiento solo aplica a instalaciones legacy v17. NO usar para proyectos Odoo 18+. Citas verificadas contra fuente v17 (presente en EE 16-17, eliminado en v18): `cite_audit --methods account_consolidation --roots <ee>/17` da 0 sím✗. Con los roots por defecto v18/v19 salen marcadas = artefacto de versión, NO deuda. -->

> ⚠️ **v17-only — módulo eliminado en Odoo 18.** No aplica a proyectos v18+; conservado solo para legacy v17.

# account_consolidation Integrations (v17)

> **EE only · v17 last available version.** Module REMOVED in v18/v19 — these integrations exist ONLY in v17. All `path:line` citations are relative to `odoo_enterprise/17/account_consolidation/`. Manifest deps: `account_reports`, `web_grid` (`account/__manifest__.py:7`).

This file documents how `account_consolidation` couples to the rest of Odoo in v17. Each integration is verified against source.

---

## (a) account_reports — the consolidated balance report engine

`account_consolidation` does NOT define its own reporting framework; it rides on `account_reports`' `account.report` + custom-handler machinery.

- `account_consolidation/data/consolidated_balance_report.xml:4`, `consolidated_balance_report` declares an `account.report` ("Consolidated Balance") with `custom_handler_model_id ref="model_consolidation_trial_balance_report_handler"` (`:11`) and `filter_multi_company` = `selector` (`:10`).
- The handler is `consolidation.trial.balance.report.handler`, `_inherit = 'account.report.custom.handler'` (`account_consolidation/report/trial_balance.py:11-13`, `TrialBalanceCustomHandler`) — an AbstractModel, NOT a stored report.
- It overrides the `account.report` extension points:
  - `_get_custom_display_config` (`account_consolidation/report/trial_balance.py:16-25`, `TrialBalanceCustomHandler._get_custom_display_config`) swaps in OWL components `account_consolidation.ConsolidationReportFilters`, `ConsolidationReportLineCell`, `ConsolidationReportHeader`.
  - `_dynamic_lines_generator` (`:27-34`) builds lines through `report/builder/` (`DefaultBuilder`, `ComparisonBuilder`) and `report/handler/` (`PeriodsHandler`, `JournalsHandler`) — imported at `account_consolidation/report/trial_balance.py:5-8`.
  - `_custom_options_initializer` (`:36-62`) injects consolidation-specific options (`selected_period_id`, `consolidation_hierarchy`, `consolidation_show_zero_balance_accounts`) and rewrites the report buttons.
  - `export_to_pdf` (`:213-218`) is SHADOWED to force landscape when `len(options['column_headers'][0]) > 5`, then delegates back to `account.report.export_to_pdf` — because the consolidation report uses dynamic column headers, not declared columns, so the standard landscape heuristic doesn't apply.
- Audit drill-down (`action_open_audit`, `:184-211`) parses the report `line_id` via `self.env['account.report']._parse_line_id(...)` (`:185`) — direct dependency on `account.report`'s line-id encoding.

**Migration consequence:** removing the module orphans this `account.report` record; the consolidated balance view is the single hardest piece to reproduce in v18+ (must be rebuilt as a fresh `account.report` + custom handler). See `v19_changes.md`.

---

## (b) account.move / account.move.line — cross-company AML aggregation

Journal generation reads posted move lines from EVERY company in the chart and folds them into consolidation accounts.

- Entry point `consolidation.period.action_generate_journals` (`account_consolidation/models/consolidation_period.py:155`, `ConsolidationPeriod.action_generate_journals`) checks access then escalates to `record.sudo()` (`:168`) specifically because it must read AML across companies the user may not all belong to. Comment at `:167`: "Since he has the rights to be here, we can go sudo from here".
- Per-company aggregation lives in `consolidation.company_period._get_journal_lines_values` (`account_consolidation/models/consolidation_period.py:614-637`, `ConsolidationCompanyPeriod._get_journal_lines_values`):
  - Non-historical accounts → ONE line via `_get_total_balance_and_audit_lines` (`:641-651`), which runs `self.env['account.move.line']._read_group(domain, [], ['balance:sum', 'id:array_agg'])` (`:650`) — a single grouped sum plus the array of contributing AML ids for audit.
  - Historical accounts → ONE line PER move line via `_get_historical_journal_lines_values` (`:690-703`), `search()`-ing the AML directly.
- The AML selection domain `_get_move_lines_domain` (`account_consolidation/models/consolidation_period.py:705-726`, `ConsolidationCompanyPeriod._get_move_lines_domain`):
  - `('parent_state', '=', 'posted')` `:718` — only posted entries.
  - `('company_id', '=', self.company_id.id)` `:719` — scoped to the company period's company.
  - `('journal_id', 'not in', self.mapped('exclude_journal_ids.id'))` `:720` — honors per-company-period journal exclusions (`exclude_journal_ids`, `account_consolidation/models/consolidation_period.py:555`, `ConsolidationCompanyPeriod.exclude_journal_ids`).
  - `('account_id.consolidation_account_ids', '=', consolidation_account.id)` `:721` — the mapping link (see (c)).
  - date window `:722-725` with an `include_initial_balance` OR-branch so balance-sheet accounts carry opening balances.
- The contributing AML ids are stored back on the generated line via `move_line_ids = [(6,0,...)]` (`account_consolidation/models/consolidation_period.py:635`, `ConsolidationCompanyPeriod._get_journal_lines_values`), enabling audit drill-down. The reverse link is `account.move.line.consolidation_journal_line_ids` (M2M, `account_consolidation/models/account_move.py:9`, `class AccountMoveLine` (v17: 18.0 retiró el módulo)).

**Migration consequence:** this is the actual "consolidation engine" — a cross-company posted-AML aggregator with FX/rate application. Replacing it in v18+ means an `account.report` with a custom expression engine doing the same cross-company `_read_group`.

---

## (c) account.account ↔ consolidation.account mapping (M2M)

The bridge between real company GL accounts and consolidation accounts is a single M2M, surfaced through context-filtered helper fields on both sides.

- `account.account.consolidation_account_ids` (M2M → `consolidation.account`, `account_consolidation/models/account_account.py:10`, `class Account`) is the canonical link.
- `consolidation.account.account_ids` (M2M → `account.account`, `account_consolidation/models/consolidation_coa.py:164`, `ConsolidationAccount.account_ids`) is the inverse direction used during mapping.
- Both models expose a `*_chart_filtered_*` computed/searchable mirror that filters by `context['chart_id']`:
  - `account.account.consolidation_account_chart_filtered_ids` (`account_consolidation/models/account_account.py:11-16`, `class Account`, compute `_compute_filtered_consolidation_account_ids` `:19-32`, inverse `:34-44`, search `:46-59`). This field is COMPUTED & NON-STORED — no DB column.
  - `consolidation.account.filtered_used_in_ids` (`account_consolidation/models/consolidation_coa.py:180-184`, `ConsolidationAccount.filtered_used_in_ids`, compute/inverse/search `:218-255`).
- The mapping UI is driven by actions that inject `chart_id`/`company_id` into context: `consolidation.chart.action_open_mapping` (`account_consolidation/models/consolidation_coa.py:77-92`, `ConsolidationChart.action_open_mapping`), `consolidation.period.action_open_mapping` (`account_consolidation/models/consolidation_period.py:184-206`), `res.company.action_open_mapping` (`account_consolidation/models/res_company.py:24-36`, `Company.action_open_mapping`).
- "Unmapped accounts" counters (`consolidation.period._compute_company_unmapped_accounts_counts`, `account_consolidation/models/consolidation_period.py:86-110`, `ConsolidationPeriod._compute_company_unmapped_accounts_counts`) query `account.account` with `consolidation_account_chart_filtered_ids = False` AND `used = True`.

**Gotcha tie-in:** the smart M2M write on `consolidation.account` (ACG7, `account_consolidation/models/consolidation_coa.py:191-201`, `ConsolidationAccount.write`) preserves out-of-`env.companies` mappings — relevant when scripting mappings.

---

## (d) web_grid — trial-balance editing grid

`web_grid` is a hard manifest dependency (`account/__manifest__.py:7`) used for the editable trial-balance matrix (accounts × journals).

- `consolidation.period.action_open_trial_balance_grid` (`account_consolidation/models/consolidation_period.py:221-238`, `ConsolidationPeriod.action_open_trial_balance_grid`) opens `consolidation.journal.line` in `view_mode='grid,graph,form'`.
- The report "Edit" button routes to `TrialBalanceCustomHandler.action_open_view_grid` (`account_consolidation/report/trial_balance.py:164-182`, `TrialBalanceCustomHandler.action_open_view_grid`), opening the same grid view (`view_trial_balance_report_grid`).
- Grid cell writes go through `consolidation.journal.line.grid_update_cell` (`account_consolidation/models/consolidation_journal.py:167-181`, `ConsolidationJournalLine.grid_update_cell`): it reads `default_account_id`/`default_journal_id` from context, refuses to edit if the journal is `auto_generated` (`:174-175`), and CREATEs an adjustment line noted "Trial balance adjustment".
- `_journal_is_editable` (`account_consolidation/models/consolidation_journal.py:185-199`, `ConsolidationJournalLine._journal_is_editable`) tells the grid which columns are editable (only non-auto-generated journals).

**Migration consequence:** `web_grid` survives into v18/v19, but the consolidation grid views/actions vanish with the module.

---

## (e) Multi-company / res.company

Consolidation is inherently multi-company; `res.company` is extended and used as the unit of aggregation.

- `consolidation.chart.company_ids` (M2M, `account_consolidation/models/consolidation_coa.py:23`, `ConsolidationChart.company_ids`) defines which companies are in scope.
- One `consolidation.company_period` is generated per company (`consolidation.period._get_company_periods_default_values_from_chart`, `account_consolidation/models/consolidation_period.py:341-360`, `ConsolidationPeriod._get_company_periods_default_values_from_chart`, iterating `chart_id.company_ids`).
- `res.company` extensions (`models/res_company.py`):
  - `consolidation_color` (`:13`) with default `_default_consolidation_color` = `search_count([]) % 12` (`:9-10`) — colors mapped accounts in the mapping tree.
  - `account_consolidation_currency_is_different` (computed, `:15-22`) compares `context['consolidation_currency_id']` to the company currency — drives FX UI.
  - `action_open_mapping` (`:24-36`) and `action_open_rate_ranges` (`:38-56`) — company-scoped mapping and historical-rate entry.
- Cross-company AML read requires `sudo()` (see (b), `account_consolidation/models/consolidation_period.py:168`, `ConsolidationPeriod`) because the operator may not be a member of all `company_ids`.
- The "unmapped accounts" computation intersects chart companies with `env.user.company_ids` (`account_consolidation/models/consolidation_period.py:94-96`, `ConsolidationPeriod._compute_company_unmapped_accounts_counts` (usa `env.user.company_ids`)) — accounts in companies the user can't access are silently excluded from the counter.

---

## (f) res.currency + consolidation.rate — FX conversion

Currency conversion combines per-company-period fixed rates, a dedicated historical-rate model, and standard `res.currency` as fallback.

- Target currency is `consolidation.chart.currency_id` (`account_consolidation/models/consolidation_coa.py:13`, `ConsolidationChart.currency_id`); source is the company currency (`consolidation.company_period.currency_company_id`, related, `account_consolidation/models/consolidation_period.py:538`, `ConsolidationCompanyPeriod.currency_company_id`).
- Fixed per-period rates: `currency_rate_avg` / `currency_rate_end` (`account_consolidation/models/consolidation_period.py:539-542`, `ConsolidationCompanyPeriod.currency_rate_avg`, `digits=[12,12]`). `_convert` (`:728-745`) picks `currency_rate_<mode>` by `getattr` and DIVIDES (stored as "1 chart ccy = N company ccy", reverse op).
- Historical mode (`hist`) uses the `consolidation.rate` model (`account_consolidation/models/consolidation_rate.py:7`, `ConsolidationRate`): per `(chart_id, company_id)` rate valid over `[date_start, date_end]` (`:10-14`). Lookup `get_rate_for(date, company_id, chart_id)` (`:16-36`) returns the matching `rate` or `False`.
  - `consolidation.company_period._apply_historical_rates` (`account_consolidation/models/consolidation_period.py:670-688`, `ConsolidationCompanyPeriod._apply_historical_rates`): if `consolidation.rate` returns a rate, multiply; ELSE fall back to standard `res.currency._convert(amount, chart_currency, company, move_line.date)` (`:687`).
  - `consolidation.company_period.conversion_rate` (computed, `account_consolidation/models/consolidation_period.py:556`, `ConsolidationCompanyPeriod.conversion_rate`, con su compute en `:567-573`) reads `consolidation.rate.get_rate_for` using `context['date']/company_id/chart_id`.
- Sub-consolidation FX: `consolidation.period.composition.currency_rate` (`account_consolidation/models/consolidation_period.py:409`, `ConsolidationPeriodComposition.currency_rate`) defaults via `_get_default_currency_rate` (`:511-521`) → `self.env['res.currency']._get_conversion_rate(from, to, company, now)` between composed-chart and using-chart currencies.

**Migration consequence:** v17 has its OWN rate model (`consolidation.rate`) decoupled from `res.currency.rate`. There is no automatic propagation (ACG11). In v18+, FX must be modeled with standard `res.currency.rate` + manual adjustment journals.

---

## Integration map (one-liner per edge)

| From | To | Mechanism | Source |
|------|----|-----------|--------|
| consolidation report | `account.report` / `account.report.custom.handler` | custom handler model | `account_consolidation/report/trial_balance.py:11-13`, `TrialBalanceCustomHandler`, `account_consolidation/data/consolidated_balance_report.xml:11`, `consolidated_balance_report` |
| company_period | `account.move.line` | cross-company `_read_group` under sudo | `account_consolidation/models/consolidation_period.py:650`, `ConsolidationCompanyPeriod._get_total_balance_and_audit_lines`, `:168` |
| `account.account` | `consolidation.account` | M2M `consolidation_account_ids` | `account_consolidation/models/account_account.py:10`, `class Account` |
| consolidation grid | `web_grid` | grid view + `grid_update_cell` | `account_consolidation/models/consolidation_journal.py:167`, `ConsolidationJournalLine.grid_update_cell`, `account_consolidation/models/consolidation_period.py:221`, `ConsolidationPeriod.action_open_trial_balance_grid` |
| chart | `res.company` | `company_ids` M2M + per-company periods | `account_consolidation/models/consolidation_coa.py:23`, `ConsolidationChart.company_ids`, `account_consolidation/models/consolidation_period.py:351`, `ConsolidationPeriod._get_company_periods_default_values_from_chart` |
| company_period | `res.currency` / `consolidation.rate` | rate lookup + `_convert` fallback | `account_consolidation/models/consolidation_period.py:670-688`, `ConsolidationCompanyPeriod`, `account_consolidation/models/consolidation_rate.py:16`, `ConsolidationRate.get_rate_for` |
| journal.line | `account.move.line` | audit back-link M2M | `account_consolidation/models/account_move.py:9`, `class AccountMoveLine` (v17: 18.0 retiró el módulo), `account_consolidation/models/consolidation_period.py:635`, `ConsolidationCompanyPeriod._get_journal_lines_values` |
