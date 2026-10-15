<!-- DEPRECATED v17-only: el módulo account_consolidation fue ELIMINADO en Odoo 18 (no existe en v18/v19 CE ni EE). Este conocimiento solo aplica a instalaciones legacy v17. NO usar para proyectos Odoo 18+. Citas verificadas contra fuente v17 (presente en EE 16-17, eliminado en v18): `cite_audit --methods account_consolidation --roots <ee>/17` da 0 sím✗. Con los roots por defecto v18/v19 salen marcadas = artefacto de versión, NO deuda. -->

> ⚠️ **v17-only — módulo eliminado en Odoo 18.** No aplica a proyectos v18+; conservado solo para legacy v17.

# account_consolidation Gotchas — Odoo 17.0 Enterprise (last available version)

> **EE only · v17 last.** Module REMOVED in v18.0+. See `core.md` for migration guidance.
> All gotchas below pertain to v17.0; every `path:line` is relative to `odoo_enterprise/17/account_consolidation/`.

## Module Lifecycle

**ACG1. account_consolidation REMOVED in v18+** — Module simply doesn't exist past v17.0. Impact: any v17 → v18 upgrade involving this module REQUIRES external migration plan; standard upgrade.odoo.com leaves orphan tables and broken references.

**ACG2. consolidation.* tables persist after upgrade** — Tables remain in PostgreSQL but no module owns them. Impact: data still queryable directly via SQL during migration window, but ORM access broken.

## Chart & Account Setup

**ACG3. action_generate_journals deletes auto_generated entries** — `account_consolidation/models/consolidation_period.py:170-176`, `ConsolidationPeriod.action_generate_journals`: `auto_generated=True` journals (and their lines, with `allow_unlink=True` context) are unlinked then recreated. Same drop-and-rebuild in `consolidation.journal.action_generate_journal_lines` (`account_consolidation/models/consolidation_journal.py:110-114`, `ConsolidationJournal.action_generate_journal_lines`). Manual edits LOST. Impact: customizations must use `auto_generated=False` journals or be re-applied after each regeneration.

**ACG4. Closed periods FORBID new journals** — `consolidation.journal._check_not_locked_period` (`account_consolidation/models/consolidation_journal.py:44-51`, `ConsolidationJournal._check_not_locked_period`) raises if a journal is added to a `state='closed'` period. Impact: late adjustments require reopening period via `action_toggle_state` (`account_consolidation/models/consolidation_period.py:134`, `ConsolidationPeriod.action_toggle_state`), losing audit-trail consistency.

**ACG5. Group with mixed children type forbidden** — `consolidation.group._check_unique_type_of_descendant` (`account_consolidation/models/consolidation_coa.py:321-328`, `ConsolidationGroup._check_unique_type_of_descendant`): children must be all groups OR all accounts. Impact: cannot have a section with both sub-section AND direct accounts; must wrap loose accounts in nested group.

**ACG6. period.copy() shifts dates by +1 day, end=None** — `consolidation.period.copy()` (`account_consolidation/models/consolidation_period.py:126-130`, `ConsolidationPeriod.copy`) sets `date_analysis_begin = old.date_analysis_end + timedelta(days=1)`, `date_analysis_end = None`. Impact: rapid duplication for next quarter requires manual end_date entry; missing it produces ambiguous open period. NOTE: copy() crashes if `date_analysis_end` is empty (None + timedelta → TypeError).

## Account Mapping

**ACG7. consolidation.account.account_ids M2M smart-write** — `consolidation.account.write()` (`account_consolidation/models/consolidation_coa.py:191-201`, `ConsolidationAccount.write`): when the M2M arrives as a `[(6,0,ids)]` replace command AND the account spans companies outside `env.companies`, it rewrites the command into incremental `(4,..)`/`(3,..)` adds/removes to preserve out-of-scope companies' mappings. Impact: scripted bulk `(6,0,...)` update does NOT replace wholesale — produces a merge.

**ACG8. filtered_used_in_ids depends on context['chart_id']** — `_compute_filtered_used_in_ids` (`account_consolidation/models/consolidation_coa.py:218-230`, `ConsolidationAccount._compute_filtered_used_in_ids`) returns ALL `used_in_ids` when `context['chart_id']` is absent, else filters to that chart. Same pattern for `account.account.consolidation_account_chart_filtered_ids` (`account_consolidation/models/account_account.py:19-32`, `Account`). Impact: building actions outside standard navigation (which sets `chart_id` in context) produces inflated lists.

**ACG9. _name_search supports code prefix shortcut** — `consolidation.account._name_search` (`account_consolidation/models/consolidation_coa.py:280-288`, `ConsolidationAccount._name_search`): builds `['|', ('code','=ilike', name.split(' ')[0]+'%'), ('name', operator, name)]`. Typing `100` matches `code='100xx'` AND name. Impact: numeric searches behave differently from typical ilike; migration scripts using plain ilike may not match the same records.

## Currency & Rates

**ACG10. Currency modes need a rate; silent fallback** — `currency_mode` (`account_consolidation/models/consolidation_coa.py:165`, `ConsolidationAccount.currency_mode`) selects which `currency_rate_<mode>` to use; conversion in `consolidation.company_period._convert` (`account_consolidation/models/consolidation_period.py:728-745`, `ConsolidationCompanyPeriod._convert`) returns the amount UNCHANGED if the rate is falsy (`if currency_rate: return amount/currency_rate`). For `hist`, `_apply_historical_rates` (`:670-688`) falls back to `currency._convert()` if `consolidation.rate.get_rate_for` returns False (`account_consolidation/models/consolidation_rate.py:36`, `ConsolidationRate.get_rate_for`). Impact: missing/zero rate produces wrong consolidated balance, no warning.

**ACG11. rate_consolidation per company_period** — Each company in each period has independent `rate_consolidation` (`account_consolidation/models/consolidation_period.py:548`, `ConsolidationCompanyPeriod.rate_consolidation`), applied in `_apply_consolidation_rate` as `(rate/100.0)*amount` (`:747-755`). No automatic propagation from `res.currency.rate`. Impact: forgotten rate update = stale FX in next consolidation.

**ACG12. originating_currency_id determined by source type** — `consolidation.journal._compute_originating_fields` (`account_consolidation/models/consolidation_journal.py:72-85`, `ConsolidationJournal._compute_originating_fields`): `company_period_id` → `currency_company_id`; else `composition_id` → `using_chart_currency_id`. Impact: sub-consolidations carry a different originating currency/rate than direct companies — easy to mis-debug.

## Sign & Inversion

**ACG13. Sign cascades multiplicatively** — `consolidation.account._compute_sign` (`account_consolidation/models/consolidation_coa.py:257-260`, `ConsolidationAccount._compute_sign`): `sign = (-1 if invert else 1) * (group_id or chart_id).sign`; group multiplies its parent (`:341-344`, `recursive=True`); chart base (`:56-59`). Impact: 3 nested inversions produce -1*-1*-1 = -1; counterintuitive double-flip.

**ACG14. Sign doesn't change stored balance** — `sign` is a compute, never written to `amount`. `consolidation.journal.line.amount` (`account_consolidation/models/consolidation_journal.py:132`, `ConsolidationJournalLine.amount`) keeps original sign. Impact: SQL/`_read_group` against `amount` produce raw values; reports apply sign on render.

## Sub-consolidations

**ACG15. composition_id forbids parallel company_period_id** — `consolidation.journal._check_unique_origin` (`account_consolidation/models/consolidation_journal.py:35-42`, `ConsolidationJournal._check_unique_origin`). Journal must link to ONE source. Impact: a chart consolidating BOTH direct companies AND another chart's results requires TWO separate journals (one per origin type).

**ACG16. children_ids M2M self-ref via custom rel table** — `account_consolidation_inner_rel` join (`account_consolidation/models/consolidation_coa.py:24-27`, `ConsolidationChart.children_ids`). Chart hierarchy traversal via M2M (NO `_parent_store` on `consolidation.chart`). Impact: cycles theoretically possible — no automatic cycle check on the chart M2M.

**ACG17. Sub-consolidation triggers RECURSIVE journal regen** — *(v17 only — `account_consolidation` was removed from EE in v18+; verify against v17 source.)* `consolidation.period.composition._generate_journal` (`account_consolidation/models/consolidation_period.py:429-452`, `ConsolidationPeriodComposition._generate_journal`) calls `self.composed_period_id.action_generate_journals()` (`:443`) before reading totals. Closed composed period is skipped by the `state=='closed'` guard in `action_generate_journals` (`:161`), so the parent reuses whatever lines exist. Impact: closing a child period before the parent locks the parent into stale child data.

## Period Management

**ACG18. dashboard_sections is JSON Text field** — `_compute_dashboard_sections` (`account_consolidation/models/consolidation_period.py:45-59`, `ConsolidationPeriod._compute_dashboard_sections`) stores `json.dumps(...)` of group totals from a `_read_group`. Impact: parsing required client-side; not directly searchable.

**ACG19. company_unmapped_accounts_counts is JSON Text** — `_compute_company_unmapped_accounts_counts` (`account_consolidation/models/consolidation_period.py:86-110`, `ConsolidationPeriod._compute_company_unmapped_accounts_counts`), same `json.dumps` pattern, intersects `record.company_period_ids` companies with `env.user.company_ids`. Impact: counts shown via OWL component, no list-view sorting; companies the user can't access are silently excluded.

**ACG20. generate_guessed_company_periods runs ONCE on onchange** — `account_consolidation/models/consolidation_period.py:112-122`, `ConsolidationPeriod.generate_guessed_company_periods`: guarded by `if not self.company_period_ids:`. Once company periods exist, the `date_analysis_end` onchange does nothing. Impact: changing `chart_id` after generation doesn't refresh the company list.

## Dashboard & Reports

**ACG21. show_on_dashboard at GROUP level only** — `show_on_dashboard` lives on `consolidation.group` (`account_consolidation/models/consolidation_coa.py:300`, `ConsolidationGroup.show_on_dashboard`); dashboard query filters `group_id.show_on_dashboard=True` (`account_consolidation/models/consolidation_period.py:52`, `ConsolidationPeriod._compute_dashboard_sections`). No account-level flag. Impact: granular account display requires one-account groups (clutter workaround).

**ACG22. consolidated_balance_report uses a CUSTOM HANDLER, not the standard engine** — `account_consolidation/report/trial_balance.py:11-13` , `TrialBalanceCustomHandler` (modelo `consolidation.trial.balance.report.handler`; modulo retirado en v18, la fuente es EE17) `_inherit='account.report.custom.handler'`, lines built in `_dynamic_lines_generator` (`:27`) via `report/builder/` + `report/handler/`. The `account.report` record references it via `custom_handler_model_id` (`account_consolidation/data/consolidated_balance_report.xml:11`, `consolidated_balance_report` (modulo retirado en v18, la fuente es EE17)). Impact: extending this report requires the `account.report` custom-handler pattern, not column/expression edits.

**ACG23. balance computed via _read_group of journal lines** — `consolidation.journal._compute_balance` (`account_consolidation/models/consolidation_journal.py:60-70`, `ConsolidationJournal._compute_balance`) runs one grouped `amount:sum` query. Impact: dashboard slow with thousands of journal lines per period.

## Onboarding

**ACG24. Onboarding steps tied to module install** — `consolidation.chart.action_save_onboarding_consolidation_step` and siblings validate by xml_id via `onboarding.onboarding.step` (`account_consolidation/models/consolidation_coa.py:110-113`, `ConsolidationChart.action_save_onboarding_consolidation_step`, `:130-132`; `account_consolidation/models/consolidation_period.py:253-257`, `ConsolidationPeriod.action_save_onboarding_create_step`). Impact: re-installing module re-arms onboarding; users see "first consolidation" prompt again.

**ACG25. setting_consolidation_action returns "last chart"** — `account_consolidation/models/consolidation_coa.py:95-108`, `ConsolidationChart.setting_consolidation_action`: opens `search([], order='id desc', limit=1)`. Impact: in a DB that already has charts, the onboarding "Create" button opens the existing latest chart instead of creating a new one — mid-onboarding confusion.

## Journal Lines & Audit

**ACG26. Auto-generated journal lines are read-only** — `consolidation.journal.line.write()` raises `UserError` if the parent journal is `auto_generated` (`account_consolidation/models/consolidation_journal.py:155-158`, `ConsolidationJournalLine.write`); `_unlink_except_autogenerated` (`:160-163`) blocks unlink unless `context['allow_unlink']` is set. Impact: scripts/imports touching generated lines must pass `with_context(allow_unlink=True)` and cannot `write()` at all — must regenerate.

**ACG27. _check_conditional_unicity exempts hist mode** — `account_consolidation/models/consolidation_journal.py:138-152`, `ConsolidationJournalLine._check_conditional_unicity`: one line per (account, auto-generated journal) is enforced EXCEPT when `account.currency_mode == 'hist'`, because historical accounts emit one line per move line (`_get_historical_journal_lines_values`, `account_consolidation/models/consolidation_period.py:690-703`, `ConsolidationCompanyPeriod._get_historical_journal_lines_values`). Impact: a `hist` account legitimately produces many lines per journal — don't "dedupe" them.

**ACG28. hist accounts explode journal line count** — Non-hist accounts produce ONE aggregated line (`_get_total_balance_and_audit_lines` via `_read_group`, `account_consolidation/models/consolidation_period.py:641-651`, `ConsolidationCompanyPeriod._get_total_balance_and_audit_lines`); hist accounts produce ONE LINE PER posted move line (`:690-703`). Impact: switching an account to `hist` mode can multiply journal line volume by orders of magnitude, degrading dashboard/report performance.

## v17 → v18+ Migration

**ACG29. No upstream migration script** — Odoo 18 doesn't ship `migrations/` for `account_consolidation` (module absent entirely). Impact: upgrade leaves orphan tables/`ir_model` rows; cleanup may need manual `DELETE FROM ir_model WHERE model LIKE 'consolidation.%'`. See `v19_changes.md` for the full SQL.

**ACG30. account.account orphan columns** — v17 adds `consolidation_account_ids` (M2M, `account_consolidation/models/account_account.py:10`, `class Account`) and `consolidation_color` (`:17`); `account.move.line` gains `consolidation_journal_line_ids` (`account_consolidation/models/account_move.py:9`, `class AccountMoveLine` (v17: 18.0 retiró el módulo)); `res.company` gains `consolidation_color` (`account_consolidation/models/res_company.py:13`, `class Company`). M2M columns live in relation tables, not the base table, but `consolidation_color` is a real `account_account`/`res_company` column. Impact: audit before dropping; queries referencing these post-strip fail.

**ACG31. Onboarding records require manual cleanup** — `onboarding.onboarding`, `onboarding.onboarding.step` records (`models/onboarding_onboarding.py`, `models/onboarding_onboarding_step.py`) persist if module manually removed. Impact: settings → onboarding shows broken steps.
