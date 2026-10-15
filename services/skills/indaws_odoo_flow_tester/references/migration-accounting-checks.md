# Migration Accounting Integrity Checks

Reference for verifying that **accounting and master data** survive an Odoo migration intact. Used as part of `odoo-flow-tester` Modo B (Verification) when the user provides a migration analysis document, or stand-alone via `scripts/verify_migration_integrity.py`.

This complements the SQL post-upgrade checks in `developer-skills:odoo-database-upgrader` (FASE 5). That skill runs SQL on the Postgres of the upgraded dump; this one runs over the live API on **both ends** (origin and target), so it works for any migration path:

- Upgrade by Odoo S.A. (v15 → v18, etc.)
- Manual rebuild on a fresh DB with imported opening balances
- Database split / merge / company copy
- Re-platform (vN → vN, different host/edition)

---

## When to use

- After any data migration where the customer expects "los números cuadran".
- Pre-migration: build the **baseline** as part of the pre-migration checklist (the consultant blocks the cut-over until origin numbers are frozen).
- Post-migration: run the **compare** against the target before sign-off.
- Recurring: re-run after each correction round, until 100% green.

---

## Critical input: migration cut-off date (`--created-before`)

This script supports the realistic case where the **source DB keeps producing new records after the cut-over** while the target DB is a frozen snapshot. To compare like-for-like, both sides must be filtered to records that existed at the moment of the migration. That filter is the `--created-before YYYY-MM-DD` flag, applied to `record.create_date` everywhere.

**This date is known by the consultant who ran the migration**, not by the customer — the customer rarely knows the exact day the dump was taken. Before running the verifier, the consultant must look up the cut-off date from their own migration log (or from the upgrade.odoo.com job timestamp).

There is no sane default. If you do not pass `--created-before` the script compares the full DB on each side, including drift in both DBs since the cut-over (in the source: new entries posted after migration; in the target: any manual changes done in the staging DB), which produces noisy false positives. Always set it.

A safe rule: pick the **end of the month immediately before the cut-over**. If the migration happened in February 2026, set `--created-before 2026-01-31`. This excludes anything created in February at all and avoids ambiguity around the exact day the dump was taken.

`create_date` is preserved bit-for-bit by Odoo S.A.'s upgrade service (verified empirically on Sucemart v16→v18). So filtering by it on both sides yields the same set of records on both DBs.

`--created-after` is also available for symmetric windowing if you only want to verify a specific year of activity (e.g. `--created-after 2025-01-01 --created-before 2025-12-31` for "the 2025 movements only"). Use that only when the customer explicitly wants a delta, not a snapshot — for absolute trial balances `--created-after` would drop the opening balance and make balances meaningless.

---

## Workflow (4 fases)

```
+-----------------+     +----------------+     +----------------+     +----------------+
| 1. EXTRACT      | --> | 2. MIGRATE     | --> | 3. COMPARE     | --> | 4. SIGN-OFF    |
| origin -> JSON  |     | (out of scope) |     | target vs base |     | client ack     |
+-----------------+     +----------------+     +----------------+     +----------------+
```

### 1. Extract baseline (origin)

Connect `odoo-pilot` to the origin DB. Run:

```bash
python3 scripts/verify_migration_integrity.py extract \
  --pilot-dir <path>/odoo-pilot/scripts \
  --output baseline_<client>_<YYYYMMDD>.json \
  --checks all
```

Outputs a JSON with every aggregate the comparator needs. Store it under version control alongside the migration plan. **Freeze the origin DB before extracting** — if origin keeps moving, the baseline is invalid.

### 2. Migrate

Out of scope for this skill. See `developer-skills:odoo-database-upgrader` for upgrade-service path, or follow the migration plan defined by the consultant.

### 3. Compare (target)

Re-point `odoo-pilot` env to the target DB. Run:

```bash
python3 scripts/verify_migration_integrity.py compare \
  --pilot-dir <path>/odoo-pilot/scripts \
  --baseline baseline_<client>_<YYYYMMDD>.json \
  --output report_<client>_<YYYYMMDD>.json \
  --tolerance-amount 0.01 \
  --tolerance-pct 0.0
```

Generates a JSON report and a Markdown summary. Default tolerance is 0.01 EUR per amount comparison and 0% on counts (counts must match exactly unless stated otherwise).

### 4. Sign-off

Convert the report into the client deliverable using `templates/migration_integrity_report.md`. Get the financial controller to confirm in writing before turning off the origin instance.

---

## Checks catalogue

The script runs these checks. Each maps to a `check_id` you can include or exclude with `--checks`.

### Global integrity

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `global_balance` | `sum(debit) - sum(credit)` over all posted `account.move.line` | 0.01 | YES |
| `global_balance_per_company` | Same, per `company_id` | 0.01 | YES |
| `global_balance_per_currency` | Same, per `currency_id` (using `amount_currency`) | 0.01 | YES |

A non-zero global balance means the origin is already broken or the migration lost lines. Stop and investigate before any per-account work.

### Trial balance (Sumas y Saldos)

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `trial_balance` | Per `account.account` (code), sum debit, sum credit, balance, line count. Honors `--as-of-date` / `--from-date`. | 0.01 / count exact | YES |
| `trial_balance_per_year` | Same, partitioned by fiscal year | 0.01 / count exact | YES |
| `trial_balance_per_period` | Same, partitioned by period (month) | 0.01 | NO (slow, opt-in) |

For **"sumas y saldos cuadran a una fecha"**, run `trial_balance` with `--as-of-date 2024-12-31` (or whichever cut-off date the controller wants to lock). The script applies the date filter to all accounting checks (`global_balance`, `partner_ledger_*`, `tax_totals_*`, `move_state_count`, `move_count_per_year`, `hash_chain_count`, `unreconciled_amount`, `stock_valuation_total`) so the entire baseline reflects the same point in time. **Master-data checks ignore the date** (they are current-state by definition).

Common drift sources:
- Account code renumbering during migration → match by code, not by id.
- Account merge (two origin accounts → one target) → expect the post-merge balance to be the sum; flag in baseline metadata.
- New "Asset/Liability transfer" entries created by the migration tool (Odoo S.A.) → these should be zero-net per account.

### Partner ledger (Mayor por tercero)

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `partner_ledger_receivable` | Per `partner_id`, balance on accounts with `account_type='asset_receivable'` | 0.01 | YES |
| `partner_ledger_payable` | Per `partner_id`, balance on accounts with `account_type='liability_payable'` | 0.01 | YES |
| `partner_count_with_balance` | Number of partners with non-zero open balance | exact | YES |

Reasoning: a wrong partner mapping during migration silently re-labels invoices to the wrong customer. The global balance still cuadra. The partner ledger does not.

### Reconciliation

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `reconciliation_rate` | Per reconcilable account, ratio of `full_reconcile_id IS NOT NULL` lines | -5pp | NO |
| `unreconciled_amount` | Per reconcilable account, abs sum of unreconciled balance | 0.01 | YES |

A drop in reconciliation rate is expected in some migration paths (the reconciliation tables are not always migrated). What is **not** acceptable is a change in the unreconciled amount: open invoices are open invoices, before and after.

### Document state preservation

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `move_state_count` | Count `account.move` per (journal, move_type, state) | exact | YES |
| `posted_before_count` | Count moves with `posted_before=True` (legal lock) | exact | YES |
| `move_count_per_year` | Count moves per fiscal year | exact | YES |
| `invoice_name_preservation` | Per posted invoice/bill, `name` (legal number) preserved literally — or moved to `ref` / configured legacy field | per-row classification | YES |

`invoice_name_preservation` is the answer to "el número de factura `FAC/2010/00123` tiene que seguir siendo `FAC/2010/00123`". Pre-v12 migrations are the typical reason this becomes a problem: the legacy `account.invoice.number` lived in a separate field and may have ended up in `name`, in `ref`, in a custom field, or lost. The check matches origin↔target invoices by `(journal_code, invoice_date, partner_vat, amount_total, currency_code)` and classifies each:

- `preserved` — `target.name` literally equals `baseline.name`
- `moved_to_ref` — number ended up in `ref`
- `moved_to_custom` — number ended up in a configured legacy field (use `--legacy-number-field x_old_number`)
- `mismatch` — the invoice exists but the number doesn't match anywhere
- `not_found_in_target` — the matching key is not in target
- `ambiguous_match` — multiple target invoices share the same key (data quality smell, investigate)

The check FAILS if any of `mismatch`, `not_found_in_target` or `ambiguous_match` is non-zero. `moved_to_ref` / `moved_to_custom` do not fail the check (the number is not lost, just relocated) but are reported so the consultant can decide whether the relocation is acceptable.

Spain-specific: `posted_before` must be preserved for any move covered by the audit trail. Losing `posted_before=True` means the customer can `button_draft()` legally locked entries.

### Spain-specific (l10n_es)

Run only if `l10n_es` or any of its EDI submodules is installed.

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `hash_chain_count` | Per journal with `restrict_mode_hash_table=True`, count of moves with `secure_sequence_number IS NOT NULL` | exact | YES |
| `aeat_module_state` | Periods declared per AEAT model (303, 347, 349, 390) | exact | NO |
| `sii_state_count` | Count `account.move` per `l10n_es_edi_sii_state` (sent/registered/cancelled) | exact | NO |
| `tbai_state_count` | Same for TicketBAI (`l10n_es_edi_tbai_state`) | exact | NO |
| `verifactu_state_count` | Same for Verifactu | exact | NO |

Hash chain integrity is the hardest check to fix post-cutover. If it breaks, the customer cannot post new invoices in the affected journals. This must be 100% green before sign-off.

### Tax totals

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `tax_totals_per_period` | Per `account.tax.id`, per **year**, sum of `balance` on tax-line move lines | 0.01 | YES |
| `tax_totals_per_quarter` | Per `account.tax.id`, per **quarter** (or arbitrary `--periods`), sum of `balance` | 0.01 | YES |
| `mod303_per_period` | Per **mod303 casilla** (`account.account.tag` `+01`/`-03`/...), per `--periods`, sum of `balance` | 0.01 | YES |
| `tax_count_active` | Count of active `account.tax` per `type_tax_use` | exact | NO |

`tax_totals_per_quarter` and `mod303_per_period` are the right tools for "el 303 del Q3 y Q4 cuadran". Use `--periods 2024Q3,2024Q4` (or `--periods 2024-09,2024-10,2024-11,2024-12` for monthly REDEME). `mod303_per_period` aggregates by **casilla** (e.g. `+01` Régimen general base, `+03` Régimen general cuota, `+28` Soportado intracom.) which is what the customer's controller reads on the form.

If tax totals drift, the next AEAT declaration will not match what the previous quarter reported.

### Stock valuation (when stock is migrated)

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `stock_valuation_total` | Sum `stock.valuation.layer.value` per (location, product category) | 0.01 | YES |
| `stock_qty_per_product` | Sum on-hand qty per `product.product` (active products only) | 0 | YES |

The stock valuation must equal the balance of the inventory account at migration date.

### Assets

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `asset_count_per_category` | Count `account.asset` per category, per state | exact | YES |
| `asset_value_per_category` | Sum `original_value` (gross) and `book_value` (net) per category & state | 0.01 | YES |

> **v18+ note**: `value_residual` became a non-stored computed field, so it cannot be aggregated via `read_group`. The check uses `book_value` (stored, equals gross minus accumulated depreciation posted) which is the figure the controller signs off in practice. Add a per-asset `value_residual` spot-check post-migration if the customer specifically asks for it.
| `depreciation_moves_count` | Count of posted `account.move` with `asset_id IS NOT NULL` | exact | YES |

### Master data smoke tests

| check_id | What | Tolerance | Critical |
|---|---|---|---|
| `partner_count_active` | Count `res.partner` with `active=True` | exact | NO |
| `product_count_active` | Count `product.product` per type | exact | NO |
| `journal_count` | Count `account.journal` per type | exact | YES |
| `account_count_per_type` | Count `account.account` per `account_type` | exact | YES |
| `currency_rate_count` | Count `res.currency.rate` records per currency | exact | NO |

Active is intentional — inactive partners/products are noise.

---

## Tolerance philosophy

- **Amounts**: default 0.01 (cent rounding). For multi-currency baselines, raise to 0.05 if the migration recomputes rates.
- **Counts**: exact. A different count means a real difference.
- **Percentages**: only on reconciliation rate — a 5-percentage-point drop is acceptable provided `unreconciled_amount` is unchanged.

Never silently widen tolerance. If a check fails, the consultant must:
1. Investigate the root cause.
2. Document the diff in the migration log.
3. Either fix the data or get explicit client written approval to accept the discrepancy.
4. Only then add an `accepted_diffs` entry to the baseline so the next compare run does not re-flag it.

---

## Output format

### Baseline JSON

```json
{
  "metadata": {
    "extracted_at": "2026-05-03T10:14:22+00:00",
    "odoo_url": "https://origin.example.com",
    "odoo_db": "origin_db",
    "odoo_version": "saas~17.4+e",
    "extraction_runtime_seconds": 84.2,
    "checks": ["global_balance", "trial_balance", "partner_ledger_receivable", "..."],
    "scope": {
      "companies": [1, 2],
      "fiscal_years": [2024, 2025],
      "currency": "EUR"
    }
  },
  "results": {
    "global_balance": {
      "value": 0.0,
      "details": {"debit_total": 1234567.89, "credit_total": 1234567.89, "line_count": 84231}
    },
    "trial_balance": {
      "rows": [
        {"account_code": "430000", "account_name": "Clientes", "account_type": "asset_receivable",
         "debit": 154321.00, "credit": 142100.00, "balance": 12221.00, "line_count": 421},
        ...
      ]
    },
    ...
  }
}
```

### Compare report JSON

```json
{
  "metadata": {
    "compared_at": "2026-05-03T16:42:11+00:00",
    "baseline_extracted_at": "2026-05-03T10:14:22+00:00",
    "target_odoo_url": "https://target.example.com",
    "target_odoo_db": "target_db",
    "target_odoo_version": "18.0+e",
    "tolerance": {"amount": 0.01, "pct": 0.0}
  },
  "summary": {
    "total_checks": 12,
    "passed": 10,
    "failed": 2,
    "skipped": 0,
    "critical_failed": 1
  },
  "results": [
    {
      "check_id": "global_balance",
      "status": "pass",
      "baseline": 0.0,
      "target": 0.0,
      "diff": 0.0
    },
    {
      "check_id": "trial_balance",
      "status": "fail",
      "critical": true,
      "rows_with_diff": [
        {"account_code": "430000", "field": "balance",
         "baseline": 12221.00, "target": 12220.00, "diff": -1.00}
      ]
    }
  ]
}
```

---

## Common gotchas (Spain context)

1. **`account_type` renamed in v17+** — pre-v17 used `user_type_id` linking to `account.account.type`. Post-v17 uses `account_type` selection field directly. The script handles both.
2. **`balance` is a computed read-only field on `account.account`** in v17+ — query `account.move.line` and aggregate, do not read `account.account.balance` directly.
3. **`parent_state` filter is mandatory** — only `parent_state='posted'` lines should count. Drafts and cancelled moves contaminate the trial balance.
4. **Multi-company**: always filter by `company_id`. A consolidated view across companies is not what the customer signs off on — they sign off per legal entity.
5. **Hash chain reset**: if the migration intentionally resets the chain (Odoo S.A. upgrade does this in some scenarios), document it in the baseline metadata and exclude `hash_chain_count` from the compare.
6. **Date filter for partner ledger**: when comparing post-migration, filter `date <= migration_cutoff_date` so any new entries the customer posts on the target after go-live do not contaminate the comparison.

---

## Integration with the rest of the skill

- The catalog (`scripts/flows.py`) is for **transactional** flows (create SO, post invoice). This reference is for **integrity** of already-migrated data. Different concern, complementary.
- For Mode C (Execution), do not run accounting flows on a target until this integrity check passes. Otherwise you contaminate the data you are trying to verify.
- For the test taxonomy, this corresponds to type 8 `DATA_INTEGRITY`, subtypes `count_comparison`, `balance_comparison`, `valuation_comparison`. See `references/test-taxonomy.md`.

---

## Recipes (típicos de Indaws)

### "Que las sumas y saldos cuadren a 31/12/2024" (migración hecha en feb 2025)

```bash
# Origen
python3 verify_migration_integrity.py extract \
  --pilot-dir <path> --output baseline_yearend.json \
  --checks trial_balance,partner_ledger_receivable,partner_ledger_payable,unreconciled_amount,hash_chain_count \
  --as-of-date 2024-12-31 \
  --created-before 2025-01-31    # PREGUNTAR fecha real al consultor / cliente

# Destino
python3 verify_migration_integrity.py compare \
  --pilot-dir <path> --baseline baseline_yearend.json \
  --output report_yearend.json
```

### "Que el modelo 303 cuadra para Q3 y Q4 de 2024"

```bash
# Origen
python3 verify_migration_integrity.py extract \
  --pilot-dir <path> --output baseline_303.json \
  --checks tax_totals_per_quarter,mod303_per_period \
  --periods 2024Q3,2024Q4

# Destino
python3 verify_migration_integrity.py compare \
  --pilot-dir <path> --baseline baseline_303.json \
  --output report_303.json
```

`mod303_per_period` es el que cuadra casilla a casilla con el formulario que se presentó a la AEAT. `tax_totals_per_quarter` es el corte por impuesto (útil para detectar dónde está la diferencia si una casilla descuadra).

### "Que el número de las facturas migradas se conserva"

Si la migración pre-v12 ya fue limpia y el número quedó en `name`:

```bash
# Origen
python3 verify_migration_integrity.py extract \
  --pilot-dir <path> --output baseline_inv.json \
  --checks invoice_name_preservation
```

Si en origen el número legacy está en un campo custom (típico de migraciones manuales antiguas), pasa el campo:

```bash
python3 verify_migration_integrity.py extract \
  --pilot-dir <path> --output baseline_inv.json \
  --checks invoice_name_preservation \
  --legacy-number-field x_legacy_number
```

El compare clasifica cada factura en `preserved` / `moved_to_ref` / `moved_to_custom` / `mismatch` / `not_found_in_target` / `ambiguous_match`. El sign-off requiere `mismatch == 0`, `not_found_in_target == 0` y `ambiguous_match == 0`. Las relocations (`moved_to_ref`, `moved_to_custom`) son válidas pero deben justificarse en el log de migración.

### "Todo a la vez en una migración real"

```bash
# Origen
python3 verify_migration_integrity.py extract \
  --pilot-dir <path> --output baseline_full.json \
  --as-of-date 2024-12-31 \
  --created-before 2025-01-31    \
  --periods 2024Q3,2024Q4 \
  --legacy-number-field x_legacy_number

# Destino (después de migrar)
python3 verify_migration_integrity.py compare \
  --pilot-dir <path> --baseline baseline_full.json \
  --output report_full.json
```

`--created-before` se hereda automáticamente del baseline en la fase compare; no hay que pasarlo otra vez.

---

## Sign-off checklist (paste into client report)

Before turning off the origin instance, the consultant must confirm:

- [ ] Baseline extracted from a frozen origin DB
- [ ] All `critical: YES` checks pass on target
- [ ] Any non-critical failures are documented and accepted by the client controller
- [ ] Hash chain (l10n_es) is intact OR documented reset is approved
- [ ] Partner ledger matches per legal entity
- [ ] Reconciliation: unreconciled amount unchanged
- [ ] Stock valuation matches the inventory accounts on the trial balance at cut-over date
- [ ] Asset gross and residual values match per category
- [ ] Tax totals match for the last 4 closed periods (last full year)
- [ ] Client controller's written sign-off is attached to the migration log
