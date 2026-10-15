<!-- DEPRECATED v17-only: el módulo account_consolidation fue ELIMINADO en Odoo 18 (no existe en v18/v19 CE ni EE). Este conocimiento solo aplica a instalaciones legacy v17. NO usar para proyectos Odoo 18+. Citas verificadas contra fuente v17 (presente en EE 16-17, eliminado en v18): `cite_audit --methods account_consolidation --roots <ee>/17` da 0 sím✗. Con los roots por defecto v18/v19 salen marcadas = artefacto de versión, NO deuda. -->

> ⚠️ **v17-only — módulo eliminado en Odoo 18.** No aplica a proyectos v18+; conservado solo para legacy v17.

# account_consolidation — v18 / v19 Status

> **EE only · v17 last available version.** There is no v18/v19 module — the only "change" is its REMOVAL. v17 `path:line` citations are relative to `odoo_enterprise/17/account_consolidation/`.

## Module Removed

`account_consolidation` is **not present** in `odoo_enterprise/18/` or `odoo_enterprise/19/`.

It was last shipped in **Odoo 17.0 Enterprise** (`odoo_enterprise/17/account_consolidation/`). v17 `account/__manifest__.py:7` declares `depends: ['account_reports','web_grid']` and `'application': True` — so the upgrade dependency that disappears is the standalone Consolidation app, not a sub-feature of `account_reports`.

Verified via:
```bash
find "$ODOO_ENTERPRISE_SRC" -maxdepth 2 -type d -name "*consolid*"
# odoo_enterprise/15/l10n_be_us_consolidation_demo
# odoo_enterprise/15/account_consolidation
# odoo_enterprise/16/l10n_be_us_consolidation_demo
# odoo_enterprise/16/account_consolidation
# odoo_enterprise/17/l10n_be_us_consolidation_demo
# odoo_enterprise/17/account_consolidation
# (no entries for 18 or 19)
```

## Implications for v17 → v18 / v19 Upgrades

### 1. Standard Upgrade Service Behaviour

`upgrade.odoo.com` will report `account_consolidation` as "removed" / "to_remove". Database tables (`consolidation_chart`, `consolidation_account`, `consolidation_period`, etc.) remain in PostgreSQL after upgrade but no module owns them — orphan tables and `ir_model` rows.

### 2. Pre-Upgrade Snapshot

Before submitting v17 → v18 upgrade dump:
```sql
-- Dump consolidation data to JSON
SELECT json_agg(row_to_json(c)) FROM consolidation_chart c;
SELECT json_agg(row_to_json(a)) FROM consolidation_account a;
SELECT json_agg(row_to_json(p)) FROM consolidation_period p;
SELECT json_agg(row_to_json(j)) FROM consolidation_journal j;
SELECT json_agg(row_to_json(jl)) FROM consolidation_journal_line jl;
```

Save externally for post-migration reconstruction.

### 3. Post-Upgrade Cleanup SQL

Run after upgrade succeeds (and after data extracted):

```sql
-- Remove orphan models
DELETE FROM ir_model_fields WHERE model LIKE 'consolidation.%';
DELETE FROM ir_model_access WHERE model_id IN (SELECT id FROM ir_model WHERE model LIKE 'consolidation.%');
DELETE FROM ir_rule WHERE model_id IN (SELECT id FROM ir_model WHERE model LIKE 'consolidation.%');
DELETE FROM ir_model_data WHERE model LIKE 'consolidation.%';
DELETE FROM ir_model WHERE model LIKE 'consolidation.%';

-- Remove orphan tables (after data is preserved!)
DROP TABLE IF EXISTS consolidation_journal_line CASCADE;
DROP TABLE IF EXISTS consolidation_journal CASCADE;
DROP TABLE IF EXISTS consolidation_period_composition CASCADE;
DROP TABLE IF EXISTS consolidation_company_period CASCADE;
DROP TABLE IF EXISTS consolidation_period CASCADE;
DROP TABLE IF EXISTS consolidation_rate CASCADE;
DROP TABLE IF EXISTS consolidation_account CASCADE;
DROP TABLE IF EXISTS consolidation_group CASCADE;
DROP TABLE IF EXISTS consolidation_chart CASCADE;
DROP TABLE IF EXISTS account_consolidation_inner_rel CASCADE;
DROP TABLE IF EXISTS consolidation_accounts_rel CASCADE;
```

```sql
-- Remove orphan columns/relations added to standard models by v17 _inherit.
-- NOTE: consolidation_account_chart_filtered_ids is a COMPUTED, non-stored field
--       (models/account_account.py:11-16) → NO column exists, do NOT try to drop it.
-- Real stored artifacts to clean:
ALTER TABLE account_account DROP COLUMN IF EXISTS consolidation_color;   -- models/account_account.py:17 (related, store=False → may not exist; audit first)
ALTER TABLE res_company   DROP COLUMN IF EXISTS consolidation_color;     -- models/res_company.py:13 (stored=True → real column)
-- M2M relation tables (the M2M fields themselves have no base-table column):
DROP TABLE IF EXISTS account_account_consolidation_account_rel CASCADE;            -- account.account.consolidation_account_ids (models/account_account.py:10)
DROP TABLE IF EXISTS account_move_line_consolidation_journal_line_rel CASCADE;     -- account.move.line.consolidation_journal_line_ids (models/account_move.py:9)
-- (exact rel table names are auto-generated; confirm via information_schema before dropping)
```

### 4. Replacement Strategies

| Need | v18+ Approach |
|------|---------------|
| Cross-company P&L view | `account.report` (Profit & Loss) with multi-company filter |
| Multi-currency consolidation | Manual rate journals + `account_inter_company_rules` |
| Hierarchical account groups | `account.group` (standard) + custom `account.report` engine |
| Sub-consolidation | Multiple `account.report` records with parent/child computation |
| FX rate per period | `res.currency.rate` (standard) + cron snapshots |
| Locked period state | `account.move` lock_date + Fiscal Lock dates |
| Dashboard sections | `account.report` columns + `spreadsheet.dashboard` |

### 5. Client Communication

Customers expecting consolidation in v18+ should be told **clearly**:
> "Odoo Consolidation app was discontinued in v18. Your v17 setup will not migrate automatically. We recommend [option from table above] OR keeping v17 long-term OR adopting external consolidation software."

### 6. Alternative OCA / Custom Modules

No mature OCA replacement available as of v18.0. Some community partners have built bespoke modules but these are NOT certified by Odoo SA. Risk profile high for finance audits.

## v19 Specific

No changes from v18: still removed. v19 introduces no replacement either.

## v19.2 Status

Verified absent in `odoo_enterprise/19.2/` directory listing — the module continues NOT to be re-added in any v19.x point release.
