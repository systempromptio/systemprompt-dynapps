# OCA Accounting — How It Patches/Extends the Core

Where OCA accounting modules hook into standard Odoo, where they CONFLICT, and the friction points when you migrate or code on top. Source: `OCA/v18/`.

---

## 1. Install-level conflicts & replacements (read first)

| OCA module | Relationship to core | Friction |
|------------|----------------------|----------|
| `account_asset_management` | **Excludes** native `account_asset` (`account/__manifest__.py:11`) | Can't run both. EE→CE asset migration = data conversion, not a swap. |
| `account_reconcile_oca` | **Replaces** the reconciliation widget (OWL client action + `account.reconcile.model` OCA variant) | On Enterprise (`account_accountant`) the two compete for the reconciliation job. Pick one. |
| `account_payment_order` + SEPA suite | **Parallel** payment pipeline to Odoo's native `account.payment` register | Two ways to pay an invoice (register payment vs payment order). Reports/automation must know which flow the client uses. |
| `account_financial_report` | **Adds** CE reports Odoo CE lacks (Trial Balance, GL, Aged) | Pure additive. On Enterprise these overlap with native Accounting reports — duplicate menus confuse users. |
| `account_analytic_required` | **Constrains** `account.move.line` posting | Adds a wall that didn't exist; legacy data/imports without analytic now fail (OAG46-48). |

## 2. The payment-order pipeline vs native `account.payment`

OCA `account_payment_order` introduces a batch workflow that does NOT use Odoo's "Register Payment" wizard:

```
invoice (posted) → payment.line (via "Add to payment order" / mode on invoice)
  → account.payment.order  draft → open → generated → uploaded
       draft2open:  DELETES + recreates account.payment (OAG6)
       open2generated:  generate_payment_file() → SEPA PAIN XML (stub in base, OAG7)
       generated2uploaded:  stamps mandate seq type + last_debit_date (OAG14)
  → bank file (PAIN.001 CT / PAIN.008 DD)
```

Friction points:
- `account_payment_mode` decides which journal + method each payment uses; a mode pointing at the wrong journal type only errors at `draft2open` (OAG12).
- SEPA DD sequence type (FRST/RCUR/OOFF/FNAL) is mandate-state-driven and only finalized at upload — a failed upload risks re-sending FRST (OAG14). This is the single most common SEPA support ticket.
- Grouping collapses destination accounts to the first line (OAG9) — don't group payments across different counterpart accounts.

## 3. Reconciliation: OCA widget vs Enterprise

`account_reconcile_oca` reimplements the bank-statement reconciliation as an OWL client action on `account.bank.statement.line`. The matching engine is `account.reconcile.model` (the OCA rule variant in `account_reconcile_model_oca`).

- It writes to the SAME `account.move`/`account.move.line`/`account.partial.reconcile` core tables — so once reconciled, downstream behavior is identical to core.
- The fragility is in the widget's compute path: FX zero-division (OAG1), half-computed auto-reconcile (OAG2), single-line assumptions (OAG3), sign flip on partial clamp (OAG4). These are widget bugs, not data-model issues.
- `unique_matching=True` on a rule that returns nothing (OAG5) reads to users as "the rule is broken".

## 4. l10n_es_aeat_sii_oca / verifactu — the EDI layer over `account.move`

The Spanish OCA EDI modules hook `account.move._post()` (or a queued cron) to push each invoice to AEAT:
- **SII** (`l10n_es_aeat_sii_oca`): SOAP/zeep, per-invoice state tracking on the move, registration keys mapped from tax/fiscal-position. <3-day legal window.
- **Verifactu** (`l10n_es_verifactu_oca`): chaining model (`verifactu.chaining`) hashes invoices in sequence — a gap or out-of-order post breaks the chain.
- **Friction with Indaws:** Indaws ships `indaws_l10n_es_edi_sii` and `indaws_l10n_es_edi_verifactu` as bridges/enhancements ON TOP of the OCA modules (NOT replacements). If both an OCA EDI module and the Indaws bridge are installed, confirm which one actually owns the `_post` hook to avoid double submission.

## 5. AEAT declaration framework — reads `account.move` by tax mapping

`l10n_es_aeat` (base) + the mod1xx/mod3xx/mod34x models compute declarations by querying `account.move.line` over a date range and mapping taxes via `l10n.es.aeat.map.tax`. Key frictions:
- A declaration is a **snapshot** — recalculating ("Calculate" button) re-queries; a posted invoice edited after calculation desyncs the declaration until recalculated.
- Tax mapping is per-module config; a custom tax not in the map silently doesn't appear in the declaration.
- `mod390`/`mod190`/`mod296` (annual summaries) DEPEND on their quarterly counterparts' data being present.

## 6. Multi-currency: the recurring latent bug across OCA reports

Several OCA financial modules sum amounts without converting currency — same class of bug as the Indaws/inBuilt cost summaries:
- `account_financial_report` trial balance merges multi-currency accounts by `account_id` (OAG32) and distorts FX initial balances (OAG31).
- `account_cutoff_base` per-line rounding can imbalance (OAG44).
- `account_payment_order` line FX date defaults to today, revaluing late (OAG11).

When auditing a multi-currency client on OCA reporting, cross-check totals against the native GL.

## 7. Overlap & division of labor with Indaws

| Need | OCA module | Indaws equivalent | Who wins |
|------|-----------|-------------------|----------|
| SEPA CT/DD files | `account_banking_sepa_*` | uses OCA directly | OCA (Indaws does NOT reimplement) |
| Reconciliation UI (CE) | `account_reconcile_oca` | — | OCA (Indaws uses EE or OCA) |
| Fixed € discount on lines | `account_invoice_fixed_discount` | `indaws_account_triple_discount` | Indaws (more features) |
| Invoice sections by SO | `account_invoice_section_sale_order` | `indaws_invoice_section` / `section.line.mixin` | Indaws (hierarchical tree) |
| Global discount | `account_global_discount` | `indaws_sale_pricelist_global_rule` | Different scope — can coexist |
| SII / Verifactu | `l10n_es_aeat_sii_oca` / `l10n_es_verifactu_oca` | `indaws_l10n_es_edi_*` bridges ON TOP | Both — bridge enhances OCA |
| Asset management (CE) | `account_asset_management` | — | OCA |
| Financial reports (CE) | `account_financial_report` | — | OCA |

**Rule of thumb:** for Spanish localization plumbing (AEAT, SII, Verifactu, SEPA, N43) Indaws builds ON the OCA stack. For commercial logic (discounts, sections, rappel, composition) Indaws replaces/supersedes the equivalent OCA module. Never install both an Indaws and OCA module that own the same field — check ownership first.

## 8. v19 migration friction (see [[v19_changes]])

- OCA v19 lost 80%+ of modules (not yet ported, not removed). **`l10n_es_aeat_mod303`, SII, Verifactu, Facturae, vat_book are MISSING in v19 OCA** = blocking for any Spanish client until ported.
- `account_reconcile_oca` (CE reconciliation) is GONE in v19 OCA — a CE client migrating loses the reconciliation widget.
- `account_asset_management`, `account_cutoff_*`, `account_payment_return` all unported as of the v19 snapshot → budget manual porting.

---

## Cross-references
- Gotchas → [[gotchas]]
- Catalog → [[catalog]]
- l10n-spain → [[l10n_spain]]
- Core account flow → [[../../functional/account/core]]
- SEPA/ISO20022 → [[../../functional/account/integrations]]
- Indaws suite → [[../indaws/catalog]]
